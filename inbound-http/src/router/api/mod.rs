use axum::routing::head;

use crate::ServerState;

mod metric;
mod status;

pub(super) fn create<S: ServerState>() -> axum::Router<S> {
    axum::Router::new()
        .route("/", head(status::handle::<S>))
        .nest("/metrics", metric::create())
}
