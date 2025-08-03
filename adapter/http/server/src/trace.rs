use std::time::Duration;

use axum::http::{Request, Response};
use tower_http::trace::{
    DefaultOnRequest, DefaultOnResponse, HttpMakeClassifier, MakeSpan, OnResponse, TraceLayer,
};
use tracing::Span;

pub(crate) fn layer()
-> TraceLayer<HttpMakeClassifier, MakeSpanWithHttpProps, DefaultOnRequest, OnResponseWithStatus> {
    TraceLayer::new_for_http()
        .make_span_with(MakeSpanWithHttpProps)
        .on_response(OnResponseWithStatus::default())
}

#[derive(Clone, Debug, Default)]
pub struct MakeSpanWithHttpProps;

impl<B> MakeSpan<B> for MakeSpanWithHttpProps {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        tracing::span!(
            tracing::Level::DEBUG,
            "request",
            resource.name = "http request",
            http.uri = %request.uri(),
            http.method = %request.method(),
            http.version = ?request.version(),
            http.status_code = tracing::field::Empty,
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct OnResponseWithStatus(DefaultOnResponse);

impl<B> OnResponse<B> for OnResponseWithStatus {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        self.0.on_response(response, latency, span);
        span.record(
            "http.status_code",
            &tracing::field::display(response.status().as_u16()),
        );
        span.record(
            "otel.status_code",
            &tracing::field::display(response.status().as_u16()),
        );
    }
}
