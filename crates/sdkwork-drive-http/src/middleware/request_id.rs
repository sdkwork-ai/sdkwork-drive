use axum::{extract::Request, middleware::Next, response::Response};
use sdkwork_utils_rust::uuid;

/// Middleware that ensures every request has a request ID.
///
/// The ID is always server-owned and is stored only in request extensions.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = uuid();

    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    next.run(request).await
}

/// Request ID extracted from request extensions.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::extract::Extension;
    use axum::middleware;
    use axum::routing::get;
    use axum::Router;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn echo_request_id(Extension(request_id): Extension<RequestId>) -> String {
        request_id.0
    }

    #[tokio::test]
    async fn middleware_ignores_client_request_id_and_does_not_expose_legacy_header() {
        let app = Router::new()
            .route("/probe", get(echo_request_id))
            .layer(middleware::from_fn(request_id_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .header("X-Request-Id", "client-forged-request-id")
                    .body(Body::empty())
                    .expect("request should be built"),
            )
            .await
            .expect("request should be handled");

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers().get("X-Request-Id").is_none(),
            "Drive-owned middleware must not expose legacy X-Request-Id"
        );

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let request_id = String::from_utf8(body.to_vec()).expect("request id should be utf8");
        assert_ne!(request_id, "client-forged-request-id");
        assert_eq!(request_id.len(), 36);
    }
}
