use axum::routing::{get, post};
use myhomelab_sensor_prelude::sensor::Sensor;

use crate::ServerState;

mod execute;
mod list;

pub(super) fn create<S: ServerState>() -> axum::Router<S>
where
    for<'de> <<S as ServerState>::ManagerSensor as Sensor>::Cmd: serde::Deserialize<'de>,
{
    axum::Router::new()
        .route("/", get(list::handle::<S>))
        .route("/{name}/execute", post(execute::handle::<S>))
}
