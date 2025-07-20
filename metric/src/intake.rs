use myhomelab_prelude::Healthcheck;

use crate::entity::Metric;

pub trait Intake: Healthcheck {
    fn ingest(&self, container: &[Metric]) -> impl Future<Output = anyhow::Result<()>> + Send;
}
