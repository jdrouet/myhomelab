use axum::routing::post;

mod create;

pub(super) fn create<S: crate::ServerState>() -> axum::Router<S> {
    axum::Router::new().route("/", post(create::handle::<S>))
}
