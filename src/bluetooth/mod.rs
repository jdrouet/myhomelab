use std::time::Duration;

use anyhow::Context;
use bluer::AdapterEvent;
use opentelemetry::KeyValue;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub(crate) struct BluetoothConfig {}

impl crate::Configurable for BluetoothConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl BluetoothConfig {
    pub(crate) async fn build(
        &self,
        cancel_token: CancellationToken,
    ) -> anyhow::Result<BluetoothCollector> {
        let session = bluer::Session::new()
            .await
            .context("unable to create session")?;
        let adapter = session
            .default_adapter()
            .await
            .context("unable to find default adapter")?;

        let meter = opentelemetry::global::meter("bluetooth");

        Ok(BluetoothCollector {
            adapter,
            cancel_token,
            events_counter: meter
                .u64_counter("bluetooth.events")
                .with_description("Number of events received")
                .build(),
            device_counter: meter
                .u64_gauge("bluetooth.devices")
                .with_description("Number of discovered devices")
                .build(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct BluetoothCollector {
    adapter: bluer::Adapter,
    cancel_token: CancellationToken,
    events_counter: opentelemetry::metrics::Counter<u64>,
    device_counter: opentelemetry::metrics::Gauge<u64>,
}

impl BluetoothCollector {
    #[tracing::instrument(parent = None, skip(self), err(Debug))]
    async fn handle_event(&self, event: AdapterEvent) -> anyhow::Result<()> {
        match event {
            AdapterEvent::DeviceAdded(address) => {
                self.events_counter.add(
                    1,
                    &[
                        KeyValue::new("kind", "device-added"),
                        KeyValue::new("address", address.to_string()),
                    ],
                );
            }
            AdapterEvent::DeviceRemoved(address) => {
                self.events_counter.add(
                    1,
                    &[
                        KeyValue::new("kind", "device-removed"),
                        KeyValue::new("address", address.to_string()),
                    ],
                );
            }
            AdapterEvent::PropertyChanged(_) => {
                self.events_counter
                    .add(1, &[KeyValue::new("kind", "property-changed")]);
            }
        }
        Ok(())
    }

    #[tracing::instrument(parent = None, skip(self), err(Debug))]
    async fn handle_heartbeat(&self) -> anyhow::Result<()> {
        let addresses = self.adapter.device_addresses().await?;
        self.device_counter.record(addresses.len() as u64, &[]);
        Ok(())
    }

    #[tracing::instrument(skip(self), err(Debug))]
    pub(crate) async fn run(self) -> anyhow::Result<()> {
        tracing::info!("starting reader");
        self.adapter
            .set_powered(true)
            .await
            .context("unable to turn on adapter")?;
        self.adapter
            .set_discovery_filter(bluer::DiscoveryFilter::default())
            .await
            .context("unable to set discovery filter")?;
        tracing::info!("preparing reader");
        let mut events = self.adapter.discover_devices_with_changes().await?;
        let mut heartbeat = tokio::time::interval(Duration::from_secs(30));
        while !self.cancel_token.is_cancelled() {
            tokio::select! {
                maybe_event = events.next() => {
                    match maybe_event {
                        Some(event) => {
                            let _ = self.handle_event(event).await;
                        }
                        None => break,
                    }
                }
                _ = self.cancel_token.cancelled() => {
                    tracing::info!("shutdown requested");
                }
                _ = heartbeat.tick() => {
                    let _ = self.handle_heartbeat().await;
                }
            }
        }
        Ok(())
    }
}
