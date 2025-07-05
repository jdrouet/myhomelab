use myhomelab_adapter_file::{AdapterFile, AdapterFileConfig};
use myhomelab_adapter_http_server::{HttpServerConfig, ServerState};
use myhomelab_adapter_sqlite::{Sqlite, SqliteConfig};
use myhomelab_agent_core::{Manager, ManagerConfig};
use myhomelab_prelude::FromEnv;
use tokio_util::sync::CancellationToken;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let file_config = AdapterFileConfig::from_env()?;
    let file = file_config.build()?;

    let sqlite_config = SqliteConfig::from_env()?;
    let sqlite = sqlite_config.build().await?;
    sqlite.prepare().await?;

    let cancel_token = CancellationToken::new();

    let manager_config = ManagerConfig::default();
    let manager = Manager::unbounded_builder(cancel_token.child_token(), sqlite.clone());
    let manager =
        manager.with_reader(myhomelab_agent_reader_system::ReaderSystemConfig::default().build()?);
    let manager = manager.with_reader(
        myhomelab_agent_reader_xiaomi_lywsd03mmc_atc::ReaderXiaomiConfig::default()
            .build()
            .await?,
    );
    let manager = manager.build(&manager_config);

    let app_state = AppState { file, sqlite };

    let http_server_config = HttpServerConfig::from_env()?;
    let http_server = http_server_config.build(cancel_token.child_token(), app_state);

    tokio::try_join!(
        shutdown_signal(cancel_token),
        manager.run(),
        http_server.run(),
    )?;

    Ok(())
}
