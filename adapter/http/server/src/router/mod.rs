use std::time::Duration;

use axum::Extension;
use axum::http::StatusCode;
use myhomelab_sensor_prelude::sensor::Sensor;

use crate::ServerState;

mod api;
mod html;

pub(super) fn create<S: ServerState>() -> axum::Router<S>
where
    for<'de> <<S as ServerState>::ManagerSensor as Sensor>::Cmd: serde::Deserialize<'de>,
{
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
