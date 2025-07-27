use std::collections::BTreeMap;

use myhomelab_agent_prelude::sensor::Sensor;
use myhomelab_prelude::Healthcheck;
use sensor::AnySensor;

pub mod config;
pub mod sensor;

#[derive(Debug)]
pub struct Manager {
    inner: BTreeMap<&'static str, sensor::AnySensor>,
}

impl Healthcheck for Manager {
    async fn healthcheck(&self) -> anyhow::Result<()> {
        for sensor in self.inner.values() {
            sensor.healthcheck().await?;
        }
        Ok(())
    }
}

impl myhomelab_agent_prelude::manager::Manager for Manager {
    type Sensor = AnySensor;

    fn get_sensor(&self, name: &str) -> Option<&Self::Sensor> {
        self.inner.get(name)
    }

    fn sensors(&self) -> impl Iterator<Item = &Self::Sensor> {
        self.inner.values()
    }

    async fn wait(self) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let mut errors = Vec::default();
        for sensor in self.inner.into_values() {
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
