use std::sync::Arc;

use anyhow::Context;
use myhomelab_adapter_dataset::{AdapterDataset, AdapterDatasetConfig};
use myhomelab_adapter_http_server::ServerState;
use myhomelab_adapter_sqlite::{Sqlite, SqliteConfig};
use myhomelab_sensor_manager::sensor::AnySensor;
use myhomelab_sensor_prelude::manager::{Manager, ManagerBuilder};
use myhomelab_sensor_prelude::sensor::BuildContext;
use tokio_util::sync::CancellationToken;

mod collector;

#[derive(Clone, Debug)]
struct AppState {
    dataset: AdapterDataset,
    manager: Arc<myhomelab_sensor_manager::Manager>,
    sqlite: Sqlite,
}

impl ServerState for AppState {
    type ManagerSensor = AnySensor;

    fn dashboard_repository(&self) -> &impl myhomelab_dashboard::repository::DashboardRepository {
        &self.dataset
    }

    fn metric_intake(&self) -> &impl myhomelab_metric::intake::Intake {
        &self.sqlite
    }

    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor {
        &self.sqlite
    }

    fn sensor_manager(
        &self,
    ) -> &impl myhomelab_sensor_prelude::manager::Manager<Sensor = AnySensor> {
        self.manager.as_ref()
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

#[derive(Debug, Default, serde::Deserialize)]
struct AdapterConfig {
    #[serde(default)]
    sqlite: SqliteConfig,
}

#[derive(Debug, serde::Deserialize)]
struct ServerConfig {
    #[serde(default)]
    adapters: AdapterConfig,
    #[serde(default)]
    dataset: AdapterDatasetConfig,
    #[serde(default)]
    http: myhomelab_adapter_http_server::HttpServerConfig,
    #[serde(default)]
    manager: myhomelab_sensor_manager::config::ManagerConfig,
    #[serde(default)]
    tracing: myhomelab_adapter_opentelementry::OpenTelemetryConfig,
}

impl ServerConfig {
    fn build(path: Option<String>) -> anyhow::Result<Self> {
        let settings = config::Config::builder();
        let settings = if let Some(ref path) = path {
            settings.add_source(config::File::with_name(path))
        } else {
            settings
        };
        let settings = settings.add_source(
            config::Environment::default()
                .separator("__")
                .list_separator(","),
        );
        let settings = settings.build().context("unable to build config")?;
        settings
            .try_deserialize()
            .context("unable to deserialize config")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_config = ServerConfig::build(std::env::args().nth(1))?;

    server_config.tracing.setup()?;

    let sqlite = server_config.adapters.sqlite.build().await?;
    sqlite.prepare().await?;

    let cancel_token = CancellationToken::new();
    let builder_ctx = BuildContext {
        cancel: cancel_token.clone(),
        collector: crate::collector::Collector {
            sqlite: sqlite.clone(),
        },
    };

    let manager = Arc::new(server_config.manager.build(&builder_ctx).await?);

    let app_state = AppState {
        dataset: server_config.dataset.build(),
        sqlite,
        manager: manager.clone(),
    };

    let http_server = server_config
        .http
        .build(cancel_token.child_token(), app_state);

    tokio::try_join!(shutdown_signal(cancel_token), http_server.run())?;

    if let Some(manager) = Arc::into_inner(manager) {
        manager.wait().await?;
    }

    Ok(())
}
