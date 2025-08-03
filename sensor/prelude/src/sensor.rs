use myhomelab_prelude::Healthcheck;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::collector::Collector;

pub trait Sensor: std::fmt::Debug + Healthcheck + Send + Sync + 'static {
    type Cmd: Send + Sync;

    fn descriptor(&self) -> SensorDescriptor;
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

#[derive(Clone, Copy, Debug)]
pub struct SensorDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug)]
pub struct BuildContext<C: Collector> {
    pub cancel: CancellationToken,
    pub collector: C,
}

#[derive(Debug)]
pub struct BasicTaskSensor {
    descriptor: SensorDescriptor,
    task: JoinHandle<anyhow::Result<()>>,
}

impl BasicTaskSensor {
    pub fn new(descriptor: SensorDescriptor, task: JoinHandle<anyhow::Result<()>>) -> Self {
        Self { descriptor, task }
    }
}

impl Healthcheck for BasicTaskSensor {
    async fn healthcheck(&self) -> anyhow::Result<()> {
        if self.task.is_finished() {
            Err(anyhow::anyhow!("sensor task is dead"))
        } else {
            tracing::debug!("task is still running");
            Ok(())
        }
    }
}

impl Sensor for BasicTaskSensor {
    type Cmd = ();

    async fn execute(&self, _command: Self::Cmd) -> anyhow::Result<()> {
        tracing::debug!("this sensor doesn't execute commands");
        Ok(())
    }

    fn descriptor(&self) -> SensorDescriptor {
        self.descriptor
    }

    async fn wait(self) -> anyhow::Result<()> {
        self.task.await?
    }
}
