use myhomelab_adapter_http_client::AdapterHttpClient;
use myhomelab_dashboard::repository::DashboardRepository;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::listener::{AsyncEvent, Event};

#[derive(Debug)]
pub(crate) enum Action {
    FetchDashboardList,
    Shutdown,
    ViewEvent(Event),
}

#[derive(Debug)]
pub(crate) struct Worker {
    client: AdapterHttpClient,
    receiver: UnboundedReceiver<Action>,
    sender: UnboundedSender<Event>,
}

impl Worker {
    pub(crate) fn new(
        client: AdapterHttpClient,
        receiver: UnboundedReceiver<Action>,
        sender: UnboundedSender<Event>,
    ) -> Self {
        Self {
            client,
            receiver,
            sender,
        }
    }
}

impl Worker {
    async fn handle_fetch_dashboard_list(&self) {
        let _ = self.sender.send(Event::DashboardList(AsyncEvent::Init));
        let _ = match self.client.list_dashboards().await {
            Ok(res) => self
                .sender
                .send(Event::DashboardList(AsyncEvent::Success(res))),
            Err(err) => self
                .sender
                .send(Event::DashboardList(AsyncEvent::Error(err))),
        };
    }

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
