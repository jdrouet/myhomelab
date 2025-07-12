use std::collections::HashMap;

use myhomelab_dashboard::entity::Dashboard;
use myhomelab_metric::query::{QueryExecutor, Request, Response, TimeRange};

use crate::prelude::Component;

struct DashboardCell<'a>(&'a myhomelab_dashboard::entity::DashboardCell);

impl<'a> crate::prelude::Component for DashboardCell<'a> {
    async fn render<C: crate::prelude::Context>(
        &self,
        context: &C,
        buf: &mut String,
    ) -> anyhow::Result<()> {
        let timerange = TimeRange::last_1day();
        let mut requests = HashMap::new();
        requests.insert(
            "default".into(),
            Request {
                kind: self.0.kind,
                query: self.0.query.clone(),
            },
        );
        let result = context
            .metric_query_executor()
            .execute(requests, timerange)
            .await?;
        buf.push_str("<div class=\"cell\">");
        buf.push_str("<div class=\"cell-title\">");
        if let Some(ref title) = self.0.title {
            buf.push_str(title);
        } else {
            buf.push_str("<i>No title</i>");
        }
        buf.push_str("</div>");
        match result.get("default") {
            Some(Response::Timeseries(data)) => {
                crate::component::line_chart::LineChart::new(data, timerange)
                    .render(context, buf)
                    .await?;
            }
            _ => {}
        }
        buf.push_str("</div>");
        Ok(())
    }
}

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
        ctx: &C,
        buf: &mut String,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        async {
            buf.push_str("<main>");
            for cell in self.dashboard.cells.iter() {
                DashboardCell(cell).render(ctx, buf).await?;
            }
            buf.push_str("</main>");
            Ok(())
        }
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
