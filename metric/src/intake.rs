use myhomelab_prelude::Healthcheck;

use crate::entity::{MetricRef, value::MetricValue};

pub trait Intake: Healthcheck {
    fn ingest<'h>(
        &self,
        container: &[MetricRef<'h, MetricValue>],
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
