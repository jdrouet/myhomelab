use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::collector::Collector;

pub trait Sensor: std::fmt::Debug + Send + Sync + 'static {
    type Cmd: Send + Sync;

    fn execute(&self, command: Self::Cmd) -> impl Future<Output = anyhow::Result<()>> + Send;
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
    type Cmd = ();

    async fn execute(&self, _command: Self::Cmd) -> anyhow::Result<()> {
        tracing::debug!("this sensor doesn't execute commands");
        Ok(())
    }

    async fn wait(self) -> anyhow::Result<()> {
        self.task.await?
    }
}
