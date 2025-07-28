use std::collections::BTreeMap;

use myhomelab_sensor_prelude::collector::Collector;
use myhomelab_sensor_prelude::manager::ManagerBuilder;
use myhomelab_sensor_prelude::sensor::{BuildContext, Sensor, SensorBuilder};

use crate::sensor::AnySensor;

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
    system: ConfigWrapper<myhomelab_sensor_system::SystemSensorConfig>,
    #[serde(default)]
    xiaomi_lywsd03mmc_atc: ConfigWrapper<myhomelab_sensor_xiaomi_lywsd03mmc_atc::SensorConfig>,
    #[serde(default)]
    xiaomi_miflora: ConfigWrapper<myhomelab_sensor_xiaomi_miflora::MifloraSensorConfig>,
}

impl ManagerBuilder for ManagerConfig {
    type Output = crate::Manager;

    async fn build<C: Collector>(&self, ctx: &BuildContext<C>) -> anyhow::Result<Self::Output> {
        let mut inner = BTreeMap::new();
        if let Some(sensor) = self.system.build(ctx).await? {
            inner.insert(sensor.name(), AnySensor::System(sensor));
        }
        if let Some(sensor) = self.xiaomi_lywsd03mmc_atc.build(ctx).await? {
            inner.insert(sensor.name(), AnySensor::XiaomiLywsd03mmcAtc(sensor));
        }
        if let Some(sensor) = self.xiaomi_miflora.build(ctx).await? {
            inner.insert(sensor.name(), AnySensor::XiaomiMiflora(sensor));
        }
        Ok(crate::Manager { inner })
    }
}
