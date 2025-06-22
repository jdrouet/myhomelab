use myhomelab_adapter_sqlite::{Sqlite, SqliteConfig};
use myhomelab_inbound_http::{HttpServerConfig, ServerState};
use myhomelab_prelude::FromEnv;

#[derive(Clone, Debug)]
struct AppState {
    sqlite: Sqlite,
}

impl ServerState for AppState {
    fn metric_intake(&self) -> &impl myhomelab_metric::intake::Intake {
        &self.sqlite
    }

    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor {
        &self.sqlite
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let sqlite_config = SqliteConfig::from_env()?;
    let sqlite = sqlite_config.build().await?;

    let app_state = AppState { sqlite };

    let http_server_config = HttpServerConfig::from_env()?;
    let http_server = http_server_config.build(app_state);

    http_server.run().await
}
