use anyhow::Context;
use myhomelab_metric::entity::Metric;

pub trait Sender: Clone + Send + Sync + 'static {
    fn push(&self, item: Metric) -> impl Future<Output = anyhow::Result<()>> + Send;
}

impl Sender for tokio::sync::mpsc::Sender<Metric> {
    async fn push(&self, item: Metric) -> anyhow::Result<()> {
        self.send(item).await.context("sending to mpsc queue")
    }
}

impl Sender for tokio::sync::mpsc::UnboundedSender<Metric> {
    async fn push(&self, item: Metric) -> anyhow::Result<()> {
        self.send(item).context("sending to mpsc queue")
    }
}

pub trait Receiver {
    fn pull(&mut self) -> impl Future<Output = Option<Metric>> + Send;
    fn is_empty(&self) -> bool;
}

impl Receiver for tokio::sync::mpsc::Receiver<Metric> {
    async fn pull(&mut self) -> Option<Metric> {
        self.recv().await
    }

    fn is_empty(&self) -> bool {
        tokio::sync::mpsc::Receiver::is_empty(self)
    }
}

impl Receiver for tokio::sync::mpsc::UnboundedReceiver<Metric> {
    async fn pull(&mut self) -> Option<Metric> {
        self.recv().await
    }

    fn is_empty(&self) -> bool {
        tokio::sync::mpsc::UnboundedReceiver::is_empty(self)
    }
}
