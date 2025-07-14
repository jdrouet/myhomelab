use axum::extract::{Path, State};
use axum::response::Html;
use myhomelab_client_web::page::PageWrapper;
use myhomelab_client_web::page::dashboard::DashboardPage;
use myhomelab_dashboard::repository::DashboardRepository;
use myhomelab_prelude::time::TimeRange;
use serde_qs::web::QsQuery;
use uuid::Uuid;

use super::ServerContext;
use crate::ServerState;

#[derive(Debug, serde::Deserialize)]
pub struct QueryParams {
    #[serde(default)]
    timerange: TimeRange,
}

#[tracing::instrument(skip_all)]
pub(super) async fn handle<S: ServerState>(
    State(state): State<S>,
    Path(dashboard_id): Path<Uuid>,
    QsQuery(params): QsQuery<QueryParams>,
) -> Html<String> {
    let dashboard = match state
        .dashboard_repository()
        .find_dashboard_by_id(dashboard_id)
        .await
    {
        Ok(Some(dashboard)) => dashboard,
        Ok(None) => return Html("Dashboard not found...".into()),
        Err(err) => return Html(err.to_string()),
    };
    let home = DashboardPage::new(dashboard, params.timerange);
    let mut buffer = String::with_capacity(1024);
    match PageWrapper::new(home)
        .render(&ServerContext(state), &mut buffer)
        .await
    {
        Ok(_) => Html(buffer),
        Err(err) => Html(err.to_string()),
    }
}
