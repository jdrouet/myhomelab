use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::mpsc::Sender;

pub trait Reader: std::fmt::Debug + Send + Sync + 'static {
    fn wait(self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait ReaderBuilder {
    type Output: Reader;

    fn build<S: Sender>(
        &self,
        ctx: &BuildContext<S>,
    ) -> impl Future<Output = anyhow::Result<Self::Output>> + Send;
}

#[derive(Debug)]
pub struct BuildContext<S: Sender> {
    pub cancel: CancellationToken,
    pub sender: S,
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
