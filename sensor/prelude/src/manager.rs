use crate::collector::Collector;
use crate::sensor::{BuildContext, Sensor};

pub trait ManagerBuilder {
    type Output: Manager;

    fn build<C: Collector>(
        &self,
        ctx: &BuildContext<C>,
    ) -> impl Future<Output = anyhow::Result<Self::Output>> + Send;
}

pub trait Manager: myhomelab_prelude::Healthcheck {
    type Sensor: Sensor;

    fn get_sensor(&self, name: &str) -> Option<&Self::Sensor>;
    fn sensors(&self) -> impl Iterator<Item = &Self::Sensor>;
    fn wait(self) -> impl Future<Output = anyhow::Result<()>> + Send;
}
