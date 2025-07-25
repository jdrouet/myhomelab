use axum::routing::post;

mod execute;

pub(super) fn create<S: crate::ServerState>() -> axum::Router<S> {
    axum::Router::new().route("/execute", post(execute::handle::<S>))
}
