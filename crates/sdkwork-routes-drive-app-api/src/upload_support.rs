use crate::dto::*;
use crate::error::{map_service_error, not_found_problem, ProblemDetail};
use crate::ids::next_drive_id;
use crate::object_store::{
    build_s3_object_store_for_provider, find_storage_provider_by_id,
    missing_signing_provider_error, require_active_storage_provider,
    unsupported_signing_provider_error,
};
use crate::state::AppState;
use crate::uploader::map_uploader_upload_item_row;
use async_trait::async_trait;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompletedMultipartPart,
    CreateMultipartUploadRequest, DeleteObjectRequest, DriveObjectLocator,
};
use sdkwork_drive_workspace_service::application::storage_target_service::{
    resolve_storage_target as resolve_workspace_storage_target, DriveStorageTarget,
    ResolveDriveStorageTargetCommand,
};
use sdkwork_drive_workspace_service::application::upload_session_rules::{
    ensure_upload_session_not_expired as ensure_session_not_expired,
    validate_mutable_upload_session as validate_session_mutable,
};
use sdkwork_drive_workspace_service::domain::uploader::DriveUploadItem;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_object_store::{
    next_storage_object_version_no as next_workspace_storage_object_version_no,
    read_node_content_state as read_workspace_node_content_state,
};
use sdkwork_drive_workspace_service::infrastructure::sql::upload_query_columns::DRIVE_UPLOAD_ITEM_SELECT_COLUMNS;
use sdkwork_drive_workspace_service::infrastructure::sql::upload_session_store::SqlUploadSessionStore;
use sdkwork_drive_workspace_service::ports::upload_multipart_storage::{
    AbortDriveMultipartUploadCommand, CompleteDriveMultipartUploadCommand,
    CreateDriveMultipartUploadCommand, DeleteDriveStoredObjectCommand, DriveUploadMultipartStorage,
};
use sdkwork_drive_workspace_service::ports::upload_session_store::DriveUploadSessionStore;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;
use std::collections::BTreeMap;

/// Multipart-storage adapter over the shared object-store runtime.
///
/// The workspace service owns the upload-session state machine but must not
/// depend on provider credentials or the S3 SDK; this adapter is the surface-side
/// half, resolving each provider through `DriveObjectStoreRuntime` (including the
/// account-center credential path) and mapping provider errors onto the same
/// service error kinds the HTTP route used to produce.
#[derive(Clone, Debug)]
pub(crate) struct AppUploadMultipartStorage {
    state: AppState,
}

impl AppUploadMultipartStorage {
    pub(crate) fn new(state: AppState) -> Self {
        Self { state }
    }

    async fn object_store_for(
        &self,
        storage_provider_id: &str,
        bucket: &str,
    ) -> Result<
        std::sync::Arc<dyn sdkwork_drive_storage_contract::DriveObjectStore>,
        DriveServiceError,
    > {
        let provider = find_storage_provider_by_id(&self.state.pool, storage_provider_id).await?;
        let Some(provider) = provider else {
            return Err(missing_signing_provider_error(bucket));
        };
        let provider = require_active_storage_provider(provider, bucket)?;
        let Some(object_store) =
            build_s3_object_store_for_provider(&self.state, &provider.to_domain_provider()).await?
        else {
            return Err(unsupported_signing_provider_error(bucket));
        };
        Ok(object_store)
    }
}

#[async_trait]
impl DriveUploadMultipartStorage for AppUploadMultipartStorage {
    async fn create_multipart_upload(
        &self,
        command: CreateDriveMultipartUploadCommand,
    ) -> Result<String, DriveServiceError> {
        let object_store = self
            .object_store_for(&command.storage_provider_id, &command.bucket)
            .await?;
        let created = object_store
            .create_multipart_upload(CreateMultipartUploadRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
                content_type: None,
                metadata: BTreeMap::new(),
                checksum_sha256_hex: None,
            })
            .await
            .map_err(map_object_store_error_to_service)?;
        Ok(created.upload_id)
    }

    async fn complete_multipart_upload(
        &self,
        command: CompleteDriveMultipartUploadCommand,
    ) -> Result<(), DriveServiceError> {
        let object_store = self
            .object_store_for(&command.storage_provider_id, &command.bucket)
            .await?;
        object_store
            .complete_multipart_upload(CompleteMultipartUploadRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
                upload_id: command.storage_upload_id,
                parts: command
                    .parts
                    .into_iter()
                    .map(|part| CompletedMultipartPart {
                        part_number: part.part_no,
                        etag: part.etag.trim().to_string(),
                    })
                    .collect(),
            })
            .await
            .map_err(map_object_store_error_to_service)?;
        Ok(())
    }

    async fn abort_multipart_upload(
        &self,
        command: AbortDriveMultipartUploadCommand,
    ) -> Result<(), DriveServiceError> {
        let object_store = self
            .object_store_for(&command.storage_provider_id, &command.bucket)
            .await?;
        object_store
            .abort_multipart_upload(AbortMultipartUploadRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
                upload_id: command.storage_upload_id,
            })
            .await
            .map_err(map_object_store_error_to_service)?;
        Ok(())
    }

    async fn delete_object(
        &self,
        command: DeleteDriveStoredObjectCommand,
    ) -> Result<bool, DriveServiceError> {
        let object_store = self
            .object_store_for(&command.storage_provider_id, &command.bucket)
            .await?;
        object_store
            .delete_object(DeleteObjectRequest {
                locator: DriveObjectLocator {
                    bucket: command.bucket,
                    object_key: command.object_key,
                },
            })
            .await
            .map_err(map_object_store_error_to_service)?;
        Ok(true)
    }
}

