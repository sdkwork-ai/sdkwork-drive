use crate::error::map_object_store_error;
use crate::state::AppState;
use crate::time::signing_ttl_seconds;
use async_trait::async_trait;
use sdkwork_drive_storage_contract::{
    DriveObjectLocator, DriveObjectStore, PresignDownloadRequest,
    PresignUploadPartRequest as ObjectStorePresignUploadPartRequest,
};
use sdkwork_drive_storage_s3::{S3DriveObjectStore, S3StoreConfig};
use sdkwork_drive_workspace_service::application::download_service::DriveDownloadService;
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProviderKind;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_object_store::SqlStorageObjectStore;
use sdkwork_drive_workspace_service::ports::storage_object_store::{
    DownloadSignCommand, DriveDownloadSigner, SignedDownloadPayload,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::BTreeMap;

pub(crate) fn build_download_service(
    state: &AppState,
) -> DriveDownloadService<SqlStorageObjectStore, AppDownloadSigner> {
    DriveDownloadService::new(
        SqlStorageObjectStore::new(state.pool.clone()),
        AppDownloadSigner::new(state.pool.clone()),
    )
}

#[derive(Debug, Clone)]
pub(crate) struct AppDownloadSigner {
    pool: PgPool,
}

#[derive(Debug, Clone)]
pub(crate) struct UploadPartSignCommand {
    pub(crate) storage_provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) upload_id: String,
    pub(crate) part_no: u16,
    pub(crate) expires_at_epoch_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignedUploadPartPayload {
    pub(crate) method: String,
    pub(crate) raw_url: String,
    pub(crate) headers: BTreeMap<String, String>,
    pub(crate) expires_at_epoch_ms: i64,
}

impl AppDownloadSigner {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub(crate) async fn sign_upload_part(
        &self,
        command: UploadPartSignCommand,
    ) -> Result<SignedUploadPartPayload, DriveServiceError> {
        let provider =
            find_storage_provider_by_id(&self.pool, &command.storage_provider_id).await?;
        let Some(provider) = provider else {
            return Err(missing_signing_provider_error(&command.bucket));
        };
        let provider = require_active_storage_provider(provider, &command.bucket)?;
        match build_s3_object_store_for_provider(&provider).await? {
            Some(object_store) => {
                let ttl_seconds = signing_ttl_seconds(command.expires_at_epoch_ms)?;
                let signed = object_store
                    .presign_upload_part(ObjectStorePresignUploadPartRequest {
                        locator: DriveObjectLocator {
                            bucket: command.bucket,
                            object_key: command.object_key,
                        },
                        upload_id: command.upload_id,
                        part_number: command.part_no,
                        expires_in_seconds: ttl_seconds,
                    })
                    .await
                    .map_err(map_object_store_error)?;
                Ok(SignedUploadPartPayload {
                    method: signed.method,
                    raw_url: signed.url,
                    headers: signed.headers,
                    expires_at_epoch_ms: signed.expires_at_epoch_ms,
                })
            }
            None => Err(unsupported_signing_provider_error(&command.bucket)),
        }
    }
}

#[async_trait]
impl DriveDownloadSigner for AppDownloadSigner {
    async fn sign_download(
        &self,
        command: DownloadSignCommand,
    ) -> Result<SignedDownloadPayload, DriveServiceError> {
        let provider =
            find_storage_provider_by_id(&self.pool, &command.storage_provider_id).await?;
        let Some(provider) = provider else {
            return Err(missing_signing_provider_error(&command.bucket));
        };
        let provider = require_active_storage_provider(provider, &command.bucket)?;
        match build_s3_object_store_for_provider(&provider).await? {
            Some(object_store) => {
                let ttl_seconds = signing_ttl_seconds(command.expires_at_epoch_ms)?;
                let signed = object_store
                    .presign_download(PresignDownloadRequest {
                        locator: DriveObjectLocator {
                            bucket: command.bucket,
                            object_key: command.object_key,
                        },
                        expires_in_seconds: ttl_seconds,
                    })
                    .await
                    .map_err(map_object_store_error)?;
                Ok(SignedDownloadPayload {
                    method: signed.method,
                    raw_url: signed.url,
                    headers: signed.headers,
                    expires_at_epoch_ms: signed.expires_at_epoch_ms,
                })
            }
            None => Err(unsupported_signing_provider_error(&command.bucket)),
        }
    }
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

#[derive(Debug, Clone)]
pub(crate) struct ActiveStorageProviderRecord {
    pub(crate) id: String,
    pub(crate) provider_kind: DriveStorageProviderKind,
    pub(crate) endpoint_url: String,
    pub(crate) region: Option<String>,
    pub(crate) bucket: String,
    pub(crate) status: String,
    pub(crate) path_style: bool,
    pub(crate) strict_tls: bool,
    pub(crate) credential_ref: Option<String>,
}

pub(crate) async fn find_active_storage_provider_by_bucket(
    pool: &PgPool,
    bucket: &str,
) -> Result<Option<ActiveStorageProviderRecord>, DriveServiceError> {
    let row = sqlx::query(
        "SELECT id, provider_kind, endpoint_url, region, bucket, path_style, strict_tls, credential_ref
         FROM dr_drive_storage_provider
         WHERE status='active' AND bucket=$1
         ORDER BY updated_at DESC, id ASC
         LIMIT 1",
    )
    .bind(bucket)
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
        id: row.get("id"),
        provider_kind,
        endpoint_url: row.get("endpoint_url"),
        region: row.get("region"),
        bucket: row.get("bucket"),
        status: "active".to_string(),
        path_style: get_bool(&row, "path_style")?,
        strict_tls: get_bool(&row, "strict_tls")?,
        credential_ref: row.get("credential_ref"),
    }))
}

