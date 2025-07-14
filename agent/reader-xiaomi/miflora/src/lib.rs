use std::num::NonZeroUsize;
use std::time::{Duration, SystemTime};

use btleplug::api::{Central, CentralEvent, Manager, Peripheral, ScanFilter};
use btleplug::platform::PeripheralId;
use device::MiFloraDevice;
use lru::LruCache;
use myhomelab_agent_prelude::mpsc::Sender;
use myhomelab_metric::entity::value::MetricValue;
use myhomelab_metric::entity::{Metric, MetricHeader, MetricTags};
use myhomelab_prelude::parse_from_env;
use myhomelab_prelude::time::current_timestamp;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

pub mod device;

// Device name (e.g. Flower care)
// MAC address prefix (C4:7C:8D = original)

const DEVICE: &str = "xiaomi-miflora";
const TIMEOUT: Duration = Duration::from_secs(60 * 60 * 2); // 2h

#[derive(Debug)]
pub struct ReaderConfig {
    enabled: bool,
    cache_size: NonZeroUsize,
}

impl myhomelab_prelude::FromEnv for ReaderConfig {
    fn from_env() -> anyhow::Result<Self> {
        let enabled =
            parse_from_env::<bool>("MYHOMELAB_READER_XIAOMI_MIFLORA_ENABLED")?.unwrap_or(false);
        let cache_size =
            parse_from_env::<NonZeroUsize>("MYHOMELAB_READER_XIAOMI_LYWSD03MMC_ATC_CACHE_SIZE")?
                .unwrap_or(NonZeroUsize::new(20).unwrap());
        Ok(Self {
            enabled,
            cache_size,
        })
    }
}

impl ReaderConfig {
    pub async fn build(&self) -> anyhow::Result<Option<Reader>> {
        if !self.enabled {
            return Ok(None);
        }
        let manager = btleplug::platform::Manager::new().await.unwrap();
        // get the first bluetooth adapter
        let adapters = manager.adapters().await?;
        let adapter = adapters.into_iter().nth(0).unwrap();

        let scan_filter = ScanFilter {
            services: vec![
                uuid::uuid!("00001204-0000-1000-8000-00805f9b34fb"),
                uuid::uuid!("00001206-0000-1000-8000-00805f9b34fb"),
                uuid::uuid!("0000fe95-0000-1000-8000-00805f9b34fb"),
                uuid::uuid!("0000fef5-0000-1000-8000-00805f9b34fb"),
            ],
        };

        Ok(Some(Reader {
            cache: LruCache::new(self.cache_size),
            manager,
            adapter,
            scan_filter,
        }))
    }
}

#[derive(Debug)]
pub struct Reader {
    cache: LruCache<PeripheralId, SystemTime>,
    #[allow(unused)]
    manager: btleplug::platform::Manager,
    adapter: btleplug::platform::Adapter,
    scan_filter: ScanFilter,
}

impl Reader {
    async fn handle_discovered(&mut self, id: PeripheralId) -> anyhow::Result<()> {
        if self
            .cache
            .get(&id)
            .map_or(false, |last_seen| *last_seen + TIMEOUT > SystemTime::now())
        {
            // device already in cache
            return Ok(());
        }
        let peripheral = self.adapter.peripheral(&id).await?;
        if !peripheral
            .properties()
            .await
            .ok()
            .and_then(|props| props.and_then(|inner| inner.local_name))
            .map_or(false, |name| name == "Flower care")
        {
            return Ok(());
        }
        peripheral.connect().await?;
        Ok(())
    }

    async fn handle_connected<S: Sender + Send>(
        &mut self,
        id: PeripheralId,
        sender: &S,
    ) -> anyhow::Result<()> {
        let peripheral = self.adapter.peripheral(&id).await?;
        let device = MiFloraDevice::new(peripheral).await?;

        let now = current_timestamp();
        let tags = MetricTags::default()
            .with_tag("device", DEVICE)
            .with_tag("address", device.address())
            .maybe_with_tag("name", device.name());

        let battery_level = device.read_battery().await?;
        let _ = sender
            .push(Metric {
                header: MetricHeader::new("device.battery", tags.clone()),
                timestamp: now,
                value: MetricValue::gauge(battery_level as f64),
            })
            .await;

        let realtime = device.read_realtime_data().await?;

        let _ = sender
            .push(Metric {
                header: MetricHeader::new("measurement.temperature", tags.clone()),
                timestamp: now,
                value: MetricValue::gauge(realtime.temperature),
            })
            .await;
        let _ = sender
            .push(Metric {
                header: MetricHeader::new("measurement.moisture", tags.clone()),
                timestamp: now,
                value: MetricValue::gauge(realtime.moisture as f64),
            })
            .await;
        let _ = sender
            .push(Metric {
                header: MetricHeader::new("measurement.light", tags.clone()),
                timestamp: now,
                value: MetricValue::gauge(realtime.light as f64),
            })
            .await;
        let _ = sender
            .push(Metric {
                header: MetricHeader::new("measurement.conductivity", tags.clone()),
                timestamp: now,
                value: MetricValue::gauge(realtime.conductivity as f64),
            })
            .await;

        device.into_peripheral().disconnect().await.unwrap();

        self.cache.push(id, SystemTime::now());

        Ok(())
    }

    async fn handle_event<S: Sender + Send>(
        &mut self,
        event: CentralEvent,
        sender: &S,
    ) -> anyhow::Result<()> {
        match event {
            CentralEvent::DeviceDiscovered(id) => self.handle_discovered(id).await,
            CentralEvent::DeviceConnected(id) => self.handle_connected(id, sender).await,
            _ => Ok(()),
        }
    }
}

impl myhomelab_agent_prelude::reader::Reader for Reader {
    #[tracing::instrument(skip_all)]
    async fn run<S: Sender + Send>(
        mut self,
        token: CancellationToken,
        sender: S,
    ) -> anyhow::Result<()> {
        println!("preparing reader");
        let mut events = self.adapter.events().await?;
        println!("starting reader");
        self.adapter.start_scan(self.scan_filter.clone()).await?;
        while !token.is_cancelled() {
            tokio::select! {
                Some(event) = events.next() => {
                    self.handle_event(event, &sender).await?;
                }
                _ = token.cancelled() => {
                    self.adapter.stop_scan().await?;
                }
            }
        }
        tracing::info!("stopped reader");
        Ok(())
    }
}
