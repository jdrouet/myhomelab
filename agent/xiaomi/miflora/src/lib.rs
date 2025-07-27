use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Context;
use btleplug::api::{Central, CentralEvent, CentralState, Manager, Peripheral, ScanFilter};
use btleplug::platform::PeripheralId;
use myhomelab_agent_prelude::collector::Collector;
use myhomelab_agent_prelude::sensor::BuildContext;
use myhomelab_event::EventLevel;
use myhomelab_metric::entity::value::MetricValue;
use myhomelab_metric::entity::{Metric, MetricTags};
use myhomelab_prelude::Healthcheck;
use myhomelab_prelude::time::current_timestamp;
use tokio::sync::RwLock;
use tokio::time::Interval;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

pub mod device;
mod event;

// Device name (e.g. Flower care)
// MAC address prefix (C4:7C:8D = original)

const DEVICE: &str = "xiaomi-miflora";

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

impl myhomelab_agent_prelude::sensor::SensorBuilder for MifloraSensorConfig {
    type Output = MifloraSensor;

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
            collector: ctx.collector.clone(),
            check_interval: tokio::time::interval(Duration::from_millis(self.check_interval)),
            sync_interval: Duration::from_millis(self.sync_interval),
            memory: memory.clone(),
        };
        let task = tokio::task::spawn(async move { runner.run().await });
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
    adapter: btleplug::platform::Adapter,
    cancel: CancellationToken,
    collector: C,
    check_interval: Interval,
    sync_interval: Duration,
    memory: Arc<RwLock<HashMap<PeripheralId, DeviceHistory>>>,
}

impl<C: Collector> MifloraRunner<C> {
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

                let address = peripheral.address();
                self.collector
                    .push_event(event::DeviceEvent::new(
                        address,
                        EventLevel::Info,
                        "device discovered",
                    ))
                    .await?;

                let _ = self.action_tx.send(Action::Synchronize {
                    force: true,
                    peripheral_id: id.clone(),
                });
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
        let peripheral = match self.adapter.peripheral(&id).await {
            Ok(inner) => inner,
            Err(btleplug::Error::DeviceNotFound) => {
                return Ok(());
            }
            Err(other) => {
                return Err(anyhow::Error::from(other).context("getting peripheral"));
            }
        };
        peripheral.connect().await.context("connecting")?;
        let device = crate::device::MiFloraDevice::new(&peripheral)
            .await
            .context("creating device")?;

        let now = current_timestamp();
        let battery = device
            .read_battery()
            .await
            .context("reading battery level")?;
        let data = device
            .read_realtime_data()
            .await
            .context("reading realtime data")?;

        peripheral.disconnect().await.context("disconnecting")?;

        let tags = MetricTags::default()
            .with_tag("device", DEVICE)
            .maybe_with_tag("name", device.name())
            .with_tag("address", device.address());
        let _ = self
            .collector
            .push_metrics(&[
                Metric {
                    name: "device.battery".into(),
                    tags: Cow::Borrowed(&tags),
                    timestamp: now,
                    value: MetricValue::gauge(battery as f64),
                },
                Metric {
                    name: "measurement.temperature".into(),
                    tags: Cow::Borrowed(&tags),
                    timestamp: now,
                    value: MetricValue::gauge(data.temperature),
                },
                Metric {
                    name: "measurement.moisture".into(),
                    tags: Cow::Borrowed(&tags),
                    timestamp: now,
                    value: MetricValue::gauge(data.moisture as f64),
                },
                Metric {
                    name: "measurement.light".into(),
                    tags: Cow::Borrowed(&tags),
                    timestamp: now,
                    value: MetricValue::gauge(data.light as f64),
                },
                Metric {
                    name: "measurement.conductivity".into(),
                    tags: Cow::Borrowed(&tags),
                    timestamp: now,
                    value: MetricValue::gauge(data.conductivity as f64),
                },
            ])
            .await;

        self.memory
            .write()
            .await
            .entry(peripheral.id())
            .or_default()
            .synced();

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
pub struct MifloraSensor {
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    #[allow(unused)]
    memory: Arc<RwLock<HashMap<PeripheralId, DeviceHistory>>>,
    task: tokio::task::JoinHandle<anyhow::Result<()>>,
}

impl Healthcheck for MifloraSensor {
    async fn healthcheck(&self) -> anyhow::Result<()> {
        if self.task.is_finished() {
            Err(anyhow::anyhow!("sensor task is dead"))
        } else {
            Ok(())
        }
    }
}

impl myhomelab_agent_prelude::sensor::Sensor for MifloraSensor {
    type Cmd = MifloraCommand;

    async fn execute(&self, command: Self::Cmd) -> anyhow::Result<()> {
        self.action_tx
            .send(command.into())
            .context("sending action to the action queue")
    }

    fn name(&self) -> &'static str {
        DEVICE
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
}

impl DeviceHistory {
    fn should_sync(&self, sync_interval: Duration) -> bool {
        self.last_sync
            .is_none_or(|last| last + sync_interval < SystemTime::now())
    }

    fn synced(&mut self) {
        self.last_sync = Some(SystemTime::now());
    }
}

#[derive(Clone, Debug)]
enum Action {
    Synchronize {
        force: bool,
        peripheral_id: PeripheralId,
    },
    SynchronizeAll {
        force: bool,
    },
}

impl From<MifloraCommand> for Action {
    fn from(value: MifloraCommand) -> Self {
        match value {
            MifloraCommand::SynchronizeAll => Action::SynchronizeAll { force: true },
        }
    }
}
