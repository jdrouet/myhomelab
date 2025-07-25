use myhomelab_agent_prelude::collector::Collector;
use myhomelab_agent_prelude::sensor::{BuildContext, SensorBuilder};

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
    system: ConfigWrapper<myhomelab_agent_reader_system::SystemReaderConfig>,
    #[serde(default)]
    xiaomi_lywsd03mmc_atc:
        ConfigWrapper<myhomelab_agent_reader_xiaomi_lywsd03mmc_atc::SensorConfig>,
    #[serde(default)]
    xiaomi_miflora: ConfigWrapper<myhomelab_agent_reader_xiaomi_miflora::MifloraReaderConfig>,
}

impl myhomelab_agent_prelude::sensor::SensorBuilder for ManagerConfig {
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
pub struct Manager {
    system: Option<myhomelab_agent_reader_system::SystemReader>,
    xiaomi_lywsd03mmc_atc: Option<myhomelab_agent_reader_xiaomi_lywsd03mmc_atc::SensorReader>,
    xiaomi_miflora: Option<myhomelab_agent_reader_xiaomi_miflora::MifloraReader>,
}

impl myhomelab_agent_prelude::sensor::Sensor for Manager {
    async fn wait(self) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let mut errors = Vec::default();
        if let Some(sensor) = self.system {
            if let Err(err) = sensor.wait().await {
                errors.push(err);
            }
        }
        if let Some(sensor) = self.xiaomi_lywsd03mmc_atc {
            if let Err(err) = sensor.wait().await {
                errors.push(err);
            }
        }
        if let Some(sensor) = self.xiaomi_miflora {
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
