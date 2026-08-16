use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_http::api_problem::{
    map_auth_error as shared_map_auth_error, problem as shared_problem, SdkWorkProblemDetail,
};
use sdkwork_drive_security::DriveAuthError;
use sdkwork_drive_storage_contract::{DriveObjectStoreError, DriveObjectStoreErrorKind};
use sdkwork_drive_workspace_service::DriveServiceError;

pub(crate) type ProblemDetail = SdkWorkProblemDetail;

pub(crate) use sdkwork_drive_http::api_problem::SdkWorkResultCode;

pub(crate) fn not_found_binding_problem() -> (StatusCode, Json<ProblemDetail>) {
    map_service_error(DriveServiceError::NotFound(
        "default storage provider binding not found".to_string(),
    ))
}

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

pub(crate) fn map_object_store_route_error(
    error: DriveObjectStoreError,
) -> (StatusCode, Json<ProblemDetail>) {
    map_service_error(match error.kind {
        DriveObjectStoreErrorKind::NotFound => DriveServiceError::NotFound(error.message),
        DriveObjectStoreErrorKind::InvalidRequest => DriveServiceError::Validation(error.message),
        DriveObjectStoreErrorKind::Conflict => DriveServiceError::Conflict(error.message),
        DriveObjectStoreErrorKind::PermissionDenied => {
            DriveServiceError::PermissionDenied(error.message)
        }
        DriveObjectStoreErrorKind::NotSupported => DriveServiceError::Conflict(error.message),
        // 读到的字节数与 head 声明不一致（对象在读取期间被并发改写）：
        // 映射为冲突而不是 500，调用方可重试而非误判为服务故障。
        DriveObjectStoreErrorKind::IntegrityFailed => DriveServiceError::Conflict(error.message),
        _ => DriveServiceError::Internal(error.message),
    })
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
        DriveServiceError::Internal(detail) => problem(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error",
            detail,
            SdkWorkResultCode::InternalError,
        ),
    }
}

pub(crate) fn map_auth_error(error: DriveAuthError) -> (StatusCode, Json<ProblemDetail>) {
    shared_map_auth_error(error)
}

pub(crate) fn problem(
    status: StatusCode,
    title: &str,
    detail: impl Into<String>,
    code: SdkWorkResultCode,
) -> (StatusCode, Json<ProblemDetail>) {
    shared_problem(status, title, detail, code)
}
