pub trait Healthcheck {
    fn healthcheck(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
}
