use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::listener::Event;

#[derive(Debug)]
pub(crate) enum Action {
    FetchDashboardList,
    Shutdown,
    ViewEvent(Event),
}

#[derive(Debug)]
pub(crate) struct Worker {
    receiver: UnboundedReceiver<Action>,
    sender: UnboundedSender<Event>,
}

impl Worker {
    pub(crate) fn new(receiver: UnboundedReceiver<Action>, sender: UnboundedSender<Event>) -> Self {
        Self { receiver, sender }
    }
}

impl Worker {
    async fn handle_fetch_dashboard_list(&self) {}

    pub(crate) async fn run(mut self) -> anyhow::Result<()> {
        while let Some(action) = self.receiver.recv().await {
            match action {
                Action::FetchDashboardList => {
                    self.handle_fetch_dashboard_list().await;
                }
                Action::Shutdown => {
                    let _ = self.sender.send(Event::Shutdown);
                    break;
                }
                Action::ViewEvent(inner) => {
                    let _ = self.sender.send(inner);
                }
            }
        }
        Ok(())
    }
}
