use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_http::api_problem::{problem as shared_problem, SdkWorkProblemDetail};
use sdkwork_drive_observability::error_kinds;
use sdkwork_drive_storage_contract::{DriveObjectStoreError, DriveObjectStoreErrorKind};
use sdkwork_drive_workspace_service::DriveServiceError;

pub(crate) use sdkwork_drive_http::api_problem::SdkWorkResultCode;

pub(crate) type ProblemDetail = SdkWorkProblemDetail;

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

pub(crate) fn map_download_token_error(
    error: DriveServiceError,
) -> (StatusCode, Json<ProblemDetail>) {
    match error {
        DriveServiceError::NotFound(detail) if detail.contains("expired") => problem(
            StatusCode::GONE,
            "download token expired",
            detail,
            SdkWorkResultCode::Gone,
        ),
        other => map_service_error(other),
    }
}

pub(crate) fn share_link_expired_problem() -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::GONE,
        "share link expired",
        "share link expired",
        SdkWorkResultCode::Gone,
    )
}

pub(crate) fn share_link_download_limit_problem() -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::TOO_MANY_REQUESTS,
        "download limit exceeded",
        "share link download limit exceeded",
        SdkWorkResultCode::RateLimitExceeded,
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

/// Checks whether a `sqlx::Error` is a unique-constraint violation.
///
/// Uses the sqlx `Error::as_database_error()` API when available, falling back
/// to error message inspection for engines that do not provide structured codes.
pub(crate) fn is_unique_constraint_error(error: &sqlx::Error) -> bool {
    if let Some(database_error) = error.as_database_error() {
        if let Some(code) = database_error.code() {
            if code.as_ref() == "23505" {
                return true;
            }
        }
        if let Some(constraint) = database_error.constraint() {
            return !constraint.is_empty();
        }
    }

    let message = error.to_string();
    message.contains("UNIQUE constraint failed")
        || message.contains("duplicate key value violates unique constraint")
}

pub(crate) fn unique_node_insert_conflict_target(error: &sqlx::Error) -> &'static str {
    if !is_unique_constraint_error(error) {
        return "unknown";
    }

    if let Some(database_error) = error.as_database_error() {
        if let Some(constraint) = database_error.constraint() {
            return match constraint {
                "dr_drive_node_pkey" | "ux_dr_drive_node_pkey" => "id",
                "ux_dr_drive_node_root_name_live" | "ux_dr_drive_node_child_name_live" => "name",
                _ => "unknown",
            };
        }
    }

    let message = error.to_string();
    if message.contains("dr_drive_node_pkey")
        || message.contains("ux_dr_drive_node_pkey")
        || (message.contains("UNIQUE constraint failed: dr_drive_node.id")
            || message.contains("unique constraint \"dr_drive_node_pkey\""))
    {
        return "id";
    }

    if message.contains("ux_dr_drive_node_root_name_live")
        || message.contains("ux_dr_drive_node_child_name_live")
    {
        return "name";
    }

    if message.contains("parent_node_id") || message.contains("node_name") {
        return "name";
    }

    "unknown"
}

pub(crate) fn not_found_problem(detail: impl Into<String>) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::NOT_FOUND,
        "not found",
        detail,
        SdkWorkResultCode::NotFound,
    )
}

pub(crate) fn validation_problem(detail: impl Into<String>) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        detail,
        SdkWorkResultCode::ValidationError,
    )
}

pub(crate) fn invalid_parameter_problem(
    detail: impl Into<String>,
) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "invalid parameter",
        detail,
        SdkWorkResultCode::InvalidParameter,
    )
}

pub(crate) fn missing_required_field_problem(
    detail: impl Into<String>,
) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "missing required field",
        detail,
        SdkWorkResultCode::MissingRequiredField,
    )
}

pub(crate) fn malformed_request_problem(
    detail: impl Into<String>,
) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "malformed request",
        detail,
        SdkWorkResultCode::MalformedRequest,
    )
}

pub(crate) fn precondition_required_problem(
    detail: impl Into<String>,
) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::PRECONDITION_REQUIRED,
        "precondition required",
        detail,
        SdkWorkResultCode::PreconditionRequired,
    )
}

pub(crate) fn precondition_failed_problem(
    detail: impl Into<String>,
) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::PRECONDITION_FAILED,
        "precondition failed",
        detail,
        SdkWorkResultCode::PreconditionFailed,
    )
}

pub(crate) fn payload_too_large_problem(
    detail: impl Into<String>,
) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::PAYLOAD_TOO_LARGE,
        "payload too large",
        detail,
        SdkWorkResultCode::PayloadTooLarge,
    )
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

pub(crate) fn status_error_kind(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => error_kinds::VALIDATION,
        StatusCode::FORBIDDEN => error_kinds::PERMISSION_DENIED,
        StatusCode::NOT_FOUND => error_kinds::NOT_FOUND,
        StatusCode::CONFLICT => error_kinds::CONFLICT,
        _ => error_kinds::INTERNAL,
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

pub(crate) fn map_object_store_error(error: DriveObjectStoreError) -> DriveServiceError {
    match error.kind {
        DriveObjectStoreErrorKind::NotFound => DriveServiceError::NotFound(error.message),
        DriveObjectStoreErrorKind::InvalidRequest => DriveServiceError::Validation(error.message),
        DriveObjectStoreErrorKind::Conflict => DriveServiceError::Conflict(error.message),
        DriveObjectStoreErrorKind::PermissionDenied => {
            DriveServiceError::PermissionDenied(error.message)
        }
        _ => DriveServiceError::Internal(error.message),
    }
}

pub(crate) fn map_object_store_route_error(
    error: DriveObjectStoreError,
) -> (StatusCode, Json<ProblemDetail>) {
    map_service_error(map_object_store_error(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use sdkwork_drive_http::problem_correlation::{
        current_problem_correlation, with_problem_correlation, ProblemCorrelationIds,
        UNSET_REQUEST_ID, UNSET_TRACE_ID,
    };

    #[test]
    fn internal_problem_does_not_expose_internal_detail_to_clients() {
        let (_, Json(problem)) = internal_problem(
            "insert dr_drive_node failed: duplicate key value violates unique constraint",
        );
        let json = serde_json::to_value(problem).expect("serialize problem");
        assert_eq!(json["status"], StatusCode::INTERNAL_SERVER_ERROR.as_u16());
        assert_eq!(json["detail"], "An unexpected error occurred.");
        assert_eq!(json["code"], SdkWorkResultCode::InternalError.as_i32());
    }

    #[test]
    fn problem_uses_task_local_correlation_ids_when_scoped() {
        let result = tokio::runtime::Runtime::new()
            .expect("runtime")
            .block_on(async {
                with_problem_correlation(
                    ProblemCorrelationIds::new("request-test", "trace-test"),
                    async {
                        let (status, Json(problem)) = problem(
                            StatusCode::BAD_REQUEST,
                            "validation failed",
                            "detail",
                            SdkWorkResultCode::ValidationError,
                        );
                        (status, problem)
                    },
                )
                .await
            });

        let (status, problem) = result;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json = serde_json::to_value(problem).expect("serialize problem");
        assert_eq!(json["traceId"], "trace-test");
        let ids = current_problem_correlation();
        assert_eq!(ids.request_id, UNSET_REQUEST_ID);
        assert_eq!(ids.trace_id, UNSET_TRACE_ID);
    }
}
