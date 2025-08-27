use opentelemetry::{
    KeyValue,
    metrics::{Counter, Gauge, Meter},
};
use uuid::Uuid;

use crate::bluetooth::DiscoveredDevice;

const SERVICE_ID: Uuid = Uuid::from_u128(0x120400001000800000805f9b34fb);

#[derive(Debug)]
struct Sender {
    inner: tokio::sync::mpsc::Sender<DiscoveredDevice>,
    attributes: [KeyValue; 4],
    sent: Counter<u64>,
    error: Counter<u64>,
}

impl Sender {
    #[inline]
    fn new(meter: &Meter, inner: tokio::sync::mpsc::Sender<DiscoveredDevice>) -> Self {
        Self {
            inner,
            attributes: [
                KeyValue::new("messaging.client.id", "xiaomi-miflora-collector"),
                KeyValue::new("messaging.consumer.group.name", "xiaomi-miflora"),
                KeyValue::new("messaging.destination.name", "xiaomi-miflora-runner"),
                KeyValue::new("messaging.system", "mpsc"),
            ],
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
                error.type = "send-error",
                error.message = err.to_string(),
                message = "unable to send discovered device",
                messaging.batch.message_count = 1,
                messaging.client.id = "xiaomi-miflora-collector",
                messaging.consumer.group.name = "xiaomi-miflora",
                messaging.destination.name = "xiaomi-miflora-runner",
                messaging.operation.name = "send",
                messaging.operation.type = "send",
                messaging.system = "mpsc",
            );
        }
    }
}

#[derive(Debug)]
struct Receiver {
    inner: tokio::sync::mpsc::Receiver<DiscoveredDevice>,
    attributes: [KeyValue; 4],
    received: Counter<u64>,
}

impl Receiver {
    #[inline]
    fn new(meter: &Meter, inner: tokio::sync::mpsc::Receiver<DiscoveredDevice>) -> Self {
        Self {
            inner,
            attributes: [
                KeyValue::new("messaging.client.id", "xiaomi-miflora-collector"),
                KeyValue::new("messaging.consumer.group.name", "xiaomi-miflora"),
                KeyValue::new("messaging.destination.name", "xiaomi-miflora-runner"),
                KeyValue::new("messaging.system", "mpsc"),
            ],
            received: meter
                .u64_counter("queue.events.received")
                .with_description("Number of events received from the queue")
                .build(),
        }
    }

    async fn recv(&mut self) -> Option<DiscoveredDevice> {
        let event = self.inner.recv().await?;
        self.received.add(1, &self.attributes);
        Some(event)
    }
}

#[derive(Debug)]
struct XiaomiMifloraRunner {
    receiver: Receiver,
    #[allow(unused)]
    temperature: Gauge<f64>,
    #[allow(unused)]
    battery: Gauge<f64>,
}

impl XiaomiMifloraRunner {
    async fn run(mut self) -> anyhow::Result<()> {
        while let Some(_value) = self.receiver.recv().await {
            // TODO
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
            receiver: Receiver::new(&meter, receiver),
            temperature: meter
                .f64_gauge("measurement.temperature")
                .with_unit("degree celcius")
                .build(),
            battery: meter
                .f64_gauge("system.battery")
                .with_unit("percentage")
                .build(),
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
        device: super::DiscoveredDevice,
    ) -> anyhow::Result<Option<super::DiscoveredDevice>> {
        let Some(_) = device.service_data.get(&SERVICE_ID) else {
            return Ok(Some(device));
        };

        self.sender.send(device).await;

        Ok(None)
    }
}
