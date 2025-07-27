use myhomelab_agent_prelude::sensor::Sensor;
use tokio_util::sync::CancellationToken;
use tower_http::trace::TraceLayer;

mod router;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct HttpServerConfig {
    #[serde(default = "HttpServerConfig::default_host")]
    pub host: std::net::IpAddr,
    #[serde(default = "HttpServerConfig::default_port")]
    pub port: u16,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: Self::default_host(),
            port: Self::default_port(),
        }
    }
}

impl HttpServerConfig {
    const fn default_host() -> std::net::IpAddr {
        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
    }

    const fn default_port() -> u16 {
        3000
    }

    pub fn build<S: ServerState>(&self, cancel: CancellationToken, state: S) -> HttpServer<S>
    where
        for<'de> <<S as ServerState>::ManagerSensor as Sensor>::Cmd: serde::Deserialize<'de>,
    {
        HttpServer {
            address: std::net::SocketAddr::from((self.host, self.port)),
            cancel,
            state,
        }
    }
}

#[derive(Debug)]
pub struct HttpServer<S: ServerState>
where
    for<'de> <<S as ServerState>::ManagerSensor as Sensor>::Cmd: serde::Deserialize<'de>,
{
    address: std::net::SocketAddr,
    cancel: CancellationToken,
    state: S,
}

impl<S: ServerState> HttpServer<S>
where
    for<'de> <<S as ServerState>::ManagerSensor as Sensor>::Cmd: serde::Deserialize<'de>,
{
    #[tracing::instrument(skip_all, fields(address = %self.address))]
    pub async fn run(self) -> anyhow::Result<()> {
        let Self {
            address,
            cancel,
            state,
        } = self;
        let app = crate::router::create::<S>()
            .layer(TraceLayer::new_for_http())
            .with_state(state);
        tracing::debug!("binding socket");
        let listener = tokio::net::TcpListener::bind(address).await?;
        tracing::info!("starting server");
        axum::serve(listener, app)
            .with_graceful_shutdown(cancel.cancelled_owned())
            .await?;
        Ok(())
    }
}

pub trait ServerState: Clone + Send + Sync + 'static {
    type ManagerSensor: Sensor;

    fn dashboard_repository(&self) -> &impl myhomelab_dashboard::repository::DashboardRepository;
    fn metric_intake(&self) -> &impl myhomelab_metric::intake::Intake;
    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor;
    fn sensor_manager(
        &self,
    ) -> &impl myhomelab_agent_prelude::manager::Manager<Sensor = Self::ManagerSensor>;
}
