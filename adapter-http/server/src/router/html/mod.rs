use axum::routing::get;
use myhomelab_client_web::prelude::Context;

use crate::ServerState;

mod dashboard;
mod home;

pub(super) fn create<S: ServerState>() -> axum::Router<S> {
    axum::Router::new()
        .route("/", get(home::handle::<S>))
        .route("/dashboards/{dashboard_id}", get(dashboard::handle::<S>))
}

/// Wrapper arround server state for the web client
struct ServerContext<S: ServerState>(S);

impl<S: ServerState> Context for ServerContext<S> {
    fn dashboard_repository(&self) -> &impl myhomelab_dashboard::repository::DashboardRepository {
        self.0.dashboard_repository()
    }

    fn metric_query_executor(&self) -> &impl myhomelab_metric::query::QueryExecutor {
        self.0.metric_query_executor()
    }
}
