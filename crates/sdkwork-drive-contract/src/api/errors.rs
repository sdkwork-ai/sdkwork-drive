use serde::{Deserialize, Serialize};

/// Pagination request parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page_size: Option<i64>,
    pub page_token: Option<String>,
}

/// Pagination response metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total_count: Option<i64>,
    pub page_size: i64,
    pub next_page_token: Option<String>,
}

/// Problem detail response (RFC 9457).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetail {
    #[serde(rename = "type")]
    pub problem_type: Option<String>,
    pub title: Option<String>,
    pub status: i32,
    pub detail: Option<String>,
    pub instance: Option<String>,
    #[serde(flatten)]
    pub extensions: std::collections::BTreeMap<String, serde_json::Value>,
}

/// Standard error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

/// Error body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
}

/// Field-level validation error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}
