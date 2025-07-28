use axum::Json;
use axum::extract::State;
use myhomelab_sensor_prelude::manager::Manager;
use myhomelab_sensor_prelude::sensor::Sensor;

pub(super) async fn handle<S: crate::ServerState>(
    State(state): State<S>,
) -> Json<Vec<&'static str>> {
    let sensors = state
        .sensor_manager()
        .sensors()
        .map(|sensor| sensor.name())
        .collect::<Vec<_>>();
    Json(sensors)
}
