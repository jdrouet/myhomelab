use axum::extract::State;
use axum::http::StatusCode;
use myhomelab_prelude::Healthcheck;

use crate::ServerState;

#[tracing::instrument(skip_all)]
pub(super) async fn handle<S: ServerState>(State(state): State<S>) -> StatusCode {
    let res = tokio::try_join!(
        state.metric_intake().healthcheck(),
        state.metric_query_executor().healthcheck(),
        state.sensor_manager().healthcheck(),
    );
    match res {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(err) => {
            tracing::error!(message = "healthcheck failed", cause = %err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
