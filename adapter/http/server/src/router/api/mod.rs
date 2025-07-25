use axum::routing::head;

use crate::ServerState;

mod dashboard;
mod metric;
mod sensor;
mod status;

pub(super) fn create<S: ServerState>() -> axum::Router<S> {
    axum::Router::new()
        .route("/", head(status::handle::<S>))
        .nest("/dashboards", dashboard::create())
        .nest("/metrics", metric::create())
        .nest("/sensors", sensor::create())
}
