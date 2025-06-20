use crate::metric::Metric;

pub trait Intake {
    fn ingest(&self, values: &[Metric]) -> impl Future<Output = anyhow::Result<()>> + Send;
}
