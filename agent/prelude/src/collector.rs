use myhomelab_event::intake::IntakeInput;
use myhomelab_metric::entity::Metric;

pub trait Collector: Clone + Send + Sync + 'static {
    fn push_metrics<'h>(
        &self,
        metrics: &[Metric<'h>],
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn push_event<I>(&self, input: &I) -> impl Future<Output = anyhow::Result<()>>
    where
        I: IntakeInput;
}

#[derive(Clone)]
pub struct TracingCollector;

impl Collector for TracingCollector {
    async fn push_metrics<'h>(&self, metrics: &[Metric<'h>]) -> anyhow::Result<()> {
        metrics.iter().for_each(|metric| {
            tracing::debug!(
                message = "received metric",
                metric = %metric,
            );
        });
        Ok(())
    }

    async fn push_event<I>(&self, input: &I) -> anyhow::Result<()>
    where
        I: IntakeInput,
    {
        tracing::debug!(
            message = "received event",
            event_source = ?input.source(),
            event_level = ?input.level(),
            event_message = input.message(),
            event_attributes = ?input.attributes(),
        );
        Ok(())
    }
}
