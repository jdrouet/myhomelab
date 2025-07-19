use myhomelab_metric::entity::Metric;

pub trait Collector: Clone + Send + Sync + 'static {
    fn push_metrics<V>(&self, metrics: V) -> impl Future<Output = anyhow::Result<()>> + Send
    where
        V: IntoIterator<Item = Metric> + Send;
}

#[derive(Clone)]
pub struct TracingCollector;

impl Collector for TracingCollector {
    async fn push_metrics<V>(&self, metrics: V) -> anyhow::Result<()>
    where
        V: IntoIterator<Item = Metric> + Send,
    {
        metrics.into_iter().for_each(|metric| {
            tracing::debug!("received {metric}");
        });
        Ok(())
    }
}
