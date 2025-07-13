use std::collections::HashMap;

use myhomelab_dashboard::entity::Dashboard;
use myhomelab_metric::query::{QueryExecutor, Request, Response};
use myhomelab_prelude::time::TimeRange;

use crate::prelude::Component;

struct DashboardCell<'a> {
    inner: &'a myhomelab_dashboard::entity::DashboardCell,
    timerange: TimeRange,
}

impl<'a> DashboardCell<'a> {
    pub fn new(
        inner: &'a myhomelab_dashboard::entity::DashboardCell,
        timerange: TimeRange,
    ) -> Self {
        Self { inner, timerange }
    }
}

impl<'a> crate::prelude::Component for DashboardCell<'a> {
    async fn render<C: crate::prelude::Context>(
        &self,
        context: &C,
        buf: &mut String,
    ) -> anyhow::Result<()> {
        let mut requests = HashMap::new();
        requests.insert(
            "default".into(),
            Request {
                kind: self.inner.kind,
                query: self.inner.query.clone(),
            },
        );
        let result = context
            .metric_query_executor()
            .execute(requests, self.timerange)
            .await?;
        buf.push_str("<div class=\"cell\">");
        buf.push_str("<h2 class=\"cell-title\">");
        if let Some(ref title) = self.inner.title {
            buf.push_str(title);
        } else {
            buf.push_str("<i>No title</i>");
        }
        buf.push_str("</h2>");
        if let Some(Response::Timeseries(data)) = result.get("default") {
            crate::component::line_chart::LineChart::new(data, self.timerange)
                .render(context, buf)
                .await?;
        }
        buf.push_str("</div>");
        Ok(())
    }
}

#[derive(Debug)]
pub struct DashboardPage {
    dashboard: Dashboard,
    timerange: TimeRange,
}

impl DashboardPage {
    pub fn new(dashboard: Dashboard, timerange: TimeRange) -> Self {
        Self {
            dashboard,
            timerange,
        }
    }
}

impl crate::prelude::Page for DashboardPage {
    fn title(&self) -> &str {
        self.dashboard.title.as_str()
    }

    async fn render_body<C: crate::prelude::Context>(
        &self,
        ctx: &C,
        buf: &mut String,
    ) -> anyhow::Result<()> {
        crate::component::header::Header::new(self.timerange)
            .render(ctx, buf)
            .await?;
        buf.push_str("<main class=\"container\">");
        for cell in self.dashboard.cells.iter() {
            DashboardCell::new(cell, self.timerange)
                .render(ctx, buf)
                .await?;
        }
        buf.push_str("</main>");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use myhomelab_dashboard::entity::Dashboard;
    use myhomelab_dashboard::repository::MockDashboardRepo;
    use myhomelab_metric::mock::MockMetric;
    use myhomelab_prelude::time::RelativeTimeRange;
    use uuid::Uuid;

    use super::DashboardPage;
    use crate::mock::MockContext;
    use crate::page::PageWrapper;

    #[tokio::test]
    async fn should_render_page() {
        let dashboard = Dashboard {
            id: Uuid::new_v4(),
            title: "System".into(),
            description: "System related metrics".into(),
            cells: Vec::new(),
        };
        let timerange = RelativeTimeRange::LastDay.into();
        let dashboard_page = DashboardPage::new(dashboard, timerange);
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
