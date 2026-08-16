use std::time::Instant;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sdkwork_drive_observability::metrics;
use tracing::Instrument;

fn span_route_template(path: &str) -> &'static str {
    if path.starts_with("/open/v3/api/drive/share_links/") && path.ends_with("/download_url") {
        return "/open/v3/api/drive/share_links/{token}/download_url";
    }
    if path.starts_with("/open/v3/api/drive/share_links/") {
        return "/open/v3/api/drive/share_links/{token}";
    }
    if path.starts_with("/app/v3/api/drive/nodes/") && path.contains("/share_links") {
        return "/app/v3/api/drive/nodes/{nodeId}/share_links";
    }
    if path.starts_with("/app/v3/api/drive/") {
        return "/app/v3/api/drive";
    }
    if path.starts_with("/backend/v3/api/drive/") {
        return "/backend/v3/api/drive";
    }
    if path == "/healthz" {
        return "/healthz";
    }
    if path == "/readyz" {
        return "/readyz";
    }
    if path == "/metrics" {
        return "/metrics";
    }
    "other"
}

fn infer_api_surface_label(route: &str) -> &'static str {
    if route.starts_with("/app/") {
        "app-api"
    } else if route.starts_with("/backend/") {
        "backend-api"
    } else if route.starts_with("/open/") {
        "open-api"
    } else if route.starts_with("/admin/") {
        "admin-storage-api"
    } else {
        "unknown"
    }
}

pub async fn record_request_metrics(request: Request<Body>, next: Next) -> Response {
    let method = request.method().as_str().to_string();
    let route_template = span_route_template(request.uri().path()).to_string();
    let api_surface = infer_api_surface_label(&route_template);
    let span_method = method.clone();
    let span_route = route_template.clone();
    let deployment_profile = sdkwork_drive_config::resolve_deployment_profile_label();
    let runtime_profile = std::env::var("SDKWORK_DRIVE_RUNTIME_PROFILE")
        .unwrap_or_else(|_| "development".to_string());
    async move {
        let started = Instant::now();
        metrics::record_http_request();
        let response = next.run(request).await;
        metrics::record_http_request_duration(started.elapsed());
        metrics::record_http_request_route_labels(
            &method,
            &route_template,
            response.status().as_u16(),
            api_surface,
        );
        if response.status().is_server_error() {
            metrics::record_http_request_error();
        }
        response
    }
    .instrument(tracing::info_span!(
        "drive.http.request",
        otel.name = "HTTP",
        http.request.method = %span_method,
        http.route = %span_route,
        deployment.profile = %deployment_profile,
        runtime.profile = %runtime_profile,
    ))
    .await
}

pub async fn metrics_handler(service_name: &'static str) -> impl IntoResponse {
    metrics::set_health_serving(true);
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        metrics::render_prometheus(service_name),
    )
}

#[cfg(test)]
mod tests {
    use super::span_route_template;

    #[test]
    fn span_route_template_redacts_share_link_tokens() {
        assert_eq!(
            span_route_template("/open/v3/api/drive/share_links/secret-token-value"),
            "/open/v3/api/drive/share_links/{token}"
        );
        assert_eq!(
            span_route_template("/open/v3/api/drive/share_links/secret-token-value/download_url"),
            "/open/v3/api/drive/share_links/{token}/download_url"
        );
        assert_eq!(span_route_template("/healthz"), "/healthz");
    }
}
