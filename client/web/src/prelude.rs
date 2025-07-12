pub trait Context: Send + Sync {
    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor;
}

pub trait Page: Send + Sync {
    fn title(&self) -> &str;
    fn render_body<C: Context>(
        &self,
        context: &C,
        buf: &mut String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait Component: Send + Sync {
    fn render<C: Context>(
        &self,
        context: &C,
        buf: &mut String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
