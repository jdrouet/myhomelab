use axum::extract::State;
use axum::response::Html;
use myhomelab_client_web::home::HomePage;
use myhomelab_client_web::page::PageWrapper;

use super::ServerContext;
use crate::ServerState;

#[tracing::instrument(skip_all)]
pub(super) async fn handle<S: ServerState>(State(state): State<S>) -> Html<String> {
    let home = HomePage::default();
    let mut buffer = String::with_capacity(1024);
    match PageWrapper::new(home)
        .render(&ServerContext(state), &mut buffer)
        .await
    {
        Ok(_) => Html(buffer),
        Err(err) => Html(err.to_string()),
    }
}