fn map_object_store_error_to_service(
    error: sdkwork_drive_storage_contract::DriveObjectStoreError,
) -> DriveServiceError {
    crate::error::map_object_store_error(error)
}

/// Resolve where a new object version should be written.
///
/// Delegates to the workspace service so the app surface, the in-process
/// upload-session service and any sibling module share one tenant-scoped binding
/// chain. Errors keep their service kinds, which the route maps to the same HTTP
/// statuses as before.
pub(crate) async fn resolve_storage_target(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    requested_provider_id: Option<&str>,
    requested_bucket: Option<&str>,
    node_id: &str,
    object_id: &str,
    version_no: i64,
) -> Result<StorageTarget, (StatusCode, Json<ProblemDetail>)> {
    let target: DriveStorageTarget = resolve_workspace_storage_target(
        pool,
        ResolveDriveStorageTargetCommand {
            tenant_id,
            space_id,
            requested_provider_id,
            requested_bucket,
            node_id,
            object_id,
            version_no,
        },
    )
    .await
    .map_err(map_service_error)?;
    Ok(StorageTarget {
        provider_id: target.provider_id,
        bucket: target.bucket,
        object_key: target.object_key,
    })
}

pub(crate) async fn next_storage_object_version_no(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    next_workspace_storage_object_version_no(pool, tenant_id, node_id)
        .await
        .map_err(map_service_error)
}

pub(crate) async fn read_node_content_state(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    read_workspace_node_content_state(pool, tenant_id, node_id)
        .await
        .map_err(map_service_error)
}

/// Compatibility wrapper that presents the record DTO for callers that build
/// signing commands from it.
pub(crate) async fn initiate_storage_multipart_upload(
    state: &AppState,
    target: &StorageTarget,
) -> Result<CreatedStorageMultipartUpload, (StatusCode, Json<ProblemDetail>)> {
    let upload_id = AppUploadMultipartStorage::new(state.clone())
        .create_multipart_upload(CreateDriveMultipartUploadCommand {
            storage_provider_id: target.provider_id.clone(),
            bucket: target.bucket.clone(),
            object_key: target.object_key.clone(),
        })
        .await
        .map_err(map_service_error)?;
    Ok(CreatedStorageMultipartUpload { upload_id })
}

pub(crate) async fn compensate_created_storage_multipart_upload(
    state: &AppState,
    tenant_id: &str,
    node_id: &str,
    target: &StorageTarget,
    created: &CreatedStorageMultipartUpload,
    reason: &str,
) {
    sdkwork_drive_observability::metrics::record_multipart_compensation_attempted();
    match abort_storage_multipart_upload_by_locator(
        state,
        &target.provider_id,
        &target.bucket,
        &target.object_key,
        &created.upload_id,
    )
    .await
    {
        Ok(()) => {
            sdkwork_drive_observability::metrics::record_multipart_compensation_succeeded();
            tracing::info!(
                event = "drive.upload.multipart_compensation_succeeded",
                tenant_id = %tenant_id,
                node_id = %node_id,
                provider_id = %target.provider_id,
                bucket = %target.bucket,
                object_key = %target.object_key,
                storage_upload_id = %created.upload_id,
                reason = %reason,
                "aborted multipart upload after database publication failure"
            );
        }
        Err(error) => {
            sdkwork_drive_observability::metrics::record_multipart_compensation_failed();
            tracing::warn!(
                event = "drive.upload.multipart_compensation_failed",
                tenant_id = %tenant_id,
                node_id = %node_id,
                provider_id = %target.provider_id,
                bucket = %target.bucket,
                object_key = %target.object_key,
                storage_upload_id = %created.upload_id,
                reason = %reason,
                error = ?error,
                "failed to abort multipart upload after database publication failure"
            );
        }
    }
}

