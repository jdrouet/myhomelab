use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use myhomelab_sensor_prelude::manager::Manager;
use myhomelab_sensor_prelude::sensor::Sensor;

#[tracing::instrument(skip_all, fields(name))]
pub(super) async fn handle<S: crate::ServerState>(
    State(state): State<S>,
    Path(name): Path<String>,
    Json(payload): Json<<<S as crate::ServerState>::ManagerSensor as Sensor>::Cmd>,
) -> StatusCode
where
    for<'de> <<S as crate::ServerState>::ManagerSensor as Sensor>::Cmd: serde::Deserialize<'de>,
{
    let Some(sensor) = state.sensor_manager().get_sensor(name.as_str()) else {
        tracing::debug!(message = "sensor not found", name = %name);
        return StatusCode::NOT_FOUND;
    };
    let Err(err) = sensor.execute(payload).await else {
        tracing::debug!(message = "execution triggered", name = %name);
        return StatusCode::CREATED;
    };
    tracing::error!(message = "unable to execute command", error = %err);
    StatusCode::INTERNAL_SERVER_ERROR
}
