use std::borrow::Cow;

use anyhow::Context;
use myhomelab_metric::entity::value::{GaugeValue, MetricValue};
use myhomelab_metric::entity::{Metric, MetricTags};
use myhomelab_prelude::time::current_timestamp;
use myhomelab_sensor_prelude::collector::Collector;
use myhomelab_sensor_prelude::sensor::{
    BasicTaskSensor, BuildContext, SensorBuilder, SensorDescriptor,
};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

const SERVICE_ID: uuid::Uuid = uuid::Uuid::from_u128(488837762788578050050668711589115);

struct SensorRunner<C: Collector> {
    adapter: bluer::Adapter,
    cancel: CancellationToken,
    collector: C,
}

impl<C: Collector> SensorRunner<C> {
    async fn emit_metrics(&self, name: String, address: Address, data: Vec<u8>) {
        let timestamp = current_timestamp();
        let tags = MetricTags::default()
            .with_tag("name", name)
            .with_tag("address", address.to_string());
        let mut metrics = Vec::with_capacity(3);
        if let Some(value) = crate::parser::read_temperature(&data) {
            metrics.push(Metric {
                name: Cow::Borrowed("measurement.temperature"),
                tags: Cow::Borrowed(&tags),
                timestamp,
                value: MetricValue::Gauge(GaugeValue::from(value as f64)),
            });
        }
        if let Some(value) = crate::parser::read_humidity(&data) {
            metrics.push(Metric {
                name: Cow::Borrowed("measurement.humidity"),
                tags: Cow::Borrowed(&tags),
                timestamp,
                value: MetricValue::Gauge(GaugeValue::from(value as f64)),
            });
        }
        if let Some(value) = crate::parser::read_battery(&data) {
            metrics.push(Metric {
                name: Cow::Borrowed("device.battery"),
                tags: Cow::Borrowed(&tags),
                timestamp,
                value: MetricValue::Gauge(GaugeValue::from(value as f64)),
            });
        }
        if let Err(err) = self.collector.push_metrics(&metrics).await {
            tracing::warn!(message = "unable to send metrics", error = ?err);
        }
    }

    #[tracing::instrument(parent = None, target = RUNNER_NAMESPACE, skip(self), err(Debug))]
    async fn handle_discovered(&mut self, event: AdapterEvent) -> anyhow::Result<()> {
        let AdapterEvent::DeviceAdded(address) = event else {
            return Ok(());
        };
        let device = self.adapter.device(address).context("getting device")?;
        if let Some(mut service_data) = device.service_data().await? {
            if let Some(data) = service_data.remove(&SERVICE_ID) {
                let name = device.name().await?.unwrap_or_default();
                tracing::debug!("discovered new device {name} ({address})");

                self.emit_metrics(name, address, data).await;
            }
        }
        Ok(())
    }

    #[tracing::instrument(target = RUNNER_NAMESPACE, skip_all, err(Debug))]
    async fn scan(&mut self) -> anyhow::Result<()> {
        tracing::info!("starting reader");
        self.adapter
            .set_discovery_filter(Default::default())
            .await?;
        tracing::info!("preparing reader");
        let mut events = self.adapter.discover_devices_with_changes().await?;
        while !self.cancel.is_cancelled() {
            tokio::select! {
                Some(event) = events.next() => {
                    if let Err(err) = self.handle_discovered(event).await {
                        tracing::warn!(message = "unable to handle device", error = ?err);
                    }
                }
                _ = self.cancel.cancelled() => {
                    // nothing to do
                }
            }
        }
        tracing::info!("stopped reader");
        Ok(())
    }

    #[tracing::instrument(target = RUNNER_NAMESPACE, skip_all, err(Debug))]
    async fn run(mut self) -> anyhow::Result<()> {
        tracing::info!("starting");
        self.adapter.set_powered(true).await?;
        while !self.cancel.is_cancelled() {
            self.scan().await?;
        }
        tracing::info!("completed");
        Ok(())
    }
}
