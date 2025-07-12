use myhomelab_metric::query::QueryExecutor;

#[derive(Debug, Default)]
pub struct HomePage {}

impl crate::prelude::Page for HomePage {
    fn title(&self) -> &str {
        "Home"
    }

    fn render_body<Q: QueryExecutor + Send>(
        &self,
        _context: &crate::prelude::Context<Q>,
        _buf: &mut String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        async { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use myhomelab_metric::mock::MockMetric;

    use crate::{page::PageWrapper, prelude::Context};

    use super::HomePage;

    #[tokio::test]
    async fn should_render_page() {
        let home = HomePage::default();
        let query_executor = MockMetric::new();
        let context = Context::new(query_executor);
        let mut buffer = String::with_capacity(1024);
        PageWrapper::new(home)
            .render(&context, &mut buffer)
            .await
            .unwrap();
        assert!(buffer.contains("<title>Home</title>"));
    }
}
