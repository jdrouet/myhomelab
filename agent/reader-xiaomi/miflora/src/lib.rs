use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Context;
use btleplug::api::{Central, CentralEvent, CentralState, Manager, Peripheral, ScanFilter};
use btleplug::platform::PeripheralId;
use myhomelab_agent_prelude::collector::Collector;
use myhomelab_agent_prelude::reader::BuildContext;
use tokio::sync::RwLock;
use tokio::time::Interval;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

pub mod device;

// Device name (e.g. Flower care)
// MAC address prefix (C4:7C:8D = original)

// const DEVICE: &str = "xiaomi-miflora";

#[derive(Debug, serde::Deserialize)]
pub struct MifloraReaderConfig {
    #[serde(default = "MifloraReaderConfig::default_check_interval")]
    check_interval: u64,
    #[serde(default = "MifloraReaderConfig::default_sync_interval")]
    sync_interval: u64,
}

impl MifloraReaderConfig {
    const fn default_check_interval() -> u64 {
        // every hour
        1000 * 60 * 60
    }

    const fn default_sync_interval() -> u64 {
        // every day
        1000 * 60 * 60 * 24
    }
}

impl Default for MifloraReaderConfig {
    fn default() -> Self {
        Self {
            check_interval: Self::default_check_interval(),
            sync_interval: Self::default_sync_interval(),
        }
    }
}

impl myhomelab_agent_prelude::reader::ReaderBuilder for MifloraReaderConfig {
    type Output = MifloraReader;

    async fn build<C: Collector>(&self, ctx: &BuildContext<C>) -> anyhow::Result<Self::Output> {
        let manager = btleplug::platform::Manager::new()
            .await
            .context("getting bluetooth manager")?;
        let adapters = manager
            .adapters()
            .await
            .context("getting bluetooth adapters")?;
        let adapter = adapters
            .into_iter()
            .nth(0)
            .ok_or_else(|| anyhow::anyhow!("no bluetooth adapter found"))?;

        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();

        let memory: Arc<RwLock<HashMap<PeripheralId, DeviceHistory>>> = Default::default();
        let runner = MifloraRunner {
            action_tx: action_tx.clone(),
            action_rx,
            adapter,
            cancel: ctx.cancel.child_token(),
            check_interval: tokio::time::interval(Duration::from_millis(self.check_interval)),
            sync_interval: Duration::from_millis(self.sync_interval),
            memory: memory.clone(),
        };
        let task = tokio::task::spawn(async move { runner.run().await });
        Ok(MifloraReader {
            action_tx,
            memory,
            task,
        })
    }
}

struct MifloraRunner {
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    action_rx: tokio::sync::mpsc::UnboundedReceiver<Action>,
    adapter: btleplug::platform::Adapter,
    cancel: CancellationToken,
    check_interval: Interval,
    sync_interval: Duration,
    memory: Arc<RwLock<HashMap<PeripheralId, DeviceHistory>>>,
}

impl MifloraRunner {
    #[tracing::instrument(skip(self), err)]
    async fn handle_discovered(&self, id: &PeripheralId) -> anyhow::Result<()> {
        if self.memory.read().await.contains_key(id) {
            tracing::trace!("known peripheral, skipping");
            return Ok(());
        }
        let peripheral = self
            .adapter
            .peripheral(id)
            .await
            .context("accessing peripheral")?;
        let props = peripheral
            .properties()
            .await
            .context("reading properties")?;
        let Some(name) = props.and_then(|props| props.local_name) else {
            tracing::trace!("peripheral has no name");
            return Ok(());
        };
        if name == "Flower care" {
            let mut memory = self.memory.write().await;
            if memory.insert(id.clone(), Default::default()).is_none() {
                tracing::debug!("discovered new device");
            }
        }
        Ok(())
    }

