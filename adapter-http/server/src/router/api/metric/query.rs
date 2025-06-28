use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use myhomelab_adapter_http_shared::metric::query::{BatchQueryParams, SingleQueryParams};
use myhomelab_metric::query::{QueryExecutor, Response};
use serde_qs::web::QsQuery;

pub(super) async fn handle_batch<S: crate::ServerState>(
    State(state): State<S>,
    Json(params): Json<BatchQueryParams>,
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

pub(super) async fn handle_single<S: crate::ServerState>(
    State(state): State<S>,
    QsQuery(params): QsQuery<SingleQueryParams>,
) -> Result<Json<Vec<Response>>, StatusCode> {
    let params = BatchQueryParams::from(params);
    let executor = state.metric_query_executor();
    match executor.execute(params.requests.into(), params.range).await {
        Ok(list) => Ok(Json(list)),
        Err(err) => {
            tracing::error!(message = "unable to query metrics", cause = ?err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
