use myhomelab_metric::entity::Metric;

pub trait Collector: Clone + Send + Sync + 'static {
    fn push_metrics(&self, metrics: &[Metric]) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[derive(Clone)]
pub struct TracingCollector;

impl Collector for TracingCollector {
    async fn push_metrics(&self, metrics: &[Metric]) -> anyhow::Result<()> {
        metrics.into_iter().for_each(|metric| {
            tracing::debug!("received {metric}");
        });
        Ok(())
    }
}
