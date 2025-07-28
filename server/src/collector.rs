use myhomelab_adapter_sqlite::Sqlite;
use myhomelab_metric::entity::Metric;

#[derive(Clone, Debug)]
pub(crate) struct Collector {
    pub(crate) sqlite: Sqlite,
}

impl myhomelab_sensor_prelude::collector::Collector for Collector {
    async fn push_metrics<'h>(&self, metrics: &[Metric<'h>]) -> anyhow::Result<()> {
        use myhomelab_metric::intake::Intake;

        self.sqlite.ingest(metrics).await
    }

    async fn push_event<I>(&self, input: I) -> anyhow::Result<()>
    where
        I: myhomelab_event::intake::IntakeInput,
        I: 'static,
    {
        use myhomelab_event::intake::Intake;

        self.sqlite.ingest(input).await
    }
}
