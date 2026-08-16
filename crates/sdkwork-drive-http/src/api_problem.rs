//! SDKWork HTTP ProblemDetail responses (`API_SPEC.md` section 15).

use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_security::DriveAuthError;
use serde::Serialize;

pub use sdkwork_utils_rust::SdkWorkResultCode;

/// RFC 9457 Problem Details with numeric platform `code` and `traceId`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SdkWorkProblemDetail {
    pub(crate) r#type: String,
    pub(crate) title: String,
    pub(crate) status: u16,
    pub(crate) detail: String,
    pub(crate) code: i32,
    pub(crate) trace_id: String,
}

pub fn problem(
    status: StatusCode,
    title: &str,
    detail: impl Into<String>,
    code: SdkWorkResultCode,
) -> (StatusCode, Json<SdkWorkProblemDetail>) {
    problem_with_trace_id(
        status,
        title,
        detail,
        code,
        crate::problem_correlation::current_problem_correlation().trace_id,
    )
}

pub fn problem_with_trace_id(
    status: StatusCode,
    title: &str,
    detail: impl Into<String>,
    code: SdkWorkResultCode,
    trace_id: impl Into<String>,
) -> (StatusCode, Json<SdkWorkProblemDetail>) {
    (
        status,
        Json(SdkWorkProblemDetail {
            r#type: "about:blank".to_string(),
            title: title.to_string(),
            status: status.as_u16(),
            detail: detail.into(),
            code: code.as_i32(),
            trace_id: trace_id.into(),
        }),
    )
}

pub fn map_auth_error(error: DriveAuthError) -> (StatusCode, Json<SdkWorkProblemDetail>) {
    let status = StatusCode::from_u16(error.status).unwrap_or(StatusCode::UNAUTHORIZED);
    (
        status,
        Json(SdkWorkProblemDetail {
            r#type: "about:blank".to_string(),
            title: error.title.to_string(),
            status: error.status,
            detail: error.detail,
            code: error.code,
            trace_id: error.trace_id,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problem_uses_numeric_platform_code() {
        let (_, Json(problem)) = problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "detail",
            SdkWorkResultCode::ValidationError,
        );
        assert_eq!(problem.code, 40001);
    }
}
