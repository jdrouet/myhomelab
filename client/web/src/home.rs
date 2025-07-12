#[derive(Debug, Default)]
pub struct HomePage {}

impl crate::prelude::Page for HomePage {
    fn title(&self) -> &str {
        "Home"
    }

    fn render_body<C: crate::prelude::Context>(
        &self,
        _context: &C,
        buf: &mut String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        buf.push_str("Hello World!");
        async { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use myhomelab_metric::mock::MockMetric;

    use crate::{mock::MockContext, page::PageWrapper};

    use super::HomePage;

    #[tokio::test]
    async fn should_render_page() {
        let home = HomePage::default();
        let query_executor = MockMetric::new();
        let context = MockContext::new(query_executor);
        let mut buffer = String::with_capacity(1024);
        PageWrapper::new(home)
            .render(&context, &mut buffer)
            .await
            .unwrap();
        assert!(buffer.contains("<title>Home</title>"));
    }
}
