use std::fmt::Write;

use myhomelab_dashboard::repository::DashboardRepository;
use myhomelab_prelude::time::RelativeTimeRange;

use crate::prelude::Component;

#[derive(Debug, Default)]
pub struct HomePage {}

impl crate::prelude::Page for HomePage {
    fn title(&self) -> &str {
        "Home"
    }

    async fn render_body<C: crate::prelude::Context>(
        &self,
        ctx: &C,
        buf: &mut String,
    ) -> anyhow::Result<()> {
        crate::component::header::Header::new(RelativeTimeRange::LastDay.into())
            .render(ctx, buf)
            .await?;

        let dashboards = ctx.dashboard_repository().list_dashboards().await?;

        buf.push_str("<main>");
        if dashboards.is_empty() {
            buf.push_str("No dashboard found...");
        } else {
            for dashboard in dashboards {
                let url = format!("/dashboards/{}", dashboard.id);
                write!(buf, "<a href={url:?} title={:?}>", dashboard.title)?;
                write!(buf, "<h3>{}</h3>", dashboard.title)?;
                write!(buf, "<p>{}</p>", dashboard.description)?;
                buf.push_str("</a>");
            }
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
    use uuid::Uuid;

    use super::HomePage;
    use crate::mock::MockContext;
    use crate::page::PageWrapper;

    #[tokio::test]
    async fn should_render_page_with_empty_state() {
        let home = HomePage::default();
        let mut dashboard_repository = MockDashboardRepo::new();
        dashboard_repository
            .expect_list_dashboards()
            .once()
            .returning(|| Box::pin(async { Ok(Vec::new()) }));
        let query_executor = MockMetric::new();
        let context = MockContext::new(dashboard_repository, query_executor);
        let mut buffer = String::with_capacity(1024);
        PageWrapper::new(home)
            .render(&context, &mut buffer)
            .await
            .unwrap();
        assert!(buffer.contains("<title>Home</title>"));
        assert!(buffer.contains("No dashboard found..."));
        let MockContext {
            mut dashboard_repository,
            ..
        } = context;
        dashboard_repository.checkpoint();
    }

    #[tokio::test]
    async fn should_render_page_with_list_of_dashboards() {
        let home = HomePage::default();
        let mut dashboard_repository = MockDashboardRepo::new();
        let board_id = Uuid::new_v4();
        dashboard_repository
            .expect_list_dashboards()
            .once()
            .returning(move || {
                Box::pin(async move {
                    Ok(vec![Dashboard {
                        id: board_id,
                        title: "System".into(),
                        description: "System related metrics".into(),
                        cells: Default::default(),
                    }])
                })
            });
        let query_executor = MockMetric::new();
        let context = MockContext::new(dashboard_repository, query_executor);
        let mut buffer = String::with_capacity(1024);
        PageWrapper::new(home)
            .render(&context, &mut buffer)
            .await
            .unwrap();
        assert!(buffer.contains("<title>Home</title>"));
        assert!(buffer.contains("System related metrics"));
        assert!(buffer.contains(&format!("/dashboards/{board_id}")));
        let MockContext {
            mut dashboard_repository,
            ..
        } = context;
        dashboard_repository.checkpoint();
    }
}
