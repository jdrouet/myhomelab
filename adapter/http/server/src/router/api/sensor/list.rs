use axum::Json;
use axum::extract::State;
use myhomelab_adapter_http_shared::sensor::SensorDescriptor as HttpSensorDescriptor;
use myhomelab_sensor_prelude::manager::Manager;
use myhomelab_sensor_prelude::sensor::Sensor;

#[tracing::instrument(skip_all)]
pub(super) async fn handle<S: crate::ServerState>(
    State(state): State<S>,
) -> Json<Vec<HttpSensorDescriptor>> {
    let sensors = state
        .sensor_manager()
        .sensors()
        .map(|sensor| HttpSensorDescriptor::from(sensor.descriptor()))
        .collect::<Vec<_>>();
    Json(sensors)
}