async fn abort_storage_multipart_upload_by_locator(
    state: &AppState,
    storage_provider_id: &str,
    bucket: &str,
    object_key: &str,
    storage_upload_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    AppUploadMultipartStorage::new(state.clone())
        .abort_multipart_upload(AbortDriveMultipartUploadCommand {
            storage_provider_id: storage_provider_id.to_string(),
            bucket: bucket.to_string(),
            object_key: object_key.to_string(),
            storage_upload_id: storage_upload_id.to_string(),
        })
        .await
        .map_err(map_service_error)
}

pub(crate) fn validate_mutable_upload_session(
    upload_session: &UploadSessionRecord,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    validate_session_mutable(upload_session.state.as_str()).map_err(map_service_error)
}

/// Rejects an upload session whose deadline has passed even though the maintenance
/// sweeper has not flagged it as `expired` yet.
///
/// `sweep_expired_upload_sessions` only runs periodically, so relying on
/// `state = 'expired'` alone lets a client keep presigning parts and finalizing a
/// session long after its TTL elapsed. Aborting stays allowed: releasing a session
/// that can no longer be completed is cleanup, not a content mutation.
pub(crate) fn ensure_upload_session_not_expired(
    upload_session: &UploadSessionRecord,
    now_epoch_ms: i64,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    ensure_session_not_expired(upload_session.expires_at_epoch_ms, now_epoch_ms)
        .map_err(map_service_error)
}

/// Moves an upload session back to `uploading` while presigning a part.
///
/// The guarded update lives in the session store so a presign flow cannot revert
/// a concurrently completed, aborted or expired session.
pub(crate) async fn mark_upload_session_uploading(
    pool: &PgPool,
    tenant_id: &str,
    upload_session_id: &str,
    operator_id: &str,
    now_epoch_ms: i64,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    SqlUploadSessionStore::new(pool.clone())
        .mark_uploading(tenant_id, upload_session_id, operator_id, now_epoch_ms)
        .await
        .map_err(map_service_error)
}

pub(crate) async fn find_upload_session(
    pool: &PgPool,
    tenant_id: &str,
    upload_session_id: &str,
) -> Result<UploadSessionRecord, (StatusCode, Json<ProblemDetail>)> {
    let session = SqlUploadSessionStore::new(pool.clone())
        .find_by_id(tenant_id, upload_session_id)
        .await
        .map_err(map_service_error)?
        .ok_or_else(|| not_found_problem("upload session not found"))?;
    Ok(UploadSessionRecord::from(session))
}

pub(crate) async fn find_upload_session_by_idempotency(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
    idempotency_key: &str,
) -> Result<Option<UploadSessionRecord>, (StatusCode, Json<ProblemDetail>)> {
    let session = SqlUploadSessionStore::new(pool.clone())
        .find_by_idempotency(tenant_id, space_id, node_id, idempotency_key)
        .await
        .map_err(map_service_error)?;
    Ok(session.map(UploadSessionRecord::from))
}

pub(crate) async fn ensure_upload_session_id_available(
    pool: &PgPool,
    upload_session_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    SqlUploadSessionStore::new(pool.clone())
        .ensure_id_available(upload_session_id)
        .await
        .map_err(map_service_error)
}

pub(crate) async fn update_uploader_storage_target(
    pool: &PgPool,
    tenant_id: &str,
    upload_item_id: &str,
    upload_session_id: &str,
    storage_target: &StorageTarget,
    storage_upload_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    sqlx::query(
        "UPDATE dr_drive_upload_session
         SET bucket=$1,
             object_key=$2,
             storage_provider_id=$3,
             storage_upload_id=$4,
             updated_at=CURRENT_TIMESTAMP,
             version=version + 1
         WHERE tenant_id=$5 AND id=$6",
    )
    .bind(&storage_target.bucket)
    .bind(&storage_target.object_key)
    .bind(&storage_target.provider_id)
    .bind(storage_upload_id)
    .bind(tenant_id)
    .bind(upload_session_id)
    .execute(pool)
    .await
    .map_err(|error| {
        crate::error::internal_problem(format!(
            "update uploader dr_drive_upload_session storage target failed: {error}"
        ))
    })?;

    sqlx::query(
        "UPDATE dr_drive_upload_item
         SET storage_provider_id=$1,
             storage_upload_id=$2,
             updated_at=CURRENT_TIMESTAMP
         WHERE tenant_id=$3 AND id=$4",
    )
    .bind(&storage_target.provider_id)
    .bind(storage_upload_id)
    .bind(tenant_id)
    .bind(upload_item_id)
    .execute(pool)
    .await
    .map_err(|error| {
        crate::error::internal_problem(format!(
            "update uploader dr_drive_upload_item storage target failed: {error}"
        ))
    })?;
    Ok(())
}

