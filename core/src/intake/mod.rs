pub trait Intake {
    fn ingest(&self, values: &[crate::Metric]) -> impl Future<Output = std::io::Result<()>> + Send;
}
