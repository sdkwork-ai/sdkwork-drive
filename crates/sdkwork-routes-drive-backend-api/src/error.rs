use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_http::api_problem::{
    map_auth_error as shared_map_auth_error, problem as shared_problem, SdkWorkProblemDetail,
};
use sdkwork_drive_observability::error_kinds;
use sdkwork_drive_security::DriveAuthError;
use sdkwork_drive_workspace_service::DriveServiceError;

pub(crate) type ProblemDetail = SdkWorkProblemDetail;

pub(crate) use sdkwork_drive_http::api_problem::SdkWorkResultCode;

pub(crate) fn validation_problem(detail: impl Into<String>) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        detail,
        SdkWorkResultCode::ValidationError,
    )
}

pub(crate) fn invalid_json_problem(_rejection: JsonRejection) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "invalid request body",
        "request body must be valid JSON matching the operation schema",
        SdkWorkResultCode::MalformedRequest,
    )
}

pub(crate) fn invalid_query_problem(
    _rejection: QueryRejection,
) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "invalid query parameters",
        "query parameters do not match the operation schema",
        SdkWorkResultCode::InvalidParameter,
    )
}

pub(crate) fn internal_problem(detail: impl Into<String>) -> (StatusCode, Json<ProblemDetail>) {
    let detail = detail.into();
    tracing::error!(
        target: "sdkwork.drive",
        detail = %detail,
        "internal error response"
    );
    problem(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal error",
        "An unexpected error occurred.",
        SdkWorkResultCode::InternalError,
    )
}

pub(crate) fn internal_sql_error(
    prefix: &'static str,
) -> impl Fn(sqlx::Error) -> (StatusCode, Json<ProblemDetail>) {
    move |error| {
        tracing::error!(
            target: "sdkwork.drive",
            prefix = prefix,
            error = %error,
            "database operation failed"
        );
        internal_problem("A database operation failed.")
    }
}

pub(crate) fn not_found_problem(detail: impl Into<String>) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::NOT_FOUND,
        "not found",
        detail,
        SdkWorkResultCode::NotFound,
    )
}

pub(crate) fn map_service_error(error: DriveServiceError) -> (StatusCode, Json<ProblemDetail>) {
    match error {
        DriveServiceError::Validation(detail) => {
            let code = if detail.starts_with("provider_kind is invalid;") {
                SdkWorkResultCode::InvalidParameter
            } else {
                SdkWorkResultCode::ValidationError
            };
            problem(StatusCode::BAD_REQUEST, "validation failed", detail, code)
        }
        DriveServiceError::Conflict(detail) => problem(
            StatusCode::CONFLICT,
            "conflict",
            detail,
            SdkWorkResultCode::Conflict,
        ),
        DriveServiceError::NotFound(detail) => problem(
            StatusCode::NOT_FOUND,
            "not found",
            detail,
            SdkWorkResultCode::NotFound,
        ),
        DriveServiceError::PermissionDenied(detail) => problem(
            StatusCode::FORBIDDEN,
            "permission denied",
            detail,
            SdkWorkResultCode::PermissionRequired,
        ),
        DriveServiceError::Internal(detail) => {
            tracing::error!(
                target: "sdkwork.drive",
                detail = %detail,
                "internal drive service error"
            );
            problem(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error",
                "An unexpected error occurred.",
                SdkWorkResultCode::InternalError,
            )
        }
    }
}

pub(crate) fn service_error_kind(error: &DriveServiceError) -> &'static str {
    match error {
        DriveServiceError::Validation(_) => error_kinds::VALIDATION,
        DriveServiceError::Conflict(_) => error_kinds::CONFLICT,
        DriveServiceError::NotFound(_) => error_kinds::NOT_FOUND,
        DriveServiceError::PermissionDenied(_) => error_kinds::PERMISSION_DENIED,
        DriveServiceError::Internal(_) => error_kinds::INTERNAL,
    }
}

pub(crate) fn problem(
    status: StatusCode,
    title: &str,
    detail: impl Into<String>,
    code: SdkWorkResultCode,
) -> (StatusCode, Json<ProblemDetail>) {
    shared_problem(status, title, detail, code)
}

pub(crate) fn map_auth_error(error: DriveAuthError) -> (StatusCode, Json<ProblemDetail>) {
    shared_map_auth_error(error)
}
