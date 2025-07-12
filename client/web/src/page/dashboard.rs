use myhomelab_dashboard::entity::Dashboard;

#[derive(Debug)]
pub struct DashboardPage {
    dashboard: Dashboard,
}

impl DashboardPage {
    pub fn new(dashboard: Dashboard) -> Self {
        Self { dashboard }
    }
}

impl crate::prelude::Page for DashboardPage {
    fn title(&self) -> &str {
        self.dashboard.title.as_str()
    }

    fn render_body<C: crate::prelude::Context>(
        &self,
        _ctx: &C,
        buf: &mut String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        buf.push_str("Work in progress...");
        async { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use myhomelab_dashboard::{entity::Dashboard, repository::MockDashboardRepo};
    use myhomelab_metric::mock::MockMetric;
    use uuid::Uuid;

    use crate::{mock::MockContext, page::PageWrapper};

    use super::DashboardPage;

    #[tokio::test]
    async fn should_render_page() {
        let dashboard = Dashboard {
            id: Uuid::new_v4(),
            title: "System".into(),
            description: "System related metrics".into(),
            cells: Vec::new(),
        };
        let dashboard_page = DashboardPage::new(dashboard);
        let dashboard_repository = MockDashboardRepo::new();
        let query_executor = MockMetric::new();
        let context = MockContext::new(dashboard_repository, query_executor);
        let mut buffer = String::with_capacity(1024);
        PageWrapper::new(dashboard_page)
            .render(&context, &mut buffer)
            .await
            .unwrap();
        assert!(buffer.contains("<title>System</title>"));
    }
}
