use std::num::NonZeroUsize;

use btleplug::api::{
    BDAddr, Central, CentralEvent, Manager, Peripheral, PeripheralProperties, ScanFilter,
};
use btleplug::platform::PeripheralId;
use lru::LruCache;
use myhomelab_agent_prelude::mpsc::Sender;
use myhomelab_metric::entity::value::MetricValue;
use myhomelab_metric::entity::{Metric, MetricHeader, MetricTags};
use myhomelab_prelude::{current_timestamp, parse_from_env};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

mod parse;

const DEVICE: &str = "xiaomi-lywsd03mmc-atc";
const SERVICE_ID: uuid::Uuid = uuid::Uuid::from_u128(488837762788578050050668711589115);

#[derive(Debug)]
pub struct ReaderXiaomiConfig {
    enabled: bool,
    cache_size: NonZeroUsize,
}

impl Default for ReaderXiaomiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_size: NonZeroUsize::new(10).unwrap(),
        }
    }
}

impl myhomelab_prelude::FromEnv for ReaderXiaomiConfig {
    fn from_env() -> anyhow::Result<Self> {
        let enabled = parse_from_env::<bool>("MYHOMELAB_READER_XIAOMI_LYWSD03MMC_ATC_ENABLED")?
            .unwrap_or(false);
        let cache_size =
            parse_from_env::<NonZeroUsize>("MYHOMELAB_READER_XIAOMI_LYWSD03MMC_ATC_CACHE_SIZE")?
                .unwrap_or(NonZeroUsize::new(10).unwrap());
        Ok(Self {
            enabled,
            cache_size,
        })
    }
}

impl ReaderXiaomiConfig {
    pub async fn build(&self) -> anyhow::Result<Option<ReaderXiaomi>> {
        if !self.enabled {
            return Ok(None);
        }

        let cache = LruCache::new(self.cache_size);
        let manager = btleplug::platform::Manager::new().await.unwrap();
        // get the first bluetooth adapter
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().nth(0).unwrap();

        Ok(Some(ReaderXiaomi {
            cache,
            manager,
            central,
            filter: ScanFilter {
                services: vec![SERVICE_ID],
            },
        }))
    }
}

struct Device {
    name: Option<String>,
    address: BDAddr,
}

impl From<PeripheralProperties> for Device {
    fn from(value: PeripheralProperties) -> Self {
        Self {
            name: value.local_name,
            address: value.address,
        }
    }
}

impl Device {
    fn populate(&self, header: &mut MetricTags) {
        header.maybe_set_tag("name", self.name.as_deref());
        header.set_tag("address", self.address.to_string());
    }
}

#[derive(Debug)]
pub struct ReaderXiaomi {
    cache: LruCache<PeripheralId, Device>,
    #[allow(unused)]
    manager: btleplug::platform::Manager,
    central: btleplug::platform::Adapter,
    filter: ScanFilter,
}

impl ReaderXiaomi {
    async fn push<S: Sender + Send>(
        &mut self,
        id: PeripheralId,
        sender: &S,
        values: impl Iterator<Item = (&'static str, f64)>,
    ) {
        let timestamp = current_timestamp();
        let mut tags = MetricTags::default().with_tag("device", DEVICE);
        if let Some(device) = self.cache.get(&id) {
            device.populate(&mut tags);
        }
        for (name, value) in values {
            let _ = sender
                .push(Metric {
                    header: MetricHeader::new(name, tags.clone()),
                    timestamp,
                    value: MetricValue::gauge(value),
                })
                .await;
        }
    }

    async fn handle_event<S: Sender + Send>(
        &mut self,
        event: CentralEvent,
        sender: &S,
    ) -> anyhow::Result<()> {
        match event {
            CentralEvent::DeviceDiscovered(id) => {
                let peripheral = self.central.peripheral(&id).await?;
                if let Some(properties) = peripheral.properties().await? {
                    self.cache.push(id, Device::from(properties));
                }
            }
            CentralEvent::ServiceDataAdvertisement { id, service_data } => {
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
                    self.push(id, sender, values.into_iter()).await;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl myhomelab_agent_prelude::reader::Reader for ReaderXiaomi {
    #[tracing::instrument(skip_all)]
    async fn run<S: Sender + Send>(
        mut self,
        token: CancellationToken,
        sender: S,
    ) -> anyhow::Result<()> {
        tracing::info!("preparing reader");
        let mut events = self.central.events().await?;
        tracing::info!("starting reader");
        self.central.start_scan(self.filter.clone()).await?;
        while !token.is_cancelled() {
            tokio::select! {
                Some(event) = events.next() => {
                    self.handle_event(event, &sender).await?;
                }
                _ = token.cancelled() => {
                    self.central.stop_scan().await?;
                }
            }
        }
        tracing::info!("stopped reader");
        Ok(())
    }
}
