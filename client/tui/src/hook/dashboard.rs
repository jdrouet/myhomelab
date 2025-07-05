use std::sync::{Arc, Mutex};

use myhomelab_adapter_http_client::AdapterHttpClient;
use myhomelab_dashboard::entity::Dashboard;

struct DashboardListRunner {
    channel: Arc<Mutex<Option<DashboardListState>>>,
    client: AdapterHttpClient,
}

impl DashboardListRunner {
    fn update(&self, state: DashboardListState) {
        if let Ok(mut lock) = self.channel.lock() {
            *lock = Some(state);
        }
    }

    async fn run(self) {
        use myhomelab_dashboard::repository::DashboardRepository;

        self.update(DashboardListState::Loading);
        match self.client.list_dashboards().await {
            Ok(list) => {
                self.update(DashboardListState::Success(list));
            }
            Err(err) => {
                self.update(DashboardListState::Error(anyhow::Error::from(err)));
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum DashboardListState {
    Loading,
    Success(Vec<Dashboard>),
    #[allow(unused)]
    Error(anyhow::Error),
}

#[derive(Debug)]
pub(crate) struct DashboardListHook {
    channel: Arc<Mutex<Option<DashboardListState>>>,
    client: AdapterHttpClient,
    task: tokio::task::JoinHandle<()>,
}

impl DashboardListHook {
    pub(crate) fn new(client: AdapterHttpClient) -> Self {
        let channel = Arc::new(Mutex::new(None));

        let runner = DashboardListRunner {
            channel: channel.clone(),
            client: client.clone(),
        };
        let task = tokio::task::spawn(async move { runner.run().await });

        Self {
            client,
            channel,
            task,
        }
    }

    pub fn pull(&mut self) -> Option<DashboardListState> {
        if let Ok(mut value) = self.channel.lock() {
            std::mem::take(&mut value)
        } else {
            None
        }
    }

    pub fn execute(&mut self) {
        let runner = DashboardListRunner {
            channel: self.channel.clone(),
            client: self.client.clone(),
        };
        self.task = tokio::task::spawn(async move { runner.run().await });
    }
}
