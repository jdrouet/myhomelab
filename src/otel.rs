use std::borrow::Cow;

use opentelemetry::{InstrumentationScope, KeyValue, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace::BatchSpanProcessor};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
pub struct OtelConfig {
    pub endpoint: Cow<'static, str>,
    pub environment: Cow<'static, str>,
    pub inner_level: Cow<'static, str>,
    pub service_name: Cow<'static, str>,
    pub service_version: Cow<'static, str>,
}

impl crate::Configurable for OtelConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            endpoint: std::env::var("OTEL_COLLECTOR_ENDPOINT")
                .ok()
                .map(Cow::Owned)
                .unwrap_or(Cow::Borrowed("http://localhost:4317")),
            environment: std::env::var("ENV")
                .ok()
                .map(Cow::Owned)
                .unwrap_or(Cow::Borrowed("local")),
            inner_level: std::env::var("OTEL_INNER_LEVEL")
                .ok()
                .map(Cow::Owned)
                .unwrap_or(Cow::Borrowed("error")),
            service_name: std::env::var("SERVICE_NAME")
                .ok()
                .map(Cow::Owned)
                .unwrap_or(Cow::Borrowed(env!("CARGO_PKG_NAME"))),
            service_version: std::env::var("SERVICE_VERSION")
                .ok()
                .map(Cow::Owned)
                .unwrap_or(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
        })
    }
}

impl OtelConfig {
    fn inner_filter(&self) -> tracing_subscriber::EnvFilter {
        let level = self.inner_level.as_ref();
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(format!("h2={level}").parse().unwrap())
            .add_directive(format!("hyper_util={level}").parse().unwrap())
            .add_directive(format!("opentelemetry={level}").parse().unwrap())
            .add_directive(format!("reqwest={level}").parse().unwrap())
            .add_directive(format!("tonic={level}").parse().unwrap())
            .add_directive(format!("tower={level}").parse().unwrap())
    }

    fn resources(&self) -> Resource {
        use opentelemetry_semantic_conventions::resource;

        Resource::builder()
            .with_attribute(KeyValue::new(
                resource::SERVICE_NAME,
                self.service_name.to_string(),
            ))
            .with_attribute(KeyValue::new(
                "deployment.environment",
                self.environment.clone(),
            ))
            .build()
    }

    fn setup_metrics(&self) -> anyhow::Result<()> {
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_protocol(opentelemetry_otlp::Protocol::Grpc)
            .with_endpoint(self.endpoint.to_string())
            .build()?;

        let provider = opentelemetry_sdk::metrics::MeterProviderBuilder::default()
            .with_periodic_exporter(exporter)
            .with_resource(self.resources())
            .build();

        opentelemetry::global::set_meter_provider(provider);

        Ok(())
    }

    fn setup_traces(&self) -> anyhow::Result<()> {
        let span_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_protocol(opentelemetry_otlp::Protocol::Grpc)
            .with_endpoint(self.endpoint.to_string())
            .build()?;

        let span_processor = BatchSpanProcessor::builder(span_exporter).build();

        let tracer_provider = opentelemetry_sdk::trace::TracerProviderBuilder::default()
            .with_span_processor(span_processor)
            .with_resource(self.resources())
            .build();

        let scope = InstrumentationScope::builder(self.service_name.to_string())
            .with_version(self.service_version.to_string())
            .with_schema_url(opentelemetry_semantic_conventions::SCHEMA_URL)
            .with_attributes(None)
            .build();
        let tracer = tracer_provider.tracer_with_scope(scope);

        opentelemetry::global::set_tracer_provider(tracer_provider);

        let telemetry = OpenTelemetryLayer::new(tracer);

        let log_exporter = opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_protocol(opentelemetry_otlp::Protocol::Grpc)
            .with_endpoint(self.endpoint.to_string())
            .build()?;

        let log_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
            .with_resource(self.resources())
            .with_batch_exporter(log_exporter)
            .build();

        let otel_layer =
            opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&log_provider);

        tracing_subscriber::registry()
            .with(self.inner_filter())
            .with(telemetry)
            .with(otel_layer)
            .with(tracing_subscriber::fmt::layer())
            .try_init()?;

        Ok(())
    }

    pub fn install(&self) -> anyhow::Result<()> {
        self.setup_metrics()?;
        self.setup_traces()?;
        Ok(())
    }
}
