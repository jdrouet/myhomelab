use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Context;
use bluer::{AdapterEvent, Address, DiscoveryFilter};
use myhomelab_prelude::Healthcheck;
use myhomelab_sensor_prelude::collector::Collector;
use myhomelab_sensor_prelude::sensor::{BuildContext, SensorDescriptor};
use tokio::sync::RwLock;
use tokio::time::Interval;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::device::Miflora;

pub mod device;
mod event;

// Device name (e.g. Flower care)
// MAC address prefix (C4:7C:8D = original)

const RUNNER_NAMESPACE: &str = "xiaomi_miflora::runner";

const DEVICE: &str = "xiaomi-miflora";
const DESCRIPTOR: SensorDescriptor = SensorDescriptor {
    id: DEVICE,
    name: "Xiaomi MiFlora Bluetooth",
    description: "Bluetooth reader for the Xiaomi MiFlora devices",
};

#[derive(Debug, serde::Deserialize)]
pub struct MifloraSensorConfig {
    #[serde(default = "MifloraSensorConfig::default_check_interval")]
    check_interval: u64,
    #[serde(default = "MifloraSensorConfig::default_sync_interval")]
    sync_interval: u64,
}

impl MifloraSensorConfig {
    const fn default_check_interval() -> u64 {
        // every hour
        // 1000 * 60 * 60
        // every 10 min
        1000 * 60 * 10
    }

    const fn default_sync_interval() -> u64 {
        // every day
        // 1000 * 60 * 60 * 24
        // every hour
        1000 * 60 * 60
    }
}

impl Default for MifloraSensorConfig {
    fn default() -> Self {
        Self {
            check_interval: Self::default_check_interval(),
            sync_interval: Self::default_sync_interval(),
        }
    }
}

#[cfg(not(linux))]
impl myhomelab_sensor_prelude::sensor::SensorBuilder for MifloraSensorConfig {
    type Output = MifloraSensor;

    async fn build<C: Collector>(&self, ctx: &BuildContext<C>) -> anyhow::Result<Self::Output> {
        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();

        let memory: Arc<RwLock<HashMap<Address, DeviceHistory>>> = Default::default();
        let cancel = ctx.cancel.child_token();
        let task = tokio::task::spawn(async move {
            cancel.cancelled().await;
            Ok(())
        });
        Ok(MifloraSensor {
            action_tx,
            memory,
            task,
        })
    }
}

#[cfg(linux)]
impl myhomelab_sensor_prelude::sensor::SensorBuilder for MifloraSensorConfig {
    type Output = MifloraSensor;

    async fn build<C: Collector>(&self, ctx: &BuildContext<C>) -> anyhow::Result<Self::Output> {
        let session = bluer::Session::new().await.context("creating session")?;
        let adapter = session
            .default_adapter()
            .await
            .context("getting default adapter")?;

        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();

        let memory: Arc<RwLock<HashMap<Address, DeviceHistory>>> = Default::default();
        let runner = MifloraRunner {
            action_tx: action_tx.clone(),
            action_rx,
            adapter,
            cancel: ctx.cancel.child_token(),
            collector: ctx.collector.clone(),
            check_interval: tokio::time::interval(Duration::from_millis(self.check_interval)),
            sync_interval: Duration::from_millis(self.sync_interval),
            memory: memory.clone(),
        };
        let task = tokio::task::spawn(runner.run().instrument(tracing::info_span!("runner")));
        Ok(MifloraSensor {
            action_tx,
            memory,
            task,
        })
    }
}

struct MifloraRunner<C: Collector> {
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    action_rx: tokio::sync::mpsc::UnboundedReceiver<Action>,
    adapter: bluer::Adapter,
    cancel: CancellationToken,
    collector: C,
    check_interval: Interval,
    sync_interval: Duration,
    memory: Arc<RwLock<HashMap<Address, DeviceHistory>>>,
}

impl<C: Collector> MifloraRunner<C> {
    async fn should_sync(&self, address: Address) -> bool {
        self.memory
            .read()
            .await
            .get(&address)
            .map_or(true, |history| history.should_sync(self.sync_interval))
    }

    async fn mark_synced(&self, address: Address) {
        self.memory
            .write()
            .await
            .entry(address)
            .or_default()
            .failed();
    }

    async fn mark_failed(&self, address: Address) {
        self.memory
            .write()
            .await
            .entry(address)
            .or_default()
            .failed();
    }
}

