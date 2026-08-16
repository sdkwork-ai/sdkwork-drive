use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http::{header, HeaderMap, HeaderValue};
use sdkwork_drive_security::DriveAppContext;
use sdkwork_drive_security::TRACE_ID_HEADER;
use sdkwork_web_core::WebRequestContext;

use crate::trace_ids::resolve_trace_id;

pub const UNSET_REQUEST_ID: &str = "request-unset";
pub const UNSET_TRACE_ID: &str = "trace-unset";

#[derive(Debug, Clone)]
pub struct ProblemCorrelationIds {
    pub request_id: String,
    pub trace_id: String,
}

impl ProblemCorrelationIds {
    pub fn new(request_id: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            trace_id: trace_id.into(),
        }
    }
}

pub async fn problem_correlation_middleware(request: Request, next: Next) -> Response {
    let ids = correlation_ids_from_request(&request);
    let traceparent = optional_header(request.headers(), "traceparent");
    let mut response = with_problem_correlation(ids.clone(), next.run(request)).await;
    normalize_problem_content_type(&mut response);
    inject_correlation_response_headers(response.headers_mut(), &ids, traceparent.as_deref());
    response
}

pub async fn with_problem_correlation<T>(
    ids: ProblemCorrelationIds,
    operation: impl std::future::Future<Output = T>,
) -> T {
    PROBLEM_CORRELATION.scope(ids, operation).await
}

pub fn current_problem_correlation() -> ProblemCorrelationIds {
    PROBLEM_CORRELATION
        .try_with(|ids| ids.clone())
        .unwrap_or_else(|_| ProblemCorrelationIds {
            request_id: UNSET_REQUEST_ID.to_string(),
            trace_id: UNSET_TRACE_ID.to_string(),
        })
}

pub fn correlation_ids_from_request(request: &Request) -> ProblemCorrelationIds {
    if let Some(ids) = request.extensions().get::<ProblemCorrelationIds>() {
        return ids.clone();
    }
    if let Some(ctx) = request.extensions().get::<DriveAppContext>() {
        return ProblemCorrelationIds {
            request_id: ctx.request_id.clone(),
            trace_id: if ctx.trace_id.trim().is_empty() {
                resolve_trace_id(request.headers(), &ctx.request_id)
            } else {
                ctx.trace_id.clone()
            },
        };
    }
    if let Some(ctx) = request.extensions().get::<WebRequestContext>() {
        let request_id = ctx.request_id.0.clone();
        return ProblemCorrelationIds {
            request_id: request_id.clone(),
            trace_id: resolve_trace_id(request.headers(), &request_id),
        };
    }
    ProblemCorrelationIds {
        request_id: UNSET_REQUEST_ID.to_string(),
        trace_id: UNSET_TRACE_ID.to_string(),
    }
}

tokio::task_local! {
    static PROBLEM_CORRELATION: ProblemCorrelationIds;
}

fn optional_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn inject_correlation_response_headers(
    headers: &mut HeaderMap,
    ids: &ProblemCorrelationIds,
    traceparent: Option<&str>,
) {
    if let Ok(value) = HeaderValue::from_str(&ids.trace_id) {
        headers.insert(TRACE_ID_HEADER, value);
    }
    if let Some(traceparent) = traceparent {
        if let Ok(value) = HeaderValue::from_str(traceparent) {
            headers.insert("traceparent", value);
        }
    }
}

fn normalize_problem_content_type(response: &mut Response) {
    if !response.status().is_client_error() && !response.status().is_server_error() {
        return;
    }
    let is_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value.eq_ignore_ascii_case("application/json")
                || value.to_ascii_lowercase().starts_with("application/json;")
        });
    if is_json {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;

    #[test]
    fn injects_correlation_trace_headers() {
        let mut headers = HeaderMap::new();
        inject_correlation_response_headers(
            &mut headers,
            &ProblemCorrelationIds::new("request-001", "trace-001"),
            Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
        );
        assert_eq!(
            headers
                .get(TRACE_ID_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("trace-001")
        );
        assert_eq!(
            headers
                .get("traceparent")
                .and_then(|value| value.to_str().ok()),
            Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
        );
    }

    #[test]
    fn correlation_ids_prefer_drive_app_context() {
        let mut request = Request::new(Body::empty());
        request.extensions_mut().insert(DriveAppContext {
            tenant_id: "tenant-001".to_string(),
            user_id: "user-001".to_string(),
            organization_id: None,
            session_id: None,
            app_id: Some("appbase".to_string()),
            environment: Some("dev".to_string()),
            deployment_mode: Some("saas".to_string()),
            auth_level: Some("password".to_string()),
            data_scope: Vec::new(),
            permission_scope: Vec::new(),
            actor_id: "user-001".to_string(),
            actor_kind: "user".to_string(),
            device_id: None,
            request_id: "request-abc".to_string(),
            trace_id: "trace-xyz".to_string(),
        });

        let ids = correlation_ids_from_request(&request);
        assert_eq!(ids.request_id, "request-abc");
        assert_eq!(ids.trace_id, "trace-xyz");
    }

    #[test]
    fn normalizes_json_error_responses_to_problem_json() {
        let mut response = Response::builder()
            .status(http::StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .body(Body::empty())
            .expect("response should be built");

        normalize_problem_content_type(&mut response);

        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/problem+json")
        );
    }

    #[test]
    fn leaves_success_json_content_type_unchanged() {
        let mut response = Response::builder()
            .status(http::StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::empty())
            .expect("response should be built");

        normalize_problem_content_type(&mut response);

        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
    }
}
