use std::time::Duration;

use myhomelab_sensor_prelude::collector::TracingCollector;
use myhomelab_sensor_prelude::sensor::{BuildContext, Sensor, SensorBuilder};
use myhomelab_sensor_xiaomi_miflora::{MifloraCommand, MifloraSensorConfig};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = MifloraSensorConfig::default();
    let cancel = CancellationToken::new();
    let build_ctx = BuildContext {
        cancel: cancel.child_token(),
        collector: TracingCollector,
    };
    let sensor = config.build(&build_ctx).await?;
    tokio::time::sleep(Duration::new(30, 0)).await;
    if let Err(err) = sensor.execute(MifloraCommand::SynchronizeAll).await {
        tracing::error!(message = "unable to execute command", cause = ?err);
    }
    tokio::time::sleep(Duration::new(60, 0)).await;
    cancel.cancel();
    sensor.wait().await
}
