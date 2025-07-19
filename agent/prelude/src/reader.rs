use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::collector::Collector;

pub trait Reader: std::fmt::Debug + Send + Sync + 'static {
    fn wait(self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait ReaderBuilder {
    type Output: Reader;

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
pub struct BasicTaskReader {
    task: JoinHandle<anyhow::Result<()>>,
}

impl BasicTaskReader {
    pub fn new(task: JoinHandle<anyhow::Result<()>>) -> Self {
        Self { task }
    }
}

impl Reader for BasicTaskReader {
    async fn wait(self) -> anyhow::Result<()> {
        self.task.await?
    }
}
