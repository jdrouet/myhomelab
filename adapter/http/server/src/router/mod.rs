use std::time::Duration;

use axum::Extension;
use axum::http::StatusCode;

use crate::ServerState;

mod api;
mod html;

pub(super) fn create<S: ServerState>() -> axum::Router<S> {
    html::create()
        .nest("/api", api::create())
        .layer(Extension(
            serde_qs::axum::QsQueryConfig::new()
                .config(serde_qs::Config::default())
                .error_handler(|err| {
                    serde_qs::axum::QsQueryRejection::new(err, StatusCode::UNPROCESSABLE_ENTITY)
                }),
        ))
        .layer(tower_http::timeout::TimeoutLayer::new(
            Duration::from_millis(500),
        ))
}
