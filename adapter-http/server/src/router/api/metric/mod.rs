use axum::routing::post;

mod intake;
mod query;

pub(super) fn create<S: crate::ServerState>() -> axum::Router<S> {
    axum::Router::new()
        .route("/intake", post(intake::handle::<S>))
        .route(
            "/query",
            post(query::handle_batch::<S>).get(query::handle_single::<S>),
        )
}
