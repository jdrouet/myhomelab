#![cfg(any(test, feature = "mocks"))]

pub struct MockContext<MQE> {
    metric_query_executor: MQE,
}

impl<MQE> MockContext<MQE>
where
    MQE: myhomelab_metric::query::QueryExecutor + Send + Sync,
{
    pub fn new(metric_query_executor: MQE) -> Self {
        Self {
            metric_query_executor,
        }
    }
}

impl<MQE> crate::prelude::Context for MockContext<MQE>
where
    MQE: myhomelab_metric::query::QueryExecutor + Send + Sync,
{
    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor {
        &self.metric_query_executor
    }
}
