use std::{
    borrow::Cow,
    collections::HashMap,
    time::{Duration, SystemTime},
};

use bluer::gatt::remote::CharacteristicWriteRequest;
use opentelemetry::{
    KeyValue,
    metrics::{Counter, Gauge, Meter},
};
use uuid::Uuid;

const SERVICE_ID: Uuid = Uuid::from_u128(0x0000fe95_0000_1000_8000_00805f9b34fb);
const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60 * 2); // 2h

type DiscoveredDevice = (bluer::Device, Vec<KeyValue>);

#[derive(Debug)]
struct Sender {
    inner: tokio::sync::mpsc::Sender<DiscoveredDevice>,
    attributes: [KeyValue; 1],
    sent: Counter<u64>,
    error: Counter<u64>,
}

impl Sender {
    #[inline]
    fn new(meter: &Meter, inner: tokio::sync::mpsc::Sender<DiscoveredDevice>) -> Self {
        Self {
            inner,
            attributes: [KeyValue::new("topic", "xiaomi-miflora")],
            sent: meter
                .u64_counter("queue.events.sent")
                .with_description("Number of events sent in the queue")
                .build(),
            error: meter
                .u64_counter("queue.events.sent.error")
                .with_description("Number of events that failed being sent in the queue")
                .build(),
        }
    }

    async fn send(&self, event: DiscoveredDevice) {
        self.sent.add(1, &self.attributes);
        if let Err(err) = self.inner.send(event).await {
            self.error.add(1, &self.attributes);
            tracing::error!(
                message = "unable to send discovered device",
                error.type = "send-error",
                error.message = err.to_string(),
            );
        }
    }
}

#[derive(Debug)]
struct Receiver {
    inner: tokio::sync::mpsc::Receiver<DiscoveredDevice>,
    attributes: [KeyValue; 1],
    received: Counter<u64>,
}

impl Receiver {
    #[inline]
    fn new(meter: &Meter, inner: tokio::sync::mpsc::Receiver<DiscoveredDevice>) -> Self {
        Self {
            inner,
            attributes: [KeyValue::new("topic", "xiaomi-miflora")],
            received: meter
                .u64_counter("queue.events.received")
                .with_description("Number of events received from the queue")
                .build(),
        }
    }

    async fn recv(&mut self) -> Option<(bluer::Device, Vec<KeyValue>)> {
        let event = self.inner.recv().await?;
        self.received.add(1, &self.attributes);
        Some(event)
    }
}

#[derive(Debug)]
struct XiaomiMifloraRunner {
    last_check: HashMap<bluer::Address, SystemTime>,
    receiver: Receiver,
    temperature: Gauge<f64>,
    brightness: Gauge<f64>,
    moisture: Gauge<f64>,
    conductivity: Gauge<f64>,
    battery: Gauge<f64>,
}

