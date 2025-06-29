use tokio_util::sync::CancellationToken;

use crate::mpsc::Sender;

pub trait Reader: Send + Sync + 'static {
    fn run<S: Sender + Send>(
        self,
        token: CancellationToken,
        sender: S,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
