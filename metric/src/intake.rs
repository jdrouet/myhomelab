use myhomelab_prelude::Healthcheck;

use crate::entity::Metric;
use crate::entity::value::MetricValue;

pub trait Intake: Healthcheck {
    fn ingest<'h>(
        &self,
        container: &[Metric<'h, MetricValue>],
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
