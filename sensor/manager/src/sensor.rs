use anyhow::Context;
use myhomelab_prelude::Healthcheck;
use myhomelab_sensor_prelude::sensor::Sensor;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "sensor")]
pub enum AnyCommand {
    #[serde(rename = "xiaomi.miflora")]
    XiaomiMiflora(myhomelab_sensor_xiaomi_miflora::MifloraCommand),
}

#[derive(Debug)]
pub enum AnySensor {
    System(myhomelab_sensor_system::SystemSensor),
    XiaomiLywsd03mmcAtc(myhomelab_sensor_xiaomi_lywsd03mmc_atc::AtcSensor),
    XiaomiMiflora(myhomelab_sensor_xiaomi_miflora::MifloraSensor),
}

impl Sensor for AnySensor {
    type Cmd = AnyCommand;

    #[tracing::instrument(skip_all, err)]
    async fn execute(&self, command: Self::Cmd) -> anyhow::Result<()> {
        match (self, command) {
            (Self::XiaomiMiflora(sensor), AnyCommand::XiaomiMiflora(cmd)) => sensor
                .execute(cmd)
                .await
                .context("executing command on xiaomi miflora"),
            _ => Err(anyhow::anyhow!("incompatible command")),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::System(inner) => inner.name(),
            Self::XiaomiLywsd03mmcAtc(inner) => inner.name(),
            Self::XiaomiMiflora(inner) => inner.name(),
        }
    }

    #[tracing::instrument(skip_all, err)]
    async fn wait(self) -> anyhow::Result<()> {
        match self {
            Self::System(inner) => inner.wait().await,
            Self::XiaomiLywsd03mmcAtc(inner) => inner.wait().await,
            Self::XiaomiMiflora(inner) => inner.wait().await,
        }
    }
}

impl Healthcheck for AnySensor {
    #[tracing::instrument(skip_all, err)]
    async fn healthcheck(&self) -> anyhow::Result<()> {
        match self {
            Self::System(inner) => inner.healthcheck().await.context("system sensor failed"),
            Self::XiaomiLywsd03mmcAtc(inner) => inner
                .healthcheck()
                .await
                .context("xiaomy lywsd03mmc-atc sensor failed"),
            Self::XiaomiMiflora(inner) => inner
                .healthcheck()
                .await
                .context("xiaomi miflora sensor failed"),
        }
    }
}
