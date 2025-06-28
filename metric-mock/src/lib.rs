mockall::mock! {
    pub Metric {}

    impl myhomelab_prelude::Healthcheck for Metric {
        async fn healthcheck(&self) -> anyhow::Result<()>;
    }

    impl myhomelab_metric::intake::Intake for Metric {
        async fn ingest(&self, values: Vec<myhomelab_metric::entity::Metric>) -> anyhow::Result<()>;
    }

    impl myhomelab_metric::query::QueryExecutor for Metric {
        async fn execute(
            &self,
            requests: Vec<myhomelab_metric::query::Request>,
            timerange: myhomelab_metric::query::TimeRange,
        ) -> anyhow::Result<Vec<myhomelab_metric::query::Response>>;
    }
}
