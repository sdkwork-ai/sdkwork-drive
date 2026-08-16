use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(default_service_name: &'static str) {
    let service_name = resolve_service_name(default_service_name);
    if std::env::var("OTEL_SERVICE_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_none()
    {
        std::env::set_var("OTEL_SERVICE_NAME", &service_name);
    }

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| {
            EnvFilter::try_new(
                std::env::var("SDKWORK_DRIVE_LOG")
                    .unwrap_or_else(|_| "info,sdkwork=debug".to_string()),
            )
        })
        .unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(feature = "otlp")]
    if otlp_exporter_enabled() {
        init_with_otlp(default_service_name, filter);
        return;
    }

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn resolve_service_name(default_service_name: &str) -> String {
    std::env::var("OTEL_SERVICE_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_service_name.to_string())
}

pub fn otlp_exporter_enabled() -> bool {
    otlp_endpoint().is_some()
}

pub fn otlp_endpoint() -> Option<String> {
    std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("SDKWORK_DRIVE_OTEL_EXPORTER_OTLP_ENDPOINT")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}

#[cfg(feature = "otlp")]
fn init_with_otlp(default_service_name: &'static str, filter: EnvFilter) {
    use opentelemetry::global;
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use opentelemetry_sdk::Resource;
    use tracing_opentelemetry::OpenTelemetryLayer;

    global::set_text_map_propagator(TraceContextPropagator::new());

    let endpoint = otlp_endpoint().expect("otlp endpoint must be configured");
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .build()
        .expect("build otlp span exporter");

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name(resolve_service_name(default_service_name))
                .build(),
        )
        .build();

    let tracer = provider.tracer(default_service_name);
    global::set_tracer_provider(provider);

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_service_name_from_default() {
        assert_eq!(
            resolve_service_name("sdkwork-routes-drive-app-api"),
            "sdkwork-routes-drive-app-api"
        );
    }

    #[test]
    fn otlp_endpoint_prefers_standard_otel_env() {
        std::env::set_var(
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            "http://otel-collector:4318/v1/traces",
        );
        std::env::remove_var("SDKWORK_DRIVE_OTEL_EXPORTER_OTLP_ENDPOINT");
        assert_eq!(
            otlp_endpoint().as_deref(),
            Some("http://otel-collector:4318/v1/traces")
        );
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    }
}
