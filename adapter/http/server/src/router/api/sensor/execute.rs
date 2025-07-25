use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use myhomelab_adapter_http_shared::sensor::execute::Payload;
use myhomelab_agent_prelude::sensor::Sensor;

pub(super) async fn handle<S: crate::ServerState>(
    State(state): State<S>,
    Json(payload): Json<Payload>,
) -> StatusCode {
    match state.sensor_manager().execute(payload).await {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            tracing::error!(message = "unable to execute command", error = %err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