    async fn handle_action(&self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::Synchronize {
                force,
                peripheral_id,
            } => {
                if self
                    .handle_synchronize(force, peripheral_id.clone())
                    .await
                    .is_err()
                {
                    let _ = self.action_tx.send(Action::Synchronize {
                        force,
                        peripheral_id,
                    });
                }
            }
            Action::SynchronizeAll { force } => {
                let _ = self.handle_synchronize_all(force).await;
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    async fn handle_synchronize_all(&self, force: bool) -> anyhow::Result<()> {
        let peripheral_ids = self.memory.read().await.keys().cloned().collect::<Vec<_>>();
        for peripheral_id in peripheral_ids {
            let _ = self.action_tx.send(Action::Synchronize {
                force,
                peripheral_id,
            });
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    async fn handle_synchronize(&self, force: bool, id: PeripheralId) -> anyhow::Result<()> {
        let device = self
            .memory
            .read()
            .await
            .get(&id)
            .cloned()
            .unwrap_or_default();
        if !force && !device.should_sync(self.sync_interval) {
            tracing::debug!("no need to synchronize device, skipping");
            return Ok(());
        }
        let peripheral = self
            .adapter
            .peripheral(&id)
            .await
            .context("getting peripheral")?;
        peripheral.connect().await.context("connecting")?;
        let device = crate::device::MiFloraDevice::new(&peripheral)
            .await
            .context("creating device")?;
        let data = device
            .read_history_data()
            .await
            .context("reading history data")?;
        tracing::warn!(message = "received data", values = ?data);
        // TODO
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    async fn scan(&mut self) -> anyhow::Result<()> {
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .context("starting scan")?;
        let mut events = self
            .adapter
            .events()
            .await
            .context("accessing bluetooth events")?;
        while !self.cancel.is_cancelled() {
            tokio::select! {
                maybe_event = events.next() => {
                    match maybe_event {
                        Some(CentralEvent::DeviceDiscovered(id)) => {
                            let _ = self.handle_discovered(&id).await;
                        }
                        Some(CentralEvent::StateUpdate(CentralState::PoweredOff)) => {
                            tracing::warn!("peripheral powered off");
                        }
                        Some(CentralEvent::StateUpdate(CentralState::PoweredOn)) => {
                            tracing::info!("peripheral powered on");
                        }
                        Some(_) => {},
                        None => {
                            tracing::warn!("event stream closed");
                            break;
                        }
                    }
                }
                Some(action) = self.action_rx.recv() => {
                    let _ = self.handle_action(action).await;
                }
                _ = self.cancel.cancelled() => {
                    tracing::trace!("cancellation requested");
                    // nothing to do, the loop will abort
                }
                _ = self.check_interval.tick() => {
                    let _ = self.handle_synchronize_all(false).await;
                }
            }
        }
        Ok(())
    }

    async fn run(mut self) -> anyhow::Result<()> {
        tracing::info!("starting");
        while !self.cancel.is_cancelled() {
            self.scan().await?;
        }
        tracing::info!("completed");
        Ok(())
    }
}

#[derive(Debug)]
pub struct MifloraReader {
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    #[allow(unused)]
    memory: Arc<RwLock<HashMap<PeripheralId, DeviceHistory>>>,
    task: tokio::task::JoinHandle<anyhow::Result<()>>,
}

impl MifloraReader {
    pub fn execute(&self, action: Action) -> anyhow::Result<()> {
        self.action_tx
            .send(action)
            .context("sending action to the action queue")
    }
}

impl myhomelab_agent_prelude::reader::Reader for MifloraReader {
    async fn wait(self) -> anyhow::Result<()> {
        self.task.await?
    }
}

#[derive(Clone, Debug, Default)]
struct DeviceHistory {
    last_sync: Option<SystemTime>,
}

impl DeviceHistory {
    fn should_sync(&self, sync_interval: Duration) -> bool {
        self.last_sync
            .is_none_or(|last| last + sync_interval < SystemTime::now())
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    Synchronize {
        force: bool,
        peripheral_id: PeripheralId,
    },
    SynchronizeAll {
        force: bool,
    },
}
