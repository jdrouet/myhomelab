use myhomelab_adapter_sqlite::Sqlite;
use myhomelab_metric::entity::Metric;
use myhomelab_metric::intake::Intake;

#[derive(Clone, Debug)]
pub(crate) struct Collector {
    pub(crate) sqlite: Sqlite,
}

impl myhomelab_agent_prelude::collector::Collector for Collector {
    async fn push_metrics(&self, metrics: &[Metric]) -> anyhow::Result<()> {
        self.sqlite.ingest(metrics).await
    }
}
