use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use myhomelab_adapter_http_shared::metric::create::Payload;
use myhomelab_metric::intake::Intake;

pub(super) async fn handle<S: crate::ServerState>(
    State(state): State<S>,
    Json(payload): Json<Payload<'_>>,
) -> StatusCode {
    let metrics = payload.metrics().collect::<Vec<_>>();
    match state.metric_intake().ingest(&metrics).await {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            tracing::error!(message = "unable to ingest metrics", cause = ?err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
