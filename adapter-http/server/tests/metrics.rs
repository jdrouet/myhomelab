use std::{
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, atomic::AtomicU16},
};

use myhomelab_adapter_http_server::ServerState;
use myhomelab_metric::intake::Intake;
use myhomelab_metric_mock::MockMetric;
use myhomelab_prelude::Healthcheck;

static PORT_ITERATOR: AtomicU16 = AtomicU16::new(5000);

#[derive(Default)]
struct InnerState {
    metric: MockMetric,
}

impl std::fmt::Debug for InnerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(InnerState))
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Default)]
struct MockServerState(Arc<InnerState>);

impl ServerState for MockServerState {
    fn metric_intake(&self) -> &impl myhomelab_metric::intake::Intake {
        &self.0.metric
    }

    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor {
        &self.0.metric
    }
}

#[tokio::test]
async fn should_handle_healthcheck() {
    let port = PORT_ITERATOR.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let server_config = myhomelab_adapter_http_server::HttpServerConfig {
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
    };
    let mut state = InnerState {
        metric: MockMetric::new(),
    };
    state.metric.expect_healthcheck().returning(|| Ok(()));
    let state = MockServerState(Arc::new(state));
    let server = server_config.build(state.clone());
    let _handle = tokio::spawn(async { server.run().await });
    let client_config = myhomelab_adapter_http_client::AdapterHttpClientConfig {
        base_url: format!("http://localhost:{port}"),
    };
    let client = client_config.build().unwrap();
    client.healthcheck().await.unwrap();
}

#[tokio::test]
async fn should_ingest_metrics() {
    let port = PORT_ITERATOR.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let server_config = myhomelab_adapter_http_server::HttpServerConfig {
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
    };
    let mut state = InnerState {
        metric: MockMetric::new(),
    };
    state.metric.expect_ingest().once().returning(|metrics| {
        assert_eq!(metrics.len(), 10);
        Ok(())
    });
    let state = MockServerState(Arc::new(state));
    let server = server_config.build(state.clone());
    let _handle = tokio::spawn(async { server.run().await });
    let client_config = myhomelab_adapter_http_client::AdapterHttpClientConfig {
        base_url: format!("http://localhost:{port}"),
    };
    let client = client_config.build().unwrap();
    client.ingest(vec![
        myhomelab_metric::metrics!("system.memory.total", gauge, "host" => "rpi", [(0, 1024.0), (1, 1024.0), (2, 1024.0), (3, 1024.0), (4, 1024.0)]),
        myhomelab_metric::metrics!("system.memory.used", gauge, "host" => "rpi", [(0, 256.0), (1, 312.0), (2, 420.0), (3, 320.0), (4, 430.0)]),
    ].concat()).await.unwrap();
}
