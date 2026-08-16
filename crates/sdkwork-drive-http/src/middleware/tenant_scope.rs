use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Header name for tenant ID.
pub const TENANT_ID_HEADER: &str = "X-Tenant-Id";

/// Middleware that validates tenant scope on protected routes.
///
/// This middleware checks that a valid tenant ID is present in the request
/// and adds it to request extensions for downstream handlers.
pub async fn tenant_scope_middleware(mut request: Request, next: Next) -> Response {
    let tenant_id = request
        .headers()
        .get(TENANT_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    match tenant_id {
        Some(id) => {
            request.extensions_mut().insert(TenantId(id));
            next.run(request).await
        }
        None => (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "error": {
                    "code": "MISSING_TENANT_ID",
                    "message": "X-Tenant-Id header is required"
                }
            })),
        )
            .into_response(),
    }
}

/// Tenant ID extracted from request extensions.
#[derive(Debug, Clone)]
pub struct TenantId(pub String);

impl TenantId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
