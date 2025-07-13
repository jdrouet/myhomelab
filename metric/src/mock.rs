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
            requests: std::collections::HashMap<Box<str>, crate::query::Request>,
            timerange: myhomelab_prelude::time::TimeRange,
        ) -> anyhow::Result<std::collections::HashMap<Box<str>, crate::query::Response>>;
    }
}
