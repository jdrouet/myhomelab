use std::borrow::Cow;

use opentelemetry::trace::TracerProvider;
use opentelemetry::{InstrumentationScope, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::BatchSpanProcessor;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, serde::Deserialize)]
pub struct OpenTelemetryConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "OpenTelemetryConfig::default_ansi")]
    ansi: bool,
    #[serde(default = "OpenTelemetryConfig::default_service_name")]
    service_name: Cow<'static, str>,
    #[serde(default = "OpenTelemetryConfig::default_environment")]
    environment: Cow<'static, str>,
    #[serde(default = "OpenTelemetryConfig::default_endpoint")]
    endpoint: Cow<'static, str>,
}

impl Default for OpenTelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ansi: Self::default_ansi(),
            service_name: Self::default_service_name(),
            environment: Self::default_environment(),
            endpoint: Self::default_endpoint(),
        }
    }
}

impl OpenTelemetryConfig {
    const fn default_ansi() -> bool {
        true
    }

    const fn default_service_name() -> Cow<'static, str> {
        Cow::Borrowed(env!("CARGO_PKG_NAME"))
    }

    fn default_environment() -> Cow<'static, str> {
        std::env::var("ENV")
            .map(Cow::Owned)
            .unwrap_or_else(|_| "development".into())
    }

    const fn default_endpoint() -> Cow<'static, str> {
        Cow::Borrowed("http://localhost:4317")
    }
}

impl OpenTelemetryConfig {
    fn resources(&self) -> Resource {
        use opentelemetry_semantic_conventions::resource;

        Resource::builder()
            .with_attribute(KeyValue::new(
                resource::SERVICE_NAME,
                self.service_name.to_owned(),
            ))
            .with_attribute(KeyValue::new(
                "deployment.environment",
                self.environment.to_owned(),
            ))
            .build()
    }

    fn setup_metric(&self) -> anyhow::Result<()> {
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_protocol(opentelemetry_otlp::Protocol::Grpc)
            .with_endpoint(self.endpoint.to_owned())
            .build()?;

        let provider = opentelemetry_sdk::metrics::MeterProviderBuilder::default()
            .with_periodic_exporter(exporter)
            .with_resource(self.resources())
            .build();

        opentelemetry::global::set_meter_provider(provider);

        Ok(())
    }

    fn setup_tracing(&self) -> anyhow::Result<()> {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_protocol(opentelemetry_otlp::Protocol::Grpc)
            .with_endpoint(self.endpoint.to_owned())
            .build()?;

        let span_processor = BatchSpanProcessor::builder(exporter).build();

        let provider = opentelemetry_sdk::trace::TracerProviderBuilder::default()
            .with_span_processor(span_processor)
            .with_resource(self.resources())
            .build();

        let scope = InstrumentationScope::builder(self.service_name.clone())
            .with_version(env!("CARGO_PKG_VERSION"))
            .with_schema_url(opentelemetry_semantic_conventions::SCHEMA_URL)
            .with_attributes(None)
            .build();
        let tracer = provider.tracer_with_scope(scope);

        opentelemetry::global::set_tracer_provider(provider);

        let telemetry = OpenTelemetryLayer::new(tracer);
        tracing_subscriber::registry()
            .with(telemetry)
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .try_init()?;
        Ok(())
    }

    fn setup_basic(&self) -> anyhow::Result<()> {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer().with_ansi(self.ansi))
            .try_init()?;
        Ok(())
    }

    pub fn setup(self) -> anyhow::Result<()> {
        if self.enabled {
            self.setup_metric()?;
            self.setup_tracing()?;
        } else {
            self.setup_basic()?;
        }
        Ok(())
    }
}
