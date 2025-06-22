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
    pub fn build(&self) -> HttpServer {
        HttpServer {
            address: std::net::SocketAddr::from((self.host, self.port)),
        }
    }
}

#[derive(Debug)]
pub struct HttpServer {
    address: std::net::SocketAddr,
}

impl HttpServer {
    #[tracing::instrument(skip_all, fields(address = %self.address))]
    pub async fn run(self) -> anyhow::Result<()> {
        let app = axum::Router::new();
        tracing::debug!("binding socket");
        let listener = tokio::net::TcpListener::bind(self.address).await?;
        tracing::info!("starting server");
        axum::serve(listener, app).await?;
        Ok(())
    }
}
