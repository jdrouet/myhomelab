#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = myhomelab_http_server::HttpServerConfig::default();
    config.build().run().await
}
