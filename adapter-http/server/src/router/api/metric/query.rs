use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use myhomelab_adapter_http_shared::metric::query::QueryParams;
use myhomelab_metric::query::{QueryExecutor, Response};

pub(super) async fn handle<S: crate::ServerState>(
    State(state): State<S>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Vec<Response>>, StatusCode> {
    let executor = state.metric_query_executor();
    match executor.execute(params.requests, params.range).await {
        Ok(list) => Ok(Json(list)),
        Err(err) => {
            tracing::error!(message = "unable to query metrics", cause = ?err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
