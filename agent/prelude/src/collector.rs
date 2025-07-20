use myhomelab_metric::entity::MetricRef;

pub trait Collector: Clone + Send + Sync + 'static {
    fn push_metrics<'h>(
        &self,
        metrics: &[MetricRef<'h>],
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[derive(Clone)]
pub struct TracingCollector;

impl Collector for TracingCollector {
    async fn push_metrics<'h>(&self, metrics: &[MetricRef<'h>]) -> anyhow::Result<()> {
        metrics.into_iter().for_each(|metric| {
            tracing::debug!("received {metric}");
        });
        Ok(())
    }
}
