mod router;

#[derive(Clone, Debug)]
pub struct HttpServerConfig {
    pub host: std::net::IpAddr,
    pub port: u16,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            port: 3000,
        }
    }
}

impl HttpServerConfig {
    pub fn build<S: ServerState>(&self, state: S) -> HttpServer<S> {
        HttpServer {
            address: std::net::SocketAddr::from((self.host, self.port)),
            state,
        }
    }
}

#[derive(Debug)]
pub struct HttpServer<S: ServerState> {
    address: std::net::SocketAddr,
    state: S,
}

impl<S: ServerState> HttpServer<S> {
    #[tracing::instrument(skip_all, fields(address = %self.address))]
    pub async fn run(self) -> anyhow::Result<()> {
        let app = crate::router::create::<S>().with_state(self.state);
        tracing::debug!("binding socket");
        let listener = tokio::net::TcpListener::bind(self.address).await?;
        tracing::info!("starting server");
        axum::serve(listener, app).await?;
        Ok(())
    }
}

pub trait ServerState: Clone + Send + Sync + 'static {
    fn metric_intake(&self) -> &impl myhomelab_metric::intake::Intake;
    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor;
}
