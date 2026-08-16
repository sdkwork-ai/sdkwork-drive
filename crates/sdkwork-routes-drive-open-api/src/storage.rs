use crate::error::{map_service_error, ProblemDetail};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_storage_contract::{DriveObjectStoreError, DriveObjectStoreErrorKind};
use sdkwork_drive_storage_s3::{S3DriveObjectStore, S3StoreConfig};
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProviderKind;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub(crate) struct ActiveStorageProviderRecord {
    provider_kind: DriveStorageProviderKind,
    endpoint_url: String,
    region: Option<String>,
    bucket: String,
    path_style: bool,
    strict_tls: bool,
    credential_ref: Option<String>,
}

pub(crate) fn missing_signing_provider_error(bucket: &str) -> DriveServiceError {
    DriveServiceError::Conflict(format!(
        "active storage provider is required for bucket {bucket} to sign object store URLs"
    ))
}

pub(crate) fn unsupported_signing_provider_error(bucket: &str) -> DriveServiceError {
    DriveServiceError::Conflict(format!(
        "active storage provider for bucket {bucket} does not support object store URL signing"
    ))
}

pub(crate) async fn find_active_storage_provider_by_id(
    pool: &PgPool,
    provider_id: &str,
) -> Result<Option<ActiveStorageProviderRecord>, DriveServiceError> {
    let row = sqlx::query(
        "SELECT provider_kind, endpoint_url, region, bucket, path_style, strict_tls, credential_ref
         FROM dr_drive_storage_provider
         WHERE status='active' AND id=$1
         LIMIT 1",
    )
    .bind(provider_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "query active dr_drive_storage_provider failed: {error}"
        ))
    })?;

    let Some(row) = row else {
        return Ok(None);
    };
    let raw_kind: String = row.get("provider_kind");
    let provider_kind = DriveStorageProviderKind::try_from_str(&raw_kind).ok_or_else(|| {
        DriveServiceError::Internal(format!("storage provider kind is invalid: {raw_kind}"))
    })?;
    Ok(Some(ActiveStorageProviderRecord {
        provider_kind,
        endpoint_url: row.get("endpoint_url"),
        region: row.get("region"),
        bucket: row.get("bucket"),
        path_style: get_bool(&row, "path_style")?,
        strict_tls: get_bool(&row, "strict_tls")?,
        credential_ref: row.get("credential_ref"),
    }))
}

fn get_bool(row: &PgRow, column: &str) -> Result<bool, DriveServiceError> {
    row.try_get::<bool, _>(column)
        .or_else(|_| row.try_get::<i64, _>(column).map(|value| value != 0))
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "decode dr_drive_storage_provider.{column} as bool failed: {error}"
            ))
        })
}

pub(crate) async fn build_s3_object_store_for_provider(
    provider: &ActiveStorageProviderRecord,
) -> Result<Option<S3DriveObjectStore>, DriveServiceError> {
    match &provider.provider_kind {
        DriveStorageProviderKind::S3Compatible
        | DriveStorageProviderKind::AliyunOss
        | DriveStorageProviderKind::TencentCos
        | DriveStorageProviderKind::HuaweiObs
        | DriveStorageProviderKind::VolcengineTos
        | DriveStorageProviderKind::GoogleCloudStorage
        | DriveStorageProviderKind::Custom(_) => S3DriveObjectStore::new(
            S3StoreConfig::from_provider_parts(
                provider.provider_kind.as_str(),
                &provider.endpoint_url,
                provider.region.as_deref(),
                &provider.bucket,
                provider.path_style,
                provider.credential_ref.as_deref(),
                Some(provider.strict_tls),
            )
            .map_err(map_object_store_error)?,
        )
        .await
        .map(Some)
        .map_err(|error| {
            DriveServiceError::Internal(format!("build s3-compatible object store failed: {error}"))
        }),
        _ => Ok(None),
    }
}

fn map_object_store_error(error: DriveObjectStoreError) -> DriveServiceError {
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
