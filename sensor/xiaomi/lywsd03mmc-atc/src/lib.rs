use std::borrow::Cow;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

use anyhow::Context;
use btleplug::api::{
    BDAddr, Central, CentralEvent, Manager, Peripheral, PeripheralProperties, ScanFilter,
};
use btleplug::platform::PeripheralId;
use lru::LruCache;
use myhomelab_metric::entity::value::MetricValue;
use myhomelab_metric::entity::{Metric, MetricTags};
use myhomelab_prelude::time::current_timestamp;
use myhomelab_sensor_prelude::collector::Collector;
use myhomelab_sensor_prelude::sensor::{
    BasicTaskSensor, BuildContext, SensorBuilder, SensorDescriptor,
};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;
use uuid::Uuid;

mod event;
mod parse;

const DEVICE: &str = "xiaomi-lywsd03mmc-atc";
const DESCRIPTOR: SensorDescriptor = SensorDescriptor {
    id: DEVICE,
    name: "Xiaomi Lywsd03mmc sensor",
    description: "Bluetooth reader of Xiaomi Lywsd03mmc with ATC firmware",
};

const RUNNER_NAMESPACE: &str = "xiaomi_lywsd03mmc_atc::runner";

const SERVICE_ID: uuid::Uuid = uuid::Uuid::from_u128(488837762788578050050668711589115);

#[derive(Debug, serde::Deserialize)]
pub struct SensorConfig {
    cache_size: NonZeroUsize,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            cache_size: NonZeroUsize::new(10).unwrap(),
        }
    }
}

impl SensorBuilder for SensorConfig {
    type Output = AtcSensor;

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

        let runner = SensorRunner {
            adapter,
            cache: LruCache::new(self.cache_size),
            cancel: ctx.cancel.child_token(),
            collector: ctx.collector.clone(),
        };
        let task = tokio::spawn(runner.run().instrument(tracing::info_span!("runner")));

        Ok(BasicTaskSensor::new(DESCRIPTOR, task))
    }
}

#[derive(Debug)]
struct AddressWrapper(BDAddr);

impl serde::Serialize for AddressWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = self.0.to_string();
        serializer.serialize_str(value.as_str())
    }
}

#[derive(Debug, serde::Serialize)]
struct Device {
    name: Option<String>,
    address: AddressWrapper,
}

impl From<PeripheralProperties> for Device {
    fn from(value: PeripheralProperties) -> Self {
        Self {
            name: value.local_name,
            address: AddressWrapper(value.address),
        }
    }
}

impl Device {
    fn populate(&self, header: &mut MetricTags) {
        header.maybe_set_tag("name", self.name.as_deref());
        header.set_tag("address", self.address.0.to_string());
    }
}

struct SensorRunner<C: Collector> {
    adapter: btleplug::platform::Adapter,
    cache: LruCache<PeripheralId, Arc<Device>>,
    cancel: CancellationToken,
    collector: C,
}

impl<C: Collector> SensorRunner<C> {
    async fn device(&mut self, id: &PeripheralId) -> Option<Arc<Device>> {
        if let Some(device) = self.cache.get(id) {
            Some(device.clone())
        } else if let Ok(peripheral) = self.adapter.peripheral(id).await {
            let mut device = Device {
                address: AddressWrapper(peripheral.address()),
                name: None,
            };
            if let Ok(Some(props)) = peripheral.properties().await {
                device.name = props.local_name;
            }
            let device = Arc::new(device);
            self.cache.push(id.clone(), device.clone());
            Some(device)
        } else {
            None
        }
    }

    async fn push(&mut self, id: PeripheralId, values: impl Iterator<Item = (&'static str, f64)>) {
        let timestamp = current_timestamp();
        let mut tags = MetricTags::default().with_tag("device", DEVICE);
        if let Some(device) = self.device(&id).await {
            device.populate(&mut tags);
        } else {
            tags.set_tag("peripheral_id", id.to_string());
        }
        let metrics = values
            .map(|(name, value)| Metric {
                name: name.into(),
                tags: Cow::Borrowed(&tags),
                timestamp,
                value: MetricValue::gauge(value),
            })
            .collect::<Vec<_>>();
        let _ = self.collector.push_metrics(&metrics).await;
    }

    #[tracing::instrument(parent = None, target = RUNNER_NAMESPACE, skip(self), err)]
    async fn handle_discovered(&mut self, id: PeripheralId) -> anyhow::Result<()> {
        let peripheral = self.adapter.peripheral(&id).await?;
        peripheral.discover_services().await?;
        if peripheral
            .services()
            .iter()
            .any(|srv| srv.uuid == SERVICE_ID)
        {
            if let Some(properties) = peripheral.properties().await? {
                let device = Arc::new(Device::from(properties));
                let event = crate::event::DeviceDiscoveredEvent::new(device.clone());
                self.collector.push_event(event).await?;
                self.cache.push(id, device);
            }
        }
        Ok(())
    }

    #[tracing::instrument(parent = None, target = RUNNER_NAMESPACE, skip(self), err)]
    async fn handle_advertisement(
        &mut self,
        id: PeripheralId,
        service_data: HashMap<Uuid, Vec<u8>>,
    ) -> anyhow::Result<()> {
        if let Some(data) = service_data.get(&SERVICE_ID) {
            let mut values = Vec::with_capacity(3);
            if let Some(battery) = parse::read_battery(data) {
                values.push(("device.battery", battery as f64));
            }
            if let Some(temperature) = parse::read_temperature(data) {
                values.push(("measurement.temperature", temperature as f64));
            }
            if let Some(humidity) = parse::read_humidity(data) {
                values.push(("measurement.humidity", humidity as f64));
            }
            self.push(id, values.into_iter()).await;
        }
        Ok(())
    }

    async fn handle_event(&mut self, event: CentralEvent) -> anyhow::Result<()> {
        match event {
            CentralEvent::DeviceDiscovered(id) if !self.cache.contains(&id) => {
                self.handle_discovered(id).await?;
            }
            CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                self.handle_advertisement(id, service_data).await?;
            }
            _ => {}
        }
        Ok(())
    }

    #[tracing::instrument(target = RUNNER_NAMESPACE, skip_all)]
    async fn scan(&mut self) -> anyhow::Result<()> {
        tracing::info!("starting reader");
        self.adapter.start_scan(ScanFilter::default()).await?;
        tracing::info!("preparing reader");
        let mut events = self.adapter.events().await?;
        while !self.cancel.is_cancelled() {
            tokio::select! {
                Some(event) = events.next() => {
                    self.handle_event(event).await?;
                }
                _ = self.cancel.cancelled() => {
                    // nothing to do
                }
            }
        }
        tracing::info!("stopped reader");
        Ok(())
    }

    #[tracing::instrument(target = RUNNER_NAMESPACE, skip_all)]
    async fn run(mut self) -> anyhow::Result<()> {
        tracing::info!("starting");
        while !self.cancel.is_cancelled() {
            self.scan().await?;
        }
        tracing::info!("completed");
        Ok(())
    }
}

pub type AtcSensor = BasicTaskSensor;
