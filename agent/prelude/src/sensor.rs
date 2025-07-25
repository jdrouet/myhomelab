use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::collector::Collector;

pub trait Sensor: std::fmt::Debug + Send + Sync + 'static {
    fn wait(self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait SensorBuilder {
    type Output: Sensor;

    fn build<C: Collector>(
        &self,
        ctx: &BuildContext<C>,
    ) -> impl Future<Output = anyhow::Result<Self::Output>> + Send;
}

#[derive(Debug)]
pub struct BuildContext<C: Collector> {
    pub cancel: CancellationToken,
    pub collector: C,
}

#[derive(Debug)]
pub struct BasicTaskSensor {
    task: JoinHandle<anyhow::Result<()>>,
}

impl BasicTaskSensor {
    pub fn new(task: JoinHandle<anyhow::Result<()>>) -> Self {
        Self { task }
    }
}

impl Sensor for BasicTaskSensor {
    async fn wait(self) -> anyhow::Result<()> {
        self.task.await?
    }
}
