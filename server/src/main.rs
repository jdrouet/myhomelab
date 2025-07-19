use anyhow::Context;
use myhomelab_adapter_file::{AdapterFile, AdapterFileConfig};
use myhomelab_adapter_http_server::ServerState;
use myhomelab_adapter_sqlite::{Sqlite, SqliteConfig};
use myhomelab_agent_prelude::reader::BuildContext;
use myhomelab_agent_prelude::reader::{Reader, ReaderBuilder};
use myhomelab_prelude::FromEnv;
use tokio_util::sync::CancellationToken;

mod collector;

#[derive(Clone, Debug)]
struct AppState {
    file: AdapterFile,
    sqlite: Sqlite,
}

impl ServerState for AppState {
    fn dashboard_repository(&self) -> &impl myhomelab_dashboard::repository::DashboardRepository {
        &self.file
    }

    fn metric_intake(&self) -> &impl myhomelab_metric::intake::Intake {
        &self.sqlite
    }

    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor {
        &self.sqlite
    }
}

async fn shutdown_signal(cancel: CancellationToken) -> anyhow::Result<()> {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    cancel.cancel();
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct ServerConfig {
    #[serde(default)]
    http: myhomelab_adapter_http_server::HttpServerConfig,
    #[serde(default)]
    manager: myhomelab_agent_manager::ManagerConfig,
}

impl ServerConfig {
    fn from_env() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .list_separator(","),
            )
            .build()
            .context("unable to build config")?;
        settings
            .try_deserialize()
            .context("unable to deserialize config")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let file_config = AdapterFileConfig::from_env()?;
    let file = file_config.build()?;

    let sqlite_config = SqliteConfig::from_env()?;
    let sqlite = sqlite_config.build().await?;
    sqlite.prepare().await?;

    let cancel_token = CancellationToken::new();
    let builder_ctx = BuildContext {
        cancel: cancel_token.clone(),
        collector: crate::collector::Collector {
            sqlite: sqlite.clone(),
        },
    };

    let server_config = ServerConfig::from_env()?;
    let manager = server_config.manager.build(&builder_ctx).await?;

    let app_state = AppState { file, sqlite };

    let http_server = server_config
        .http
        .build(cancel_token.child_token(), app_state);

    tokio::try_join!(shutdown_signal(cancel_token), http_server.run(),)?;
    manager.wait().await?;

    Ok(())
}