pub(crate) async fn find_uploader_upload_item(
    pool: &PgPool,
    tenant_id: &str,
    upload_item_id: &str,
) -> Result<Option<DriveUploadItem>, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {DRIVE_UPLOAD_ITEM_SELECT_COLUMNS}
         FROM dr_drive_upload_item
         WHERE tenant_id=$1 AND id=$2",
    )))
    .bind(tenant_id)
    .bind(upload_item_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        crate::error::internal_problem(format!("find dr_drive_upload_item failed: {error}"))
    })?;
    row.map(|row| map_uploader_upload_item_row(&row))
        .transpose()
}

/// Insert one logical node version on an existing transaction connection.
///
/// Kept for the node-creation flows in this surface that already own their
/// transaction; upload-session completion now goes through the workspace service.
pub(crate) async fn insert_node_version_metadata<'e, E>(
    executor: E,
    metadata: NodeVersionStorageMetadata<'_>,
    attribution: NodeVersionAttribution<'_>,
    operator_id: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let version_id = next_drive_id("ver");
    sqlx::query(
        "INSERT INTO dr_drive_node_version (
            id, tenant_id, space_id, node_id, version_no, storage_object_id,
            content_type, content_length, checksum_sha256_hex, version_kind,
            version_label, change_source, change_summary, restored_from_version_id,
            app_id, app_resource_type, app_resource_id, scene, source,
            lifecycle_status, created_by, updated_by
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            NULL, $11, $12, NULL, $13, $14, $15, $16, $17,
            'active', $18, $18
         )",
    )
    .bind(&version_id)
    .bind(metadata.tenant_id)
    .bind(metadata.space_id)
    .bind(metadata.node_id)
    .bind(metadata.version_no)
    .bind(metadata.storage_object_id)
    .bind(metadata.content_type)
    .bind(metadata.content_length)
    .bind(metadata.checksum_sha256_hex)
    .bind(attribution.version_kind)
    .bind(attribution.change_source)
    .bind(attribution.change_summary)
    .bind(attribution.app_id)
    .bind(attribution.app_resource_type)
    .bind(attribution.app_resource_id)
    .bind(attribution.scene)
    .bind(attribution.source)
    .bind(operator_id)
    .execute(executor)
    .await
    .map_err(|error| {
        crate::error::internal_problem(format!("insert dr_drive_node_version failed: {error}"))
    })?;
    Ok(version_id)
}

pub(crate) struct NodeVersionStorageMetadata<'a> {
    pub(crate) tenant_id: &'a str,
    pub(crate) space_id: &'a str,
    pub(crate) node_id: &'a str,
    pub(crate) version_no: i64,
    pub(crate) storage_object_id: &'a str,
    pub(crate) content_type: &'a str,
    pub(crate) content_length: i64,
    pub(crate) checksum_sha256_hex: &'a str,
}

pub(crate) struct NodeVersionAttribution<'a> {
    pub(crate) version_kind: &'a str,
    pub(crate) change_source: &'a str,
    pub(crate) change_summary: Option<&'a str>,
    pub(crate) app_id: Option<&'a str>,
    pub(crate) app_resource_type: Option<&'a str>,
    pub(crate) app_resource_id: Option<&'a str>,
    pub(crate) scene: Option<&'a str>,
    pub(crate) source: Option<&'a str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_in_state(state: &str, expires_at_epoch_ms: i64) -> UploadSessionRecord {
        UploadSessionRecord {
            id: "session-1".to_string(),
            tenant_id: "tenant-1".to_string(),
            space_id: "space-1".to_string(),
            node_id: "node-1".to_string(),
            bucket: "bucket-1".to_string(),
            object_key: "objects/node-1/v1".to_string(),
            idempotency_key: "idem-1".to_string(),
            storage_provider_id: "provider-1".to_string(),
            storage_upload_id: "upload-1".to_string(),
            state: state.to_string(),
            expires_at_epoch_ms,
            version: 1,
        }
    }

    #[test]
    fn expired_session_is_rejected_while_still_marked_mutable() {
        let session = session_in_state("uploading", 1_000);
        let error = ensure_upload_session_not_expired(&session, 1_000).expect_err("expired");
        assert_eq!(error.0, StatusCode::CONFLICT);
    }

    #[test]
    fn session_within_its_deadline_is_accepted() {
        let session = session_in_state("created", 1_000);
        ensure_upload_session_not_expired(&session, 999).expect("not expired");
    }

    #[test]
    fn completed_session_is_not_mutable() {
        let session = session_in_state("completed", 1_000);
        let error = validate_mutable_upload_session(&session).expect_err("completed");
        assert_eq!(error.0, StatusCode::CONFLICT);
    }

    #[test]
    fn drive_multipart_part_conversion_preserves_order_and_etag() {
        let parts = [DriveMultipartPart {
            part_no: 3,
            etag: "etag-3".to_string(),
        }];
        assert_eq!(parts[0].part_no, 3);
        assert_eq!(parts[0].etag, "etag-3");
    }
}
