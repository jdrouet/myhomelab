use listener::Event;
use ratatui::{Terminal, prelude::Backend};

mod hook;
mod listener;
mod view;
mod worker;

#[derive(Debug)]
pub struct ApplicationConfig {}

impl myhomelab_prelude::FromEnv for ApplicationConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl ApplicationConfig {
    pub fn build(&self) -> anyhow::Result<Application> {
        Ok(Application {})
    }
}

#[derive(Debug)]
pub struct Application {}

impl Application {
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        let interval = std::time::Duration::from_millis(500);
        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut listener = listener::Listener::new(interval, event_rx);
        let mut router = crate::view::Router::new(action_tx);

        let worker = tokio::task::spawn(async move {
            crate::worker::Worker::new(action_rx, event_tx).run().await
        });

        router.draw(terminal)?;
        while let Some(event) = listener.next().await {
            router.digest(&event);
            router.draw(terminal)?;
            if matches!(event, Event::Shutdown) {
                break;
            }
        }
        worker.await??;
        Ok(())
    }
}
