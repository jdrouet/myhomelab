use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use myhomelab_adapter_http_client::AdapterHttpClient;
use myhomelab_metric::query::{Query, Request, RequestKind, Response, TimeRange};

struct QueryRunner {
    channel: Arc<Mutex<Option<QueryState>>>,
    client: AdapterHttpClient,
    request: Request,
}

impl QueryRunner {
    fn update(&self, state: QueryState) {
        if let Ok(mut lock) = self.channel.lock() {
            *lock = Some(state);
        }
    }

    async fn run(self) {
        use myhomelab_metric::query::QueryExecutor;

        self.update(QueryState::Loading);
        let mut requests = HashMap::with_capacity(1);
        requests.insert(Box::from("default"), self.request.clone());
        let timerange = TimeRange::from(0);
        match self.client.execute(requests, timerange).await {
            Ok(res) => {
                self.update(QueryState::Success(res));
            }
            Err(err) => {
                self.update(QueryState::Error(err));
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum QueryState {
    Loading,
    Success(HashMap<Box<str>, Response>),
    #[allow(unused)]
    Error(anyhow::Error),
}

#[derive(Debug)]
pub(crate) struct QueryHook {
    channel: Arc<Mutex<Option<QueryState>>>,
    client: AdapterHttpClient,
    request: Request,
    task: tokio::task::JoinHandle<()>,
}

impl QueryHook {
    pub(crate) fn new(client: AdapterHttpClient, kind: RequestKind, query: Query) -> Self {
        let channel = Arc::new(Mutex::new(None));

        let request = Request { kind, query };

        let runner = QueryRunner {
            channel: channel.clone(),
            client: client.clone(),
            request: request.clone(),
        };
        let task = tokio::task::spawn(async move { runner.run().await });

        Self {
            client,
            channel,
            request,
            task,
        }
    }

    pub fn pull(&mut self) -> Option<QueryState> {
        if let Ok(mut value) = self.channel.lock() {
            std::mem::take(&mut value)
        } else {
            None
        }
    }

    pub fn execute(&mut self) {
        let runner = QueryRunner {
            channel: self.channel.clone(),
            client: self.client.clone(),
            request: self.request.clone(),
        };
        self.task = tokio::task::spawn(async move { runner.run().await });
    }
}
