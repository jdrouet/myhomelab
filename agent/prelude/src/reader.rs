use crate::mpsc::Sender;

pub trait Reader: Send + Sync + 'static {
    fn run<S: Sender + Send>(self, sender: S) -> impl Future<Output = anyhow::Result<()>> + Send;
}
