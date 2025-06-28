use axum::routing::post;

mod create;
mod query;

pub(super) fn create<S: crate::ServerState>() -> axum::Router<S> {
    axum::Router::new().route("/", post(create::handle::<S>).get(query::handle::<S>))
}
