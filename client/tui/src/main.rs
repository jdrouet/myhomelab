use myhomelab_client_tui::ApplicationConfig;
use myhomelab_prelude::FromEnv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_config = ApplicationConfig::from_env()?;
    let mut app = app_config.build()?;

    let mut terminal = ratatui::init();
    let res = app.run(&mut terminal).await;
    ratatui::restore();
    res
}
