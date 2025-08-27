use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use anyhow::Context;
use bluer::AdapterEvent;
use opentelemetry::KeyValue;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

mod xiaomi_lywsd03mmc_atc;
mod xiaomi_miflora;

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
            device_rssi: meter
                .i64_gauge("bluetooth.device.rssi")
                .with_description("Received Signal Strength Indicator")
                .build(),
            //
            xiaomi_lywsd03mmc_atc: Default::default(),
            xiaomi_miflora: Default::default(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct BluetoothCollector {
    adapter: bluer::Adapter,
    cancel_token: CancellationToken,
    events_counter: opentelemetry::metrics::Counter<u64>,
    device_counter: opentelemetry::metrics::Gauge<u64>,
    device_rssi: opentelemetry::metrics::Gauge<i64>,
    //
    xiaomi_lywsd03mmc_atc: xiaomi_lywsd03mmc_atc::XiaomiLywsd03mmcAtcCollector,
    xiaomi_miflora: xiaomi_miflora::XiaomiMifloraCollector,
}

impl BluetoothCollector {
    fn track_event(&self, event: &AdapterEvent) {
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
    }

    #[tracing::instrument(
        parent = None,
        skip(self),
        fields(
            network.peer.address = self.adapter.name(),
            network.protocol.name = "bluetooth",
            otel.status_code = tracing::field::Empty,
            resource.name = "bluetooth/handle_event",
            span.kind = "server",
        ),
        err(Debug),
    )]
    async fn handle_event(&self, event: AdapterEvent) -> anyhow::Result<()> {
        self.track_event(&event);
        let span = tracing::Span::current();
        let AdapterEvent::DeviceAdded(address) = event else {
            span.record("otel.status_code", "OK");
            return Ok(());
        };
        let device = self.adapter.device(address)?;
        let device = DiscoveredDevice::try_from(device).await?;
        if let Some(rssi) = device.rssi {
            self.device_rssi.record(rssi as i64, &device.attributes);
        }
        let Some(device) = self.xiaomi_lywsd03mmc_atc.collect(device)? else {
            span.record("otel.status_code", "OK");
            return Ok(());
        };
        let Some(_device) = self.xiaomi_miflora.collect(device).await? else {
            span.record("otel.status_code", "OK");
            return Ok(());
        };
        tracing::trace!(message = "unsupported device");
        Ok(())
    }

    #[tracing::instrument(
        parent = None,
        skip(self),
        fields(
            resource.name = "bluetooth/handle_heartbeat",
        ),
        err(Debug),
    )]
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

#[derive(Clone, Debug)]
pub struct DiscoveredDevice {
    // pub inner: bluer::Device,
    // pub name: Option<String>,
    pub rssi: Option<i16>,
    pub uuids: HashSet<Uuid>,
    pub service_data: HashMap<Uuid, Vec<u8>>,
    pub attributes: Vec<KeyValue>,
}

impl DiscoveredDevice {
    pub async fn try_from(device: bluer::Device) -> bluer::Result<Self> {
        let name = device.name().await?;
        let uuids = device.uuids().await?;
        let service_data = device.service_data().await?;
        let rssi = device.rssi().await?;

        let mut attributes = Vec::with_capacity(3);
        attributes.push(KeyValue::new("address", device.address().to_string()));
        if let Some(ref name) = name {
            attributes.push(KeyValue::new("name", name.clone()));
        }

        Ok(Self {
            // name,
            rssi,
            uuids: uuids.unwrap_or_default(),
            service_data: service_data.unwrap_or_default(),
            attributes,
            // inner: device,
        })
    }
}
