use crate::ServerState;

mod api;

pub(super) fn create<S: ServerState>() -> axum::Router<S> {
    axum::Router::new().nest("/api", api::create())
}