impl XiaomiMifloraRunner {
    #[tracing::instrument(
        parent = None,
        skip_all,
        fields(
            ble.address = %device.address(),
            ble.firmware = tracing::field::Empty,
            otel.status_code = tracing::field::Empty,
            resource.name = "miflora-collector/handle_event",
        )
        err(Debug),
    )]
    async fn handle_device(
        &mut self,
        device: bluer::Device,
        attributes: Vec<KeyValue>,
    ) -> anyhow::Result<()> {
        let span = tracing::Span::current();
        let address = device.address();
        if let Some(last) = self.last_check.get(&address) {
            if *last + CHECK_INTERVAL > SystemTime::now() {
                tracing::trace!(
                    message = "device checked recently, skipping",
                    address = %address,
                    last = ?last,
                    interval = ?CHECK_INTERVAL,
                );
                span.record("otel.status_code", "OK");
                return Ok(());
            }
        }

        device.connect().await?;

        let device = MifloraDevice::new(device).await?;

        let system = device.read_system().await?;
        span.record("ble.firmware", system.firmware().as_ref());

        let realtime = device.read_realtime_values().await?;

        self.temperature.record(realtime.temperature(), &attributes);
        self.brightness.record(realtime.brightness(), &attributes);
        self.moisture.record(realtime.moisture(), &attributes);
        self.conductivity
            .record(realtime.conductivity(), &attributes);
        self.battery.record(system.battery(), &attributes);

        self.last_check.insert(address, SystemTime::now());
        span.record("otel.status_code", "OK");
        Ok(())
    }

    async fn run(mut self) -> anyhow::Result<()> {
        while let Some((device, attributes)) = self.receiver.recv().await {
            if let Err(err) = self.handle_device(device, attributes).await {
                tracing::error!(
                    message = "unable to handle device",
                    exception.message = err.to_string(),
                    exception.stacktrace = format!("{err:?}"),
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct XiaomiMifloraCollector {
    sender: Sender,
    #[allow(unused)]
    task: tokio::task::JoinHandle<anyhow::Result<()>>,
}

impl Default for XiaomiMifloraCollector {
    fn default() -> Self {
        let meter = opentelemetry::global::meter("xiaomi-miflora");

        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        let runner = XiaomiMifloraRunner {
            last_check: Default::default(),
            receiver: Receiver::new(&meter, receiver),
            temperature: meter
                .f64_gauge("measurement.temperature")
                .with_unit("degree celcius")
                .build(),
            brightness: meter
                .f64_gauge("measurement.brightness")
                .with_unit("lux")
                .build(),
            moisture: meter
                .f64_gauge("measurement.moisture")
                .with_unit("%")
                .build(),
            conductivity: meter
                .f64_gauge("measurement.conductivity")
                .with_unit("µS/cm")
                .build(),
            battery: meter.f64_gauge("system.battery").with_unit("%").build(),
        };
        let task = tokio::spawn(runner.run());

        Self {
            sender: Sender::new(&meter, sender),
            task,
        }
    }
}

impl XiaomiMifloraCollector {
    pub async fn collect(
        &self,
        device: &bluer::Device,
        attributes: &[opentelemetry::KeyValue],
    ) -> anyhow::Result<bool> {
        let uuids = device.uuids().await?;
        if !uuids.map_or(false, |set| set.contains(&SERVICE_ID)) {
            return Ok(false);
        };

        self.sender
            .send((device.clone(), attributes.to_vec()))
            .await;

        Ok(true)
    }
}

struct MifloraSystem {
    inner: Vec<u8>,
}

impl MifloraSystem {
    fn battery(&self) -> f64 {
        self.inner[0] as f64
    }

    fn firmware(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.inner[2..])
    }
}

struct MifloraRealtimeEntry {
    inner: Vec<u8>,
}

impl MifloraRealtimeEntry {
    fn temperature(&self) -> f64 {
        (u16::from_le_bytes([self.inner[0], self.inner[1]]) as f64) * 0.1
    }

    fn brightness(&self) -> f64 {
        u32::from_le_bytes([self.inner[3], self.inner[4], self.inner[5], self.inner[6]]) as f64
    }

    fn moisture(&self) -> f64 {
        self.inner[7] as f64
    }

    fn conductivity(&self) -> f64 {
        u16::from_le_bytes([self.inner[8], self.inner[9]]) as f64
    }
}

// const CMD_REALTIME_DISABLE: [u8; 2] = [0xc0, 0x1f];
const CMD_REALTIME_ENABLE: [u8; 2] = [0xa0, 0x1f];
const WRITE_OPTS: CharacteristicWriteRequest = CharacteristicWriteRequest {
    offset: 0,
    op_type: bluer::gatt::WriteOp::Request,
    prepare_authorize: false,
    _non_exhaustive: (),
};

struct MifloraDevice {
    address: bluer::Address,
    system_characteristic: bluer::gatt::remote::Characteristic,
    mode_characteristic: bluer::gatt::remote::Characteristic,
    data_characteristic: bluer::gatt::remote::Characteristic,
}

impl MifloraDevice {
    async fn new(inner: bluer::Device) -> bluer::Result<Self> {
        let data_servie = inner.service(49).await?;
        let system_characteristic = data_servie.characteristic(0x37).await?;
        let mode_characteristic = data_servie.characteristic(50).await?;
        let data_characteristic = data_servie.characteristic(52).await?;

        Ok(Self {
            address: inner.address(),
            system_characteristic,
            mode_characteristic,
            data_characteristic,
        })
    }

    #[tracing::instrument(
        skip(self),
        fields(
            ble.address = %self.address,
            resource.name = "miflora/set_mode",
            span.kind = "client",
        ),
        err(Debug),
    )]
    async fn set_mode(&self, mode: &[u8]) -> bluer::Result<()> {
        self.mode_characteristic
            .write_ext(mode, &WRITE_OPTS)
            .await?;
        let data = self.mode_characteristic.read().await?;
        if !data.eq(mode) {
            return Err(bluer::Error {
                kind: bluer::ErrorKind::Internal(bluer::InternalErrorKind::InvalidValue),
                message: "invalid mode returned".into(),
            });
        }
        Ok(())
    }

    #[tracing::instrument(
        skip(self),
        fields(
            ble.address = %self.address,
            resource.name = "miflora/read_realtime_values",
            span.kind = "client",
        ),
        err(Debug),
    )]
    async fn read_realtime_values(&self) -> bluer::Result<MifloraRealtimeEntry> {
        self.set_mode(&CMD_REALTIME_ENABLE).await?;

        let inner = self.data_characteristic.read().await?;
        Ok(MifloraRealtimeEntry { inner })
    }

    #[tracing::instrument(
        skip(self),
        fields(
            ble.address = %self.address,
            resource.name = "miflora/read_system",
            span.kind = "client",
        ),
        err(Debug),
    )]
    async fn read_system(&self) -> bluer::Result<MifloraSystem> {
        self.system_characteristic
            .read()
            .await
            .map(|inner| MifloraSystem { inner })
    }
}
