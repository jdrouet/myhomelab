use axum::routing::head;
use myhomelab_agent_prelude::sensor::Sensor;

use crate::ServerState;

mod dashboard;
mod metric;
mod sensor;
mod status;

pub(super) fn create<S: ServerState>() -> axum::Router<S>
where
    for<'de> <<S as ServerState>::ManagerSensor as Sensor>::Cmd: serde::Deserialize<'de>,
{
    axum::Router::new()
        .route("/", head(status::handle::<S>))
        .nest("/dashboards", dashboard::create())
        .nest("/metrics", metric::create())
        .nest("/sensors", sensor::create())
}
