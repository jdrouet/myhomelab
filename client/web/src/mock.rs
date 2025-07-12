#![cfg(any(test, feature = "mocks"))]

pub struct MockContext<DR, MQE> {
    pub dashboard_repository: DR,
    pub metric_query_executor: MQE,
}

impl<DR, MQE> MockContext<DR, MQE>
where
    DR: myhomelab_dashboard::repository::DashboardRepository + Send + Sync,
    MQE: myhomelab_metric::query::QueryExecutor + Send + Sync,
{
    pub fn new(dashboard_repository: DR, metric_query_executor: MQE) -> Self {
        Self {
            dashboard_repository,
            metric_query_executor,
        }
    }
}

impl<DR, MQE> crate::prelude::Context for MockContext<DR, MQE>
where
    DR: myhomelab_dashboard::repository::DashboardRepository + Send + Sync,
    MQE: myhomelab_metric::query::QueryExecutor + Send + Sync,
{
    fn dashboard_repository(&self) -> &impl myhomelab_dashboard::repository::DashboardRepository {
        &self.dashboard_repository
    }

    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor {
        &self.metric_query_executor
    }
}
