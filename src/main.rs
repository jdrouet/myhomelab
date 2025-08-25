use myhomelab::{ApplicationConfig, Configurable};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ApplicationConfig::from_env()?;
    let app = config.build().await?;
    app.run().await
}
