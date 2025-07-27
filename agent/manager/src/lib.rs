use anyhow::Context;
use myhomelab_agent_prelude::collector::Collector;
use myhomelab_agent_prelude::sensor::{BuildContext, Sensor, SensorBuilder};
use myhomelab_prelude::Healthcheck;

#[derive(Debug, Default, serde::Deserialize)]
pub struct ConfigWrapper<T> {
    pub enabled: bool,
    #[serde(default, flatten)]
    pub inner: T,
}

impl<T: SensorBuilder> ConfigWrapper<T> {
    async fn build<C: Collector>(
        &self,
        ctx: &BuildContext<C>,
    ) -> anyhow::Result<Option<T::Output>> {
        if self.enabled {
            self.inner.build(ctx).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct ManagerConfig {
    #[serde(default)]
    system: ConfigWrapper<myhomelab_agent_sensor_system::SystemSensorConfig>,
    #[serde(default)]
    xiaomi_lywsd03mmc_atc:
        ConfigWrapper<myhomelab_agent_sensor_xiaomi_lywsd03mmc_atc::SensorConfig>,
    #[serde(default)]
    xiaomi_miflora: ConfigWrapper<myhomelab_agent_sensor_xiaomi_miflora::MifloraSensorConfig>,
}

impl SensorBuilder for ManagerConfig {
    type Output = Manager;

    async fn build<C: Collector>(&self, ctx: &BuildContext<C>) -> anyhow::Result<Self::Output> {
        Ok(Manager {
            system: self.system.build(ctx).await?,
            xiaomi_lywsd03mmc_atc: self.xiaomi_lywsd03mmc_atc.build(ctx).await?,
            xiaomi_miflora: self.xiaomi_miflora.build(ctx).await?,
        })
    }
}

#[derive(Debug)]
pub enum AnySensor {
    System(myhomelab_agent_sensor_system::SystemSensor),
    XiaomiLywsd03mmcAtc(myhomelab_agent_sensor_xiaomi_lywsd03mmc_atc::AtcSensor),
    XiaomiMiflora(myhomelab_agent_sensor_xiaomi_miflora::MifloraSensor),
}

impl AnySensor {
    async fn wait(self) -> anyhow::Result<()> {
        match self {
            Self::System(inner) => inner.wait().await,
            Self::XiaomiLywsd03mmcAtc(inner) => inner.wait().await,
            Self::XiaomiMiflora(inner) => inner.wait().await,
        }
    }
}

#[derive(Debug)]
pub enum AnySensorRef<'a> {
    System(&'a myhomelab_agent_sensor_system::SystemSensor),
    XiaomiLywsd03mmcAtc(&'a myhomelab_agent_sensor_xiaomi_lywsd03mmc_atc::AtcSensor),
    XiaomiMiflora(&'a myhomelab_agent_sensor_xiaomi_miflora::MifloraSensor),
}

impl<'a> Healthcheck for AnySensorRef<'a> {
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

#[derive(Debug)]
pub struct Manager {
    system: Option<myhomelab_agent_sensor_system::SystemSensor>,
    xiaomi_lywsd03mmc_atc: Option<myhomelab_agent_sensor_xiaomi_lywsd03mmc_atc::AtcSensor>,
    xiaomi_miflora: Option<myhomelab_agent_sensor_xiaomi_miflora::MifloraSensor>,
}

impl Manager {
    pub fn iter(&self) -> impl Iterator<Item = AnySensorRef<'_>> + '_ {
        self.system
            .iter()
            .map(AnySensorRef::System)
            .chain(
                self.xiaomi_lywsd03mmc_atc
                    .iter()
                    .map(AnySensorRef::XiaomiLywsd03mmcAtc),
            )
            .chain(self.xiaomi_miflora.iter().map(AnySensorRef::XiaomiMiflora))
    }
}

impl Manager {
    fn into_iter(self) -> impl Iterator<Item = AnySensor> {
        self.system
            .into_iter()
            .map(AnySensor::System)
            .chain(
                self.xiaomi_lywsd03mmc_atc
                    .into_iter()
                    .map(AnySensor::XiaomiLywsd03mmcAtc),
            )
            .chain(
                self.xiaomi_miflora
                    .into_iter()
                    .map(AnySensor::XiaomiMiflora),
            )
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "sensor")]
pub enum ManagerCommand {
    #[serde(rename = "xiaomi.miflora")]
    XiaomiMiflora(myhomelab_agent_sensor_xiaomi_miflora::MifloraCommand),
}

impl Healthcheck for Manager {
    async fn healthcheck(&self) -> anyhow::Result<()> {
        for sensor in self.iter() {
            sensor.healthcheck().await?;
        }
        Ok(())
    }
}

impl Sensor for Manager {
    type Cmd = ManagerCommand;

    async fn execute(&self, command: Self::Cmd) -> anyhow::Result<()> {
        match command {
            ManagerCommand::XiaomiMiflora(inner) => {
                if let Some(ref sensor) = self.xiaomi_miflora {
                    sensor
                        .execute(inner)
                        .await
                        .context("executing miflora command")?;
                } else {
                    tracing::warn!(message = "miflora sensor disabled", command = ?inner);
                }
            }
        };
        Ok(())
    }

    async fn wait(self) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let mut errors = Vec::default();
        for sensor in self.into_iter() {
            if let Err(err) = sensor.wait().await {
                errors.push(err);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors
                .into_iter()
                .fold(anyhow::anyhow!("some reader failed"), |prev, err| {
                    prev.context(err)
                }))
        }
    }
}
