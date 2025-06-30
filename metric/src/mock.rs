#![cfg(feature = "mocks")]

mockall::mock! {
    pub Metric {}

    impl myhomelab_prelude::Healthcheck for Metric {
        async fn healthcheck(&self) -> anyhow::Result<()>;
    }

    impl crate::intake::Intake for Metric {
        async fn ingest(&self, values: Vec<crate::entity::Metric>) -> anyhow::Result<()>;
    }

    impl crate::query::QueryExecutor for Metric {
        async fn execute(
            &self,
            requests: Vec<crate::query::Request>,
            timerange: crate::query::TimeRange,
        ) -> anyhow::Result<Vec<crate::query::Response>>;
    }
}
