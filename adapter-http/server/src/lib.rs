use tokio_util::sync::CancellationToken;
use tower_http::trace::TraceLayer;

mod router;

const DEFAULT_HOST: std::net::IpAddr = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
const DEFAULT_PORT: u16 = 3000;

#[derive(Clone, Debug)]
pub struct HttpServerConfig {
    pub host: std::net::IpAddr,
    pub port: u16,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST,
            port: DEFAULT_PORT,
        }
    }
}

impl myhomelab_prelude::FromEnv for HttpServerConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: myhomelab_prelude::parse_from_env("MYHOMELAB_HTTP_HOST")?.unwrap_or(DEFAULT_HOST),
            port: myhomelab_prelude::parse_from_env("MYHOMELAB_HTTP_PORT")?.unwrap_or(DEFAULT_PORT),
        })
    }
}

impl HttpServerConfig {
    pub fn build<S: ServerState>(&self, cancel: CancellationToken, state: S) -> HttpServer<S> {
        HttpServer {
            address: std::net::SocketAddr::from((self.host, self.port)),
            cancel,
            state,
        }
    }
}

#[derive(Debug)]
pub struct HttpServer<S: ServerState> {
    address: std::net::SocketAddr,
    cancel: CancellationToken,
    state: S,
}

impl<S: ServerState> HttpServer<S> {
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
    fn dashboard_repository(&self) -> &impl myhomelab_dashboard::repository::DashboardRepository;
    fn metric_intake(&self) -> &impl myhomelab_metric::intake::Intake;
    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor;
}
