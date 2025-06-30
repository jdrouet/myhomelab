use axum::routing::get;

mod list;

pub(super) fn create<S: crate::ServerState>() -> axum::Router<S> {
    axum::Router::new().route("/", get(list::handle::<S>))
}
