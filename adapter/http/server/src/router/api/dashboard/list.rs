use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use myhomelab_dashboard::entity::Dashboard;
use myhomelab_dashboard::repository::DashboardRepository;

#[tracing::instrument(skip_all)]
pub(super) async fn handle<S: crate::ServerState>(
    State(state): State<S>,
) -> Result<Json<Vec<Dashboard>>, StatusCode> {
    state
        .dashboard_repository()
        .list_dashboards()
        .await
        .map(Json)
        .map_err(|err| {
            tracing::error!(message = "unable to list dashboards", cause = ?err);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
