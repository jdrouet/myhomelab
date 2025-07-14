use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use myhomelab_dashboard::entity::Dashboard;
use myhomelab_dashboard::repository::DashboardRepository;

pub(super) async fn handle<S: crate::ServerState>(
    State(state): State<S>,
    Path(dashboard_id): Path<uuid::Uuid>,
) -> Result<Json<Dashboard>, StatusCode> {
    let item = state
        .dashboard_repository()
        .find_dashboard_by_id(dashboard_id)
        .await
        .map_err(|err| {
            tracing::error!(message = "unable to list dashboards", cause = ?err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let item = item.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(item))
}
