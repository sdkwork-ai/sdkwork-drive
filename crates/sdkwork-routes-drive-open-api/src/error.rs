use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_http::api_problem::{problem as shared_problem, SdkWorkProblemDetail};
use sdkwork_drive_workspace_service::DriveServiceError;

pub(crate) use sdkwork_drive_http::api_problem::SdkWorkResultCode;

pub(crate) type ProblemDetail = SdkWorkProblemDetail;

pub(crate) fn share_link_expired_problem() -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::GONE,
        "share link expired",
        "share link expired",
        SdkWorkResultCode::Gone,
    )
}

pub(crate) fn share_link_access_code_problem() -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::FORBIDDEN,
        "access code required",
        "share link access code is missing or invalid",
        SdkWorkResultCode::PermissionRequired,
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
        problem(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error",
            "An unexpected error occurred.",
            SdkWorkResultCode::InternalError,
        )
    }
}

pub(crate) fn map_service_error(error: DriveServiceError) -> (StatusCode, Json<ProblemDetail>) {
    match error {
        DriveServiceError::Validation(detail) => problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            detail,
            SdkWorkResultCode::ValidationError,
        ),
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

pub(crate) fn problem(
    status: StatusCode,
    title: &str,
    detail: impl Into<String>,
    code: SdkWorkResultCode,
) -> (StatusCode, Json<ProblemDetail>) {
    shared_problem(status, title, detail, code)
}
