use myhomelab_metric::query::QueryExecutor;

#[derive(Debug)]
pub struct Context<Q: QueryExecutor + Send> {
    query_executory: Q,
}

impl<Q: QueryExecutor + Send> Context<Q> {
    pub fn new(query_executory: Q) -> Self {
        Self { query_executory }
    }

    pub fn query_executor(&self) -> &Q {
        &self.query_executory
    }
}

pub trait Page: Send + Sync {
    fn title(&self) -> &str;
    fn render_body<Q: QueryExecutor + Send>(
        &self,
        context: &Context<Q>,
        buf: &mut String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait Component: Send + Sync {
    fn render<Q: QueryExecutor + Send>(
        &self,
        context: &Context<Q>,
        buf: &mut String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
