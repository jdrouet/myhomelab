use std::num::NonZeroUsize;

use btleplug::{
    api::{Central, CentralEvent, Manager, Peripheral, ScanFilter, WriteType},
    platform::PeripheralId,
};
use lru::LruCache;
use myhomelab_agent_prelude::mpsc::Sender;
use myhomelab_metric::entity::{Metric, MetricHeader, MetricTags, value::MetricValue};
use myhomelab_prelude::time::current_timestamp;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

// Device name (e.g. Flower care)
// MAC address prefix (C4:7C:8D = original)

const DEVICE: &str = "xiaomi-miflora";
const SERVICE_ID: uuid::Uuid = uuid::uuid!("00001204-0000-1000-8000-00805f9b34fb");
const DATA_UUID: uuid::Uuid = uuid::uuid!("00001a01-0000-1000-8000-00805f9b34fb");
const READ_CMD_UUID: uuid::Uuid = uuid::uuid!("00001a00-0000-1000-8000-00805f9b34fb");
const BATTERY_UUID: uuid::Uuid = uuid::uuid!("00001a02-0000-1000-8000-00805f9b34fb");
const TIMEOUT: u64 = 60 * 60 * 2; // 2h

#[derive(Debug, Default)]
pub struct ReaderConfig {}

impl myhomelab_prelude::FromEnv for ReaderConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl ReaderConfig {
    pub async fn build(&self) -> anyhow::Result<Reader> {
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

        Ok(Reader {
            cache: LruCache::new(NonZeroUsize::new(10).unwrap()),
            manager,
            adapter,
            scan_filter,
        })
    }
}

#[derive(Debug)]
pub struct Reader {
    cache: LruCache<PeripheralId, u64>,
    #[allow(unused)]
    manager: btleplug::platform::Manager,
    adapter: btleplug::platform::Adapter,
    scan_filter: ScanFilter,
}

impl Reader {
    async fn handle_discovered(&mut self, id: PeripheralId) -> anyhow::Result<()> {
        let now = current_timestamp();
        if self
            .cache
            .get(&id)
            .map_or(false, |last_seen| last_seen + TIMEOUT > now)
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
        let address = peripheral.address();
        let services = peripheral.services();
        if services.iter().find(|s| s.uuid == SERVICE_ID).is_none() {
            peripheral.disconnect().await.ok();
            return Ok(());
        };

        let name = peripheral
            .properties()
            .await
            .ok()
            .and_then(|props| props)
            .and_then(|props| props.local_name);
        let address = address.to_string();
        let now = current_timestamp();
        let tags = MetricTags::default()
            .with_tag("device", DEVICE)
            .with_tag("address", address)
            .maybe_with_tag("name", name);

        // Find the battery characteristic
        if let Some(battery_char) = peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == BATTERY_UUID)
        {
            // Read battery level
            if let Some(battery_data) = peripheral
                .read(&battery_char)
                .await
                .ok()
                .filter(|data| data.len() >= 2)
            {
                let battery_level = battery_data[0] as f64;
                let _ = sender
                    .push(Metric {
                        header: MetricHeader::new("device.battery", tags.clone()),
                        timestamp: now,
                        value: MetricValue::gauge(battery_level),
                    })
                    .await;
            }
        }

        let Some(read_cmd_char) = peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == READ_CMD_UUID)
        else {
            peripheral.disconnect().await.ok();
            return Ok(());
        };
        peripheral
            .write(&read_cmd_char, &[0xA0, 0x1F], WriteType::WithResponse)
            .await?;

        let Some(data_char) = peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == DATA_UUID)
        else {
            peripheral.disconnect().await.ok();
            return Ok(());
        };

        if let Some(data) = peripheral.read(&data_char).await.ok() {
            // Data decoding (based on known MiFlora protocol)
            let temperature = u16::from_le_bytes([data[0], data[1]]) as f64 / 10.0;
            let moisture = data[7];
            let light = u32::from_le_bytes([data[3], data[4], data[5], 0x00]); // 24-bit value
            let conductivity = u16::from_le_bytes([data[8], data[9]]);

            let _ = sender
                .push(Metric {
                    header: MetricHeader::new("measurement.temperature", tags.clone()),
                    timestamp: now,
                    value: MetricValue::gauge(temperature),
                })
                .await;
            let _ = sender
                .push(Metric {
                    header: MetricHeader::new("measurement.moisture", tags.clone()),
                    timestamp: now,
                    value: MetricValue::gauge(moisture as f64),
                })
                .await;
            let _ = sender
                .push(Metric {
                    header: MetricHeader::new("measurement.light", tags.clone()),
                    timestamp: now,
                    value: MetricValue::gauge(light as f64),
                })
                .await;
            let _ = sender
                .push(Metric {
                    header: MetricHeader::new("measurement.conductivity", tags.clone()),
                    timestamp: now,
                    value: MetricValue::gauge(conductivity as f64),
                })
                .await;
        }

        peripheral.disconnect().await.unwrap();

        self.cache.push(id, now);

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
