use crate::dto::{CreateOpenDownloadUrlRequest, OpenDownloadUrlResponse, OpenShareLinkAccessQuery};
use crate::error::{
    map_service_error, problem, share_link_expired_problem, ProblemDetail, SdkWorkResultCode,
};
use crate::repository::{
    claim_share_link_download_slot, find_active_share_link, map_share_link_row,
    release_share_link_download_slot,
};
use crate::response::{created_resource, success_resource};
use crate::state::OpenState;
use crate::storage::{
    build_s3_object_store_for_provider, find_active_storage_provider_by_id,
    map_object_store_route_error, missing_signing_provider_error,
    unsupported_signing_provider_error,
};
use crate::time::now_epoch_ms;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_storage_contract::{
    DriveObjectLocator, DriveObjectStore, PresignDownloadRequest,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::Row;

pub(crate) async fn resolve_share_link(
    State(state): State<OpenState>,
    Path(token): Path<String>,
    Query(query): Query<OpenShareLinkAccessQuery>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = find_active_share_link(&state.pool, &token, query.access_code.as_deref()).await?;
    Ok(success_resource(map_share_link_row(&row)))
}

pub(crate) async fn create_share_link_download_url(
    State(state): State<OpenState>,
    Path(token): Path<String>,
    Json(payload): Json<CreateOpenDownloadUrlRequest>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = find_active_share_link(&state.pool, &token, payload.access_code.as_deref()).await?;
    let share_id: String = row.get("share_id");
    let download_limit: Option<i64> = row.get("download_limit");
    let download_count: i64 = row.get("download_count");
    if let Some(limit) = download_limit {
        if download_count >= limit {
            return Err(problem(
                StatusCode::TOO_MANY_REQUESTS,
                "download limit exceeded",
                "share link download limit exceeded",
                SdkWorkResultCode::RateLimitExceeded,
            ));
        }
    }

    let requested_ttl_seconds = validate_requested_ttl_seconds(payload.requested_ttl_seconds)?;
    let share_expires_at_epoch_ms: Option<i64> = row.get("expires_at_epoch_ms");
    let ttl_seconds = signing_ttl_seconds(requested_ttl_seconds, share_expires_at_epoch_ms)?;
    let storage_provider_id: Option<String> = row.get("storage_provider_id");
    let bucket: Option<String> = row.get("bucket");
    let object_key: Option<String> = row.get("object_key");
    let Some(storage_provider_id) = storage_provider_id.filter(|value| !value.trim().is_empty())
    else {
        return Err(map_service_error(DriveServiceError::NotFound(
            "active storage object for shared node is not found".to_string(),
        )));
    };
    let Some(bucket) = bucket.filter(|value| !value.is_empty()) else {
        return Err(map_service_error(DriveServiceError::NotFound(
            "active storage object for shared node is not found".to_string(),
        )));
    };
    let Some(object_key) = object_key.filter(|value| !value.is_empty()) else {
        return Err(map_service_error(DriveServiceError::NotFound(
            "active storage object for shared node is not found".to_string(),
        )));
    };
    let provider = find_active_storage_provider_by_id(&state.pool, &storage_provider_id)
        .await
        .map_err(map_service_error)?;
    let Some(provider) = provider else {
        return Err(map_service_error(missing_signing_provider_error(&bucket)));
    };
    let Some(object_store) = build_s3_object_store_for_provider(&provider)
        .await
        .map_err(map_service_error)?
    else {
        return Err(map_service_error(unsupported_signing_provider_error(
            &bucket,
        )));
    };

    claim_share_link_download_slot(&state.pool, &share_id).await?;
    let signed = match object_store
        .presign_download(PresignDownloadRequest {
            locator: DriveObjectLocator {
                bucket: bucket.clone(),
                object_key,
            },
            expires_in_seconds: ttl_seconds,
        })
        .await
    {
        Ok(signed) => signed,
        Err(error) => {
            if let Err(release_error) =
                release_share_link_download_slot(&state.pool, &share_id).await
            {
                tracing::warn!(
                    error = %release_error,
                    share_link_id = %share_id,
                    "release share link download slot failed after presign error"
                );
            }
            return Err(map_object_store_route_error(error));
        }
    };

    Ok(created_resource(OpenDownloadUrlResponse {
        download_url: signed.url,
        expires_at_epoch_ms: signed.expires_at_epoch_ms,
        method: signed.method,
    }))
}

fn validate_requested_ttl_seconds(
    requested_ttl_seconds: Option<u32>,
) -> Result<u32, (StatusCode, Json<ProblemDetail>)> {
    let ttl_seconds = requested_ttl_seconds.unwrap_or(120);
    if !(1..=3600).contains(&ttl_seconds) {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "requestedTtlSeconds must be between 1 and 3600 seconds",
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(ttl_seconds)
}

fn signing_ttl_seconds(
    requested_ttl_seconds: u32,
    share_expires_at_epoch_ms: Option<i64>,
) -> Result<u32, (StatusCode, Json<ProblemDetail>)> {
    let Some(share_expires_at_epoch_ms) = share_expires_at_epoch_ms else {
        return Ok(requested_ttl_seconds);
    };
    let remaining_ms = share_expires_at_epoch_ms - now_epoch_ms();
    if remaining_ms < 1_000 {
        return Err(share_link_expired_problem());
    }
    Ok(requested_ttl_seconds.min((remaining_ms / 1_000) as u32))
}
