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

#[cfg(linux)]
mod runner;
#[cfg(linux)]
mod parser;

const DEVICE: &str = "xiaomi-lywsd03mmc-atc";
const DESCRIPTOR: SensorDescriptor = SensorDescriptor {
    id: DEVICE,
    name: "Xiaomi Lywsd03mmc sensor",
    description: "Bluetooth reader of Xiaomi Lywsd03mmc with ATC firmware",
};

const RUNNER_NAMESPACE: &str = "xiaomi_lywsd03mmc_atc::runner";

#[derive(Debug, serde::Deserialize)]
pub struct SensorConfig {}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {}
    }
}

#[cfg(not(linux))]
impl SensorBuilder for SensorConfig {
    type Output = AtcSensor;

    async fn build<C: Collector>(&self, ctx: &BuildContext<C>) -> anyhow::Result<Self::Output> {
        let cancel = ctx.cancel.child_token();
        let task = tokio::spawn(async move {
            cancel.cancelled().await;
            Ok(())
        });

        Ok(BasicTaskSensor::new(DESCRIPTOR, task))
    }
}

#[cfg(linux)]
impl SensorBuilder for SensorConfig {
    type Output = AtcSensor;

    async fn build<C: Collector>(&self, ctx: &BuildContext<C>) -> anyhow::Result<Self::Output> {
        #[cfg(linux)]
        use bluer::{AdapterEvent, Address};

        let session = bluer::Session::new().await.context("creating session")?;
        let adapter = session
            .default_adapter()
            .await
            .context("getting default adapter")?;

        let runner = SensorRunner {
            adapter,
            cancel: ctx.cancel.child_token(),
            collector: ctx.collector.clone(),
        };
        let task = tokio::spawn(runner.run().instrument(tracing::info_span!("runner")));

        Ok(BasicTaskSensor::new(DESCRIPTOR, task))
    }
}

pub type AtcSensor = BasicTaskSensor;

#[derive(Debug, serde::Serialize)]
pub struct Device {}
