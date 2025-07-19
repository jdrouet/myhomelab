use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Context;
use btleplug::api::{Central, CentralEvent, CentralState, Manager, Peripheral, ScanFilter};
use btleplug::platform::PeripheralId;
use myhomelab_agent_prelude::mpsc::Sender;
use myhomelab_agent_prelude::reader::BuildContext;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

pub mod device;

// Device name (e.g. Flower care)
// MAC address prefix (C4:7C:8D = original)

// const DEVICE: &str = "xiaomi-miflora";

#[derive(Debug, serde::Deserialize)]
pub struct MifloraReaderConfig {}

impl myhomelab_agent_prelude::reader::ReaderBuilder for MifloraReaderConfig {
    type Output = MifloraReader;

    async fn build<S: Sender>(&self, ctx: &BuildContext<S>) -> anyhow::Result<Self::Output> {
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

        let memory: Arc<RwLock<HashSet<PeripheralId>>> = Default::default();
        let runner = MifloraRunner {
            adapter,
            cancel: ctx.cancel.child_token(),
            memory: memory.clone(),
        };
        let task = tokio::task::spawn(async move { runner.run().await });
        Ok(MifloraReader { memory, task })
    }
}

struct MifloraRunner {
    adapter: btleplug::platform::Adapter,
    cancel: CancellationToken,
    memory: Arc<RwLock<HashSet<PeripheralId>>>,
}

impl MifloraRunner {
    #[tracing::instrument(skip(self), err)]
    async fn handle_discovered(&self, id: &PeripheralId) -> anyhow::Result<()> {
        if self.memory.read().await.contains(&id) {
            tracing::trace!("known peripheral, skipping");
            return Ok(());
        }
        let peripheral = self
            .adapter
            .peripheral(&id)
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
            self.memory.write().await.insert(id.clone());
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    async fn scan(&self) -> anyhow::Result<()> {
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
                _ = self.cancel.cancelled() => {
                    tracing::trace!("cancellation requested");
                    // nothing to do, the loop will abort
                }
            }
        }
        Ok(())
    }

    async fn run(&self) -> anyhow::Result<()> {
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
    #[allow(unused)]
    memory: Arc<RwLock<HashSet<PeripheralId>>>,
    task: tokio::task::JoinHandle<anyhow::Result<()>>,
}

impl myhomelab_agent_prelude::reader::Reader for MifloraReader {
    async fn wait(self) -> anyhow::Result<()> {
        self.task.await?
    }
}
