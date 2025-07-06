use listener::Event;
use myhomelab_adapter_http_client::{AdapterHttpClient, AdapterHttpClientConfig};
use ratatui::Terminal;
use ratatui::prelude::Backend;

use crate::prelude::Component;

mod hook;
mod listener;
mod prelude;
mod view;

#[derive(Debug)]
pub struct ApplicationConfig {}

impl myhomelab_prelude::FromEnv for ApplicationConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl ApplicationConfig {
    pub fn build(&self) -> anyhow::Result<Application> {
        Ok(Application {
            client: AdapterHttpClientConfig::new("http://localhost:3000").build()?,
        })
    }
}

#[derive(Debug)]
pub struct Application {
    client: AdapterHttpClient,
}

impl Application {
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        let interval = std::time::Duration::from_millis(200);
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let context = crate::prelude::Context { sender: event_tx };
        let mut listener = listener::Listener::new(interval, event_rx);
        let mut router = crate::view::Router::new(self.client.clone());

        router.draw(terminal)?;
        while let Some(event) = listener.next().await {
            let shutdown = matches!(event, Event::Shutdown);
            router.digest(&context, &event);
            router.draw(terminal)?;
            if shutdown {
                break;
            }
        }
        Ok(())
    }
}
