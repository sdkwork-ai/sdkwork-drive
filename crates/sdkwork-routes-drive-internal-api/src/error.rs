use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_http::api_problem::{problem, SdkWorkProblemDetail};
use sdkwork_drive_storage_contract::{DriveObjectStoreError, DriveObjectStoreErrorKind};
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_utils_rust::SdkWorkResultCode;

pub type RouteProblem = (StatusCode, Json<SdkWorkProblemDetail>);

pub fn invalid_parameter(detail: impl Into<String>) -> RouteProblem {
    problem(
        StatusCode::BAD_REQUEST,
        "invalid parameter",
        detail,
        SdkWorkResultCode::InvalidParameter,
    )
}

pub fn missing_internal_principal() -> RouteProblem {
    problem(
        StatusCode::UNAUTHORIZED,
        "authentication required",
        "authenticated internal principal is required",
        SdkWorkResultCode::AuthenticationRequired,
    )
}

pub fn forbidden_internal_caller() -> RouteProblem {
    problem(
        StatusCode::FORBIDDEN,
        "permission required",
        "the internal caller is not authorized for root-scope event delivery",
        SdkWorkResultCode::PermissionRequired,
    )
}

pub fn precondition_failed(detail: impl Into<String>) -> RouteProblem {
    problem(
        StatusCode::PRECONDITION_FAILED,
        "precondition failed",
        detail,
        SdkWorkResultCode::PreconditionFailed,
    )
}

pub fn range_not_satisfiable(content_length: u64) -> RouteProblem {
    problem(
        StatusCode::RANGE_NOT_SATISFIABLE,
        "range not satisfiable",
        format!("requested byte range is not satisfiable for {content_length} bytes"),
        SdkWorkResultCode::InvalidParameter,
    )
}

pub fn map_service_error(error: DriveServiceError) -> RouteProblem {
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
            "permission required",
            detail,
            SdkWorkResultCode::PermissionRequired,
        ),
        DriveServiceError::Internal(detail) => {
            tracing::error!(
                target: "sdkwork.drive.internal_api",
                error = %detail,
                "internal Drive service operation failed"
            );
            problem(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error",
                "An unexpected error occurred.",
                SdkWorkResultCode::InternalError,
            )
        }
    }
}

pub fn map_object_store_error(error: DriveObjectStoreError) -> RouteProblem {
    let detail = error.message;
    match error.kind {
        DriveObjectStoreErrorKind::NotFound => problem(
            StatusCode::NOT_FOUND,
            "not found",
            "Drive content was not found.",
            SdkWorkResultCode::NotFound,
        ),
        DriveObjectStoreErrorKind::InvalidRequest => invalid_parameter(detail),
        DriveObjectStoreErrorKind::Conflict | DriveObjectStoreErrorKind::IntegrityFailed => {
            problem(
                StatusCode::CONFLICT,
                "conflict",
                "Drive content is not currently eligible for delivery.",
                SdkWorkResultCode::Conflict,
            )
        }
        DriveObjectStoreErrorKind::PermissionDenied => problem(
            StatusCode::FORBIDDEN,
            "permission required",
            "Drive content provider access was denied.",
            SdkWorkResultCode::PermissionRequired,
        ),
        DriveObjectStoreErrorKind::RateLimited => problem(
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded",
            "Drive content provider rate limit was exceeded.",
            SdkWorkResultCode::RateLimitExceeded,
        ),
        DriveObjectStoreErrorKind::Timeout => problem(
            StatusCode::GATEWAY_TIMEOUT,
            "gateway timeout",
            "Drive content provider timed out.",
            SdkWorkResultCode::GatewayTimeout,
        ),
        DriveObjectStoreErrorKind::Unavailable => problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "service unavailable",
            "Drive content provider is unavailable.",
            SdkWorkResultCode::ServiceUnavailable,
        ),
        DriveObjectStoreErrorKind::UpstreamError => problem(
            StatusCode::BAD_GATEWAY,
            "bad gateway",
            "Drive content provider returned an invalid response.",
            SdkWorkResultCode::BadGateway,
        ),
        DriveObjectStoreErrorKind::NotSupported | DriveObjectStoreErrorKind::Internal => {
            tracing::error!(
                target: "sdkwork.drive.internal_api",
                kind = ?error.kind,
                error = %detail,
                "Drive object-store operation failed"
            );
            problem(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error",
                "An unexpected error occurred.",
                SdkWorkResultCode::InternalError,
            )
        }
    }
}
