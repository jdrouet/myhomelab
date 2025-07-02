use axum::routing::get;

mod get_by_id;
mod list;

pub(super) fn create<S: crate::ServerState>() -> axum::Router<S> {
    axum::Router::new()
        .route("/", get(list::handle::<S>))
        .route("/{dashboard_id}", get(get_by_id::handle::<S>))
}