impl<C: Collector> MifloraRunner<C> {
    async fn try_handle_read(&self, device: Miflora) -> anyhow::Result<()> {
        device.connect().await?;
        let system = device.read_system().await?;
        let values = device.read_realtime_values().await?;
        device.disconnect().await?;
        tracing::info!(message = "data handled", ?system, ?values);
        Ok(())
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn handle_device_change(&self, address: Address) -> anyhow::Result<()> {
        if !self.should_sync(address).await {
            return Ok(());
        }

        let device = self.adapter.device(address).context("getting device")?;
        if crate::device::is_miflora_device(&device)
            .await
            .context("checking if miflora device")?
        {
            self.action_tx.send(Action::Synchronize {
                force: true,
                address,
            })?;
        }

        let Some(device) = crate::device::Miflora::try_from_device(device).await? else {
            return Ok(());
        };

        let res = self.try_handle_read(device).await;
        if res.is_ok() {
            self.mark_synced(address).await;
        } else {
            self.mark_failed(address).await;
        }

        res
    }

    #[tracing::instrument(parent = None, target = RUNNER_NAMESPACE, skip(self), err(Debug))]
    async fn handle_tick(&self) -> anyhow::Result<()> {
        // self.handle_synchronize_all(false).await
        Ok(())
    }

    #[tracing::instrument(parent = None, target = RUNNER_NAMESPACE, skip_all)]
    async fn handle_event(&self, event: AdapterEvent) -> anyhow::Result<()> {
        match event {
            AdapterEvent::DeviceAdded(address) => {
                self.handle_device_change(address).await?;
            }
            AdapterEvent::DeviceRemoved(address) => {
                tracing::debug!(message = "device removed", %address);
            }
            AdapterEvent::PropertyChanged(property) => {
                tracing::debug!(message = "adapter property changed", ?property);
            }
        }
        Ok(())
    }

    #[tracing::instrument(target = RUNNER_NAMESPACE, skip(self), err(Debug))]
    async fn scan(&mut self) -> anyhow::Result<()> {
        self.adapter
            .set_discovery_filter(DiscoveryFilter {
                transport: bluer::DiscoveryTransport::Le,
                ..Default::default()
            })
            .await?;
        let mut events = self
            .adapter
            .discover_devices_with_changes()
            .await
            .context("accessing bluetooth events")?;
        while !self.cancel.is_cancelled() {
            tokio::select! {
                maybe_event = events.next() => {
                    if let Some(event) = maybe_event {
                        let _ = self.handle_event(event).await;
                    }
                }
                Some(action) = self.action_rx.recv() => {
                    // self.handle_action(action).await;
                }
                _ = self.cancel.cancelled() => {
                    tracing::trace!("cancellation requested");
                    // nothing to do, the loop will abort
                }
                _ = self.check_interval.tick() => {
                    let _ = self.handle_tick().await;
                }
            }
        }
        Ok(())
    }

    async fn run(mut self) -> anyhow::Result<()> {
        tracing::info!("starting");
        self.adapter
            .set_powered(true)
            .await
            .context("unable to power adapter")?;
        while !self.cancel.is_cancelled() {
            self.scan().await?;
        }
        tracing::info!("completed");
        Ok(())
    }
}

#[derive(Debug)]
pub struct MifloraSensor {
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    memory: Arc<RwLock<HashMap<Address, DeviceHistory>>>,
    task: tokio::task::JoinHandle<anyhow::Result<()>>,
}

impl Healthcheck for MifloraSensor {
    async fn healthcheck(&self) -> anyhow::Result<()> {
        if self.task.is_finished() {
            Err(anyhow::anyhow!("sensor task is dead"))
        } else {
            tracing::debug!("task is still running");
            Ok(())
        }
    }
}

impl myhomelab_sensor_prelude::sensor::Sensor for MifloraSensor {
    type Cmd = MifloraCommand;

    #[tracing::instrument(skip(self), err(Debug))]
    async fn execute(&self, command: Self::Cmd) -> anyhow::Result<()> {
        self.action_tx
            .send(command.into())
            .context("sending action to the action queue")
    }

    fn descriptor(&self) -> SensorDescriptor {
        DESCRIPTOR
    }

    async fn wait(self) -> anyhow::Result<()> {
        self.task.await?
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "command", rename_all = "kebab-case")]
pub enum MifloraCommand {
    SynchronizeAll,
}

#[derive(Clone, Debug, Default)]
struct DeviceHistory {
    last_sync: Option<SystemTime>,
    count_failure: u8,
    last_failure: Option<SystemTime>,
}

impl DeviceHistory {
    fn failed(&mut self) {
        self.count_failure = self.count_failure.saturating_add(1);
        self.last_failure = Some(SystemTime::now());
    }

    fn should_sync(&self, sync_interval: Duration) -> bool {
        let now = SystemTime::now();
        if let Some(last) = self.last_failure {
            let error_interval = Duration::from_secs(((self.count_failure + 1) as u64) * 10);
            if last + error_interval >= now {
                tracing::trace!("waiting due to last failures");
                return false;
            }
        }
        if let Some(last) = self.last_sync {
            if last + sync_interval >= now {
                tracing::trace!("waiting due to last sync");
                return false;
            }
        }
        true
    }

    fn synced(&mut self) {
        self.count_failure = 0;
        self.last_failure = None;
        self.last_sync = Some(SystemTime::now());
    }
}

#[derive(Clone, Debug)]
enum Action {
    Synchronize { force: bool, address: Address },
    SynchronizeAll { force: bool },
}

impl From<MifloraCommand> for Action {
    fn from(value: MifloraCommand) -> Self {
        match value {
            MifloraCommand::SynchronizeAll => Action::SynchronizeAll { force: true },
        }
    }
}

const DEVICE_UUID_PREFIX: u32 = 0xfe95;

async fn is_miflora_device(device: &bluer::Device) -> anyhow::Result<bool> {
    let Some(service_data) = device
        .service_data()
        .await
        .context("getting service data")?
    else {
        return Ok(false);
    };

    Ok(service_data.iter().any(|(uuid, _data)| {
        let (id, _, _, _) = uuid.as_fields();
        id == DEVICE_UUID_PREFIX
    }))
}