pub(crate) async fn find_storage_provider_by_id(
    pool: &PgPool,
    provider_id: &str,
) -> Result<Option<ActiveStorageProviderRecord>, DriveServiceError> {
    let row = sqlx::query(
        "SELECT id, provider_kind, endpoint_url, region, bucket, status, path_style, strict_tls, credential_ref
         FROM dr_drive_storage_provider
         WHERE id=$1
         LIMIT 1",
    )
    .bind(provider_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "query dr_drive_storage_provider by id failed: {error}"
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
        id: row.get("id"),
        provider_kind,
        endpoint_url: row.get("endpoint_url"),
        region: row.get("region"),
        bucket: row.get("bucket"),
        status: row.get("status"),
        path_style: get_bool(&row, "path_style")?,
        strict_tls: get_bool(&row, "strict_tls")?,
        credential_ref: row.get("credential_ref"),
    }))
}

pub(crate) fn require_active_storage_provider(
    provider: ActiveStorageProviderRecord,
    bucket: &str,
) -> Result<ActiveStorageProviderRecord, DriveServiceError> {
    if provider.status == "active" {
        Ok(provider)
    } else {
        Err(missing_signing_provider_error(bucket))
    }
}

fn get_bool(row: &sqlx::postgres::PgRow, column: &str) -> Result<bool, DriveServiceError> {
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
    if provider_supports_s3_object_store(&provider.provider_kind) {
        S3DriveObjectStore::new(
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
        })
    } else {
        Ok(None)
    }
}

fn provider_supports_s3_object_store(provider_kind: &DriveStorageProviderKind) -> bool {
    matches!(
        provider_kind,
        DriveStorageProviderKind::S3Compatible
            | DriveStorageProviderKind::AliyunOss
            | DriveStorageProviderKind::TencentCos
            | DriveStorageProviderKind::HuaweiObs
            | DriveStorageProviderKind::VolcengineTos
            | DriveStorageProviderKind::GoogleCloudStorage
            | DriveStorageProviderKind::Custom(_)
    )
}
