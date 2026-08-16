use crate::dto::*;
use crate::error::{
    internal_sql_error, map_object_store_route_error, map_service_error, not_found_problem,
    problem, ProblemDetail, SdkWorkResultCode,
};
use crate::hashing::sha256_hex;
use crate::ids::next_drive_id;
use crate::object_store::{
    build_s3_object_store_for_provider, find_active_storage_provider_by_bucket,
    find_storage_provider_by_id, missing_signing_provider_error, require_active_storage_provider,
    unsupported_signing_provider_error,
};
use crate::state::AppState;
use crate::storage_keys::*;
use crate::uploader::map_uploader_upload_item_row;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompletedMultipartPart,
    CreateMultipartUploadRequest, DeleteObjectRequest, DriveObjectLocator, DriveObjectStore,
};
use sdkwork_drive_workspace_service::application::storage_key_service::{
    BuildStorageObjectKeyCommand, DriveStorageKeyService,
};
use sdkwork_drive_workspace_service::domain::upload::DriveUploadSessionState;
use sdkwork_drive_workspace_service::domain::uploader::DriveUploadItem;
use sdkwork_drive_workspace_service::infrastructure::sql::upload_query_columns::DRIVE_UPLOAD_ITEM_SELECT_COLUMNS;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::BTreeMap;

pub(crate) async fn resolve_storage_target(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    requested_bucket: Option<&str>,
    node_id: &str,
    object_id: &str,
    version_no: i64,
) -> Result<StorageTarget, (StatusCode, Json<ProblemDetail>)> {
    let default_target = match requested_bucket
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(bucket) => DefaultStorageProviderTarget {
            provider_id: find_active_storage_provider_by_bucket(pool, bucket)
                .await
                .map_err(map_service_error)?
                .ok_or_else(|| map_service_error(missing_signing_provider_error(bucket)))?
                .id,
            bucket: bucket.to_string(),
            storage_root_prefix: default_storage_root_prefix(tenant_id, Some(space_id)),
        },
        None => resolve_default_provider_target(pool, tenant_id, space_id).await?,
    };
    let standard_object_key =
        DriveStorageKeyService::build_object_key(BuildStorageObjectKeyCommand {
            tenant_id,
            space_id,
            node_id,
            version_no,
            object_id,
        })
        .map_err(|message| {
            problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                message,
                SdkWorkResultCode::ValidationError,
            )
        })?;
    let object_key =
        join_storage_root_prefix(&default_target.storage_root_prefix, &standard_object_key)?;
    Ok(StorageTarget {
        provider_id: default_target.provider_id,
        bucket: default_target.bucket,
        object_key,
    })
}
pub(crate) async fn next_storage_object_version_no(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_no), 0) + 1
         FROM dr_drive_storage_object
         WHERE tenant_id=$1 AND node_id=$2",
    )
    .bind(tenant_id)
    .bind(node_id)
    .fetch_one(pool)
    .await
    .map_err(internal_sql_error(
        "compute dr_drive_storage_object version failed",
    ))
}
pub(crate) async fn resolve_default_provider_target(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
) -> Result<DefaultStorageProviderTarget, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT provider.id AS provider_id, provider.bucket, binding.storage_root_prefix
         FROM dr_drive_storage_provider_binding binding
         INNER JOIN dr_drive_storage_provider provider ON provider.id = binding.provider_id
         WHERE binding.tenant_id=$1
           AND binding.space_id=$2
           AND binding.purpose='primary'
           AND binding.lifecycle_status='active'
           AND provider.status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "resolve space dr_drive_storage_provider_binding failed",
    ))?;
    if let Some(row) = row {
        return Ok(DefaultStorageProviderTarget {
            provider_id: row.get("provider_id"),
            bucket: row.get("bucket"),
            storage_root_prefix: row.get("storage_root_prefix"),
        });
    }

    let row = sqlx::query(
        "SELECT provider.id AS provider_id, provider.bucket, binding.storage_root_prefix
         FROM dr_drive_storage_provider_binding binding
         INNER JOIN dr_drive_storage_provider provider ON provider.id = binding.provider_id
         INNER JOIN dr_drive_space space ON space.tenant_id = binding.tenant_id
           AND space.id = $2
           AND space.lifecycle_status = 'active'
         WHERE binding.tenant_id=$1
           AND binding.binding_scope='space_type'
           AND binding.purpose = space.space_type
           AND binding.lifecycle_status='active'
           AND provider.status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "resolve space_type dr_drive_storage_provider_binding failed",
    ))?;
    if let Some(row) = row {
        return Ok(DefaultStorageProviderTarget {
            provider_id: row.get("provider_id"),
            bucket: row.get("bucket"),
            storage_root_prefix: row.get("storage_root_prefix"),
        });
    }

    let row = sqlx::query(
        "SELECT provider.id AS provider_id, provider.bucket, binding.storage_root_prefix
         FROM dr_drive_storage_provider_binding binding
         INNER JOIN dr_drive_storage_provider provider ON provider.id = binding.provider_id
         WHERE binding.tenant_id=$1
           AND binding.space_id IS NULL
           AND binding.purpose='primary'
           AND binding.lifecycle_status='active'
           AND provider.status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "resolve tenant dr_drive_storage_provider_binding failed",
    ))?;
    if let Some(row) = row {
        return Ok(DefaultStorageProviderTarget {
            provider_id: row.get("provider_id"),
            bucket: row.get("bucket"),
            storage_root_prefix: row.get("storage_root_prefix"),
        });
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "bucket is required when no default storage provider binding exists",
        SdkWorkResultCode::ValidationError,
    ))
}
pub(crate) async fn read_node_content_state(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let value = sqlx::query_scalar(
        "SELECT content_state
         FROM dr_drive_node
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status != 'deleted'",
    )
    .bind(tenant_id)
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "read dr_drive_node content_state failed",
    ))?;
    value.ok_or_else(|| not_found_problem("node not found"))
}
pub(crate) async fn insert_node_version_for_storage_object<'e, E>(
    executor: E,
    storage: NodeVersionStorageMetadata<'_>,
    upload_item: Option<&DriveUploadItem>,
    operator_id: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    insert_node_version_metadata(
        executor,
        storage,
        NodeVersionAttribution {
            version_kind: "auto",
            change_source: if upload_item.is_some() {
                "uploader"
            } else {
                "app_api"
            },
            change_summary: None,
            app_id: upload_item.map(|item| item.app_id.as_str()),
            app_resource_type: upload_item.map(|item| item.app_resource_type.as_str()),
            app_resource_id: upload_item.map(|item| item.app_resource_id.as_str()),
            scene: upload_item.and_then(|item| item.scene.as_deref()),
            source: upload_item.and_then(|item| item.source.as_deref()),
        },
        operator_id,
    )
    .await
}
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
    .map_err(internal_sql_error("insert dr_drive_node_version failed"))?;
    Ok(version_id)
}
pub(crate) async fn ensure_upload_session_id_available(
    pool: &PgPool,
    upload_session_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_upload_session
         WHERE id=$1",
    )
    .bind(upload_session_id)
    .fetch_one(pool)
    .await
    .map_err(internal_sql_error(
        "check dr_drive_upload_session id conflict failed",
    ))?;
    if count > 0 {
        return Err(problem(
            StatusCode::CONFLICT,
            "conflict",
            "upload session id already exists",
            SdkWorkResultCode::Conflict,
        ));
    }
    Ok(())
}
pub(crate) async fn find_upload_session(
    pool: &PgPool,
    tenant_id: &str,
    upload_session_id: &str,
) -> Result<UploadSessionRecord, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT id, tenant_id, space_id, node_id, bucket, object_key,
                idempotency_key, storage_provider_id, storage_upload_id, state,
                expires_at_epoch_ms, version
         FROM dr_drive_upload_session
         WHERE tenant_id=$1 AND id=$2",
    )
    .bind(tenant_id)
    .bind(upload_session_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find dr_drive_upload_session failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("upload session not found"));
    };
    Ok(map_upload_session_row(&row))
}
pub(crate) async fn find_upload_session_by_idempotency(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
    idempotency_key: &str,
) -> Result<Option<UploadSessionRecord>, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT id, tenant_id, space_id, node_id, bucket, object_key,
                idempotency_key, storage_provider_id, storage_upload_id, state,
                expires_at_epoch_ms, version
         FROM dr_drive_upload_session
         WHERE tenant_id=$1
           AND space_id=$2
           AND node_id=$3
           AND idempotency_key=$4
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(node_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "find dr_drive_upload_session by idempotency failed",
    ))?;
    Ok(row.as_ref().map(map_upload_session_row))
}
pub(crate) fn map_upload_session_row(row: &sqlx::postgres::PgRow) -> UploadSessionRecord {
    UploadSessionRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        node_id: row.get("node_id"),
        bucket: row.get("bucket"),
        object_key: row.get("object_key"),
        idempotency_key: row.get("idempotency_key"),
        storage_provider_id: row.get("storage_provider_id"),
        storage_upload_id: row.get("storage_upload_id"),
        state: row.get("state"),
        expires_at_epoch_ms: row.get("expires_at_epoch_ms"),
        version: row.get("version"),
    }
}
pub(crate) async fn initiate_storage_multipart_upload(
    state: &AppState,
    target: &StorageTarget,
) -> Result<CreatedStorageMultipartUpload, (StatusCode, Json<ProblemDetail>)> {
    let provider = find_storage_provider_by_id(&state.pool, &target.provider_id)
        .await
        .map_err(map_service_error)?;
    let Some(provider) = provider else {
        return Err(map_service_error(missing_signing_provider_error(
            &target.bucket,
        )));
    };
    let provider =
        require_active_storage_provider(provider, &target.bucket).map_err(map_service_error)?;
    let Some(object_store) = build_s3_object_store_for_provider(&provider)
        .await
        .map_err(map_service_error)?
    else {
        return Err(map_service_error(unsupported_signing_provider_error(
            &target.bucket,
        )));
    };
    let created = object_store
        .create_multipart_upload(CreateMultipartUploadRequest {
            locator: DriveObjectLocator {
                bucket: target.bucket.clone(),
                object_key: target.object_key.clone(),
            },
            content_type: None,
            metadata: BTreeMap::new(),
            checksum_sha256_hex: None,
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(CreatedStorageMultipartUpload {
        upload_id: created.upload_id,
    })
}
pub(crate) async fn complete_storage_multipart_upload(
    state: &AppState,
    upload_session: &UploadSessionRecord,
    parts: &[CompletedUploadPartRequest],
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let provider = find_storage_provider_by_id(&state.pool, &upload_session.storage_provider_id)
        .await
        .map_err(map_service_error)?;
    let Some(provider) = provider else {
        return Err(map_service_error(missing_signing_provider_error(
            &upload_session.bucket,
        )));
    };
    let provider = require_active_storage_provider(provider, &upload_session.bucket)
        .map_err(map_service_error)?;
    let Some(object_store) = build_s3_object_store_for_provider(&provider)
        .await
        .map_err(map_service_error)?
    else {
        return Err(map_service_error(unsupported_signing_provider_error(
            &upload_session.bucket,
        )));
    };
    object_store
        .complete_multipart_upload(CompleteMultipartUploadRequest {
            locator: DriveObjectLocator {
                bucket: upload_session.bucket.clone(),
                object_key: upload_session.object_key.clone(),
            },
            upload_id: upload_session.storage_upload_id.clone(),
            parts: parts
                .iter()
                .map(|part| CompletedMultipartPart {
                    part_number: part.part_no,
                    etag: part.etag.trim().to_string(),
                })
                .collect(),
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(())
}
pub(crate) async fn abort_storage_multipart_upload(
    state: &AppState,
    upload_session: &UploadSessionRecord,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    abort_storage_multipart_upload_by_locator(
        state,
        &upload_session.storage_provider_id,
        &upload_session.bucket,
        &upload_session.object_key,
        &upload_session.storage_upload_id,
    )
    .await
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
    let provider = find_storage_provider_by_id(&state.pool, storage_provider_id)
        .await
        .map_err(map_service_error)?;
    let Some(provider) = provider else {
        return Err(map_service_error(missing_signing_provider_error(bucket)));
    };
    let provider = require_active_storage_provider(provider, bucket).map_err(map_service_error)?;
    let Some(object_store) = build_s3_object_store_for_provider(&provider)
        .await
        .map_err(map_service_error)?
    else {
        return Err(map_service_error(unsupported_signing_provider_error(
            bucket,
        )));
    };
    object_store
        .abort_multipart_upload(AbortMultipartUploadRequest {
            locator: DriveObjectLocator {
                bucket: bucket.to_string(),
                object_key: object_key.to_string(),
            },
            upload_id: storage_upload_id.to_string(),
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(())
}
pub(crate) fn validate_mutable_upload_session(
    upload_session: &UploadSessionRecord,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    match upload_session.state.as_str() {
        "created" | "uploading" => Ok(()),
        "completing" | "completed" | "aborted" | "expired" => Err(problem(
            StatusCode::CONFLICT,
            "conflict",
            format!(
                "upload session cannot be modified from {} state",
                upload_session.state
            ),
            SdkWorkResultCode::Conflict,
        )),
        _ => Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "upload session state is invalid",
            SdkWorkResultCode::ValidationError,
        )),
    }
}
pub(crate) async fn plan_completed_storage_object_insert(
    pool: &PgPool,
    tenant_id: &str,
    upload_session: &UploadSessionRecord,
) -> Result<CompletedStorageObjectInsertPlan, (StatusCode, Json<ProblemDetail>)> {
    let version_no = parse_storage_object_version_no_from_key(
        &upload_session.object_key,
        tenant_id,
        &upload_session.space_id,
        &upload_session.node_id,
    )?;
    let next_version_no =
        next_storage_object_version_no(pool, tenant_id, &upload_session.node_id).await?;
    if version_no != next_version_no {
        return Err(problem(
            StatusCode::CONFLICT,
            "conflict",
            "upload session storage object version does not match next file version",
            SdkWorkResultCode::Conflict,
        ));
    }
    let id = format!("{}-v{}", upload_session.id, version_no);

    let id_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE id=$1",
    )
    .bind(&id)
    .fetch_one(pool)
    .await
    .map_err(internal_sql_error(
        "check dr_drive_storage_object id conflict failed",
    ))?;
    if id_count > 0 {
        return Err(problem(
            StatusCode::CONFLICT,
            "conflict",
            "storage object id already exists",
            SdkWorkResultCode::Conflict,
        ));
    }

    let active_locator_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_storage_object
         WHERE tenant_id=$1
           AND node_id=$2
           AND bucket=$3
           AND object_key=$4
           AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(&upload_session.node_id)
    .bind(&upload_session.bucket)
    .bind(&upload_session.object_key)
    .fetch_one(pool)
    .await
    .map_err(internal_sql_error(
        "check dr_drive_storage_object active locator conflict failed",
    ))?;
    if active_locator_count > 0 {
        return Err(problem(
            StatusCode::CONFLICT,
            "conflict",
            "active storage object locator already exists",
            SdkWorkResultCode::Conflict,
        ));
    }

    Ok(CompletedStorageObjectInsertPlan { id, version_no })
}
pub(crate) fn parse_storage_object_version_no_from_key(
    object_key: &str,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    let invalid = || {
        problem(
            StatusCode::CONFLICT,
            "conflict",
            "upload session object key is not a standard Drive storage key",
            SdkWorkResultCode::Conflict,
        )
    };
    let standard_object_key = standard_storage_object_key_suffix(object_key).ok_or_else(invalid)?;
    let segments: Vec<&str> = standard_object_key.split('/').collect();
    if segments.len() != 16
        || segments[0] != "sdkwork-drive"
        || segments[1] != "v1"
        || segments[2] != "t"
        || segments[4] != "tenants"
        || segments[5] != tenant_id
        || segments[6] != "spaces"
        || segments[7] != space_id
        || segments[8] != "nodes"
        || segments[9] != "n"
        || segments[11] != node_id
        || segments[12] != "versions"
        || segments[15] != "content"
    {
        return Err(invalid());
    }

    let raw_version_no = segments[13];
    if raw_version_no.len() != 10 || !raw_version_no.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid());
    }
    let version_no = raw_version_no.parse::<i64>().map_err(|_| invalid())?;
    if version_no < 1 {
        return Err(invalid());
    }

    let expected = DriveStorageKeyService::build_object_key(BuildStorageObjectKeyCommand {
        tenant_id,
        space_id,
        node_id,
        version_no,
        object_id: segments[14],
    })
    .map_err(|_| invalid())?;
    if expected != standard_object_key {
        return Err(invalid());
    }

    Ok(version_no)
}
pub(crate) fn standard_storage_object_key_suffix(object_key: &str) -> Option<&str> {
    object_key
        .rmatch_indices("sdkwork-drive/v1/t/")
        .next()
        .map(|(index, _)| &object_key[index..])
}
pub(crate) fn validate_completed_multipart_parts(
    parts: &[CompletedUploadPartRequest],
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if parts.is_empty() {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "parts are required",
            SdkWorkResultCode::ValidationError,
        ));
    }
    if parts.len() > 10_000 {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "parts must contain at most 10000 items",
            SdkWorkResultCode::ValidationError,
        ));
    }

    let mut previous_part_no = 0_u16;
    for part in parts {
        if part.part_no == 0 || part.part_no > 10_000 {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                "partNo must be between 1 and 10000",
                SdkWorkResultCode::ValidationError,
            ));
        }
        if part.etag.trim().is_empty() {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                "etag is required for every part",
                SdkWorkResultCode::ValidationError,
            ));
        }
        if part.part_no <= previous_part_no {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                "parts must be sorted by ascending unique partNo",
                SdkWorkResultCode::ValidationError,
            ));
        }
        previous_part_no = part.part_no;
    }

    Ok(())
}
pub(crate) async fn update_upload_session_state<'e, E>(
    executor: E,
    tenant_id: &str,
    upload_session_id: &str,
    next_state: &str,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let affected = sqlx::query(
        "UPDATE dr_drive_upload_session
         SET state=$1, updated_by=$2, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$3 AND id=$4",
    )
    .bind(next_state)
    .bind(operator_id)
    .bind(tenant_id)
    .bind(upload_session_id)
    .execute(executor)
    .await
    .map_err(internal_sql_error(
        "update dr_drive_upload_session state failed",
    ))?
    .rows_affected();
    if affected == 0 {
        return Err(not_found_problem("upload session not found"));
    }
    Ok(())
}
pub(crate) async fn claim_upload_session_completion(
    pool: &PgPool,
    upload_session: &UploadSessionRecord,
    operator_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let affected = sqlx::query(
        "UPDATE dr_drive_upload_session
         SET state='completing', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$2
           AND id=$3
           AND version=$4
           AND state IN ('created', 'uploading')",
    )
    .bind(operator_id)
    .bind(&upload_session.tenant_id)
    .bind(&upload_session.id)
    .bind(upload_session.version)
    .execute(pool)
    .await
    .map_err(internal_sql_error(
        "claim dr_drive_upload_session completion failed",
    ))?
    .rows_affected();
    if affected == 0 {
        return Err(problem(
            StatusCode::CONFLICT,
            "conflict",
            "upload session completion is already in progress or no longer mutable",
            SdkWorkResultCode::Conflict,
        ));
    }
    Ok(())
}

/// Compensates when storage finalize succeeded but the DB completion transaction failed.
///
/// Resets the session to `uploading` so the client can retry completion, and best-effort
/// deletes the orphaned storage object.
pub(crate) async fn recover_upload_completion_after_db_failure(
    state: &AppState,
    pool: &PgPool,
    tenant_id: &str,
    upload_session: &UploadSessionRecord,
    operator_id: &str,
) {
    if let Err(error) = update_upload_session_state(
        pool,
        tenant_id,
        &upload_session.id,
        "uploading",
        operator_id,
    )
    .await
    {
        tracing::error!(
            event = "drive.upload.completion_db_failure_session_reset_failed",
            tenant_id = %tenant_id,
            upload_session_id = %upload_session.id,
            error = ?error,
            "failed to reset upload session after DB completion failure"
        );
    }

    if let Ok(Some(provider)) =
        find_storage_provider_by_id(pool, &upload_session.storage_provider_id).await
    {
        match require_active_storage_provider(provider, &upload_session.bucket) {
            Ok(provider) => match build_s3_object_store_for_provider(&provider).await {
                Ok(Some(object_store)) => {
                    if let Err(error) = object_store
                        .delete_object(DeleteObjectRequest {
                            locator: DriveObjectLocator {
                                bucket: upload_session.bucket.clone(),
                                object_key: upload_session.object_key.clone(),
                            },
                        })
                        .await
                    {
                        tracing::warn!(
                            event = "drive.upload.completion_db_failure_orphan_delete_failed",
                            tenant_id = %tenant_id,
                            upload_session_id = %upload_session.id,
                            object_key = %upload_session.object_key,
                            error = ?error,
                            "failed to delete orphaned storage object after DB completion failure"
                        );
                    }
                }
                Ok(None) => tracing::warn!(
                    event = "drive.upload.completion_db_failure_orphan_delete_skipped",
                    tenant_id = %tenant_id,
                    upload_session_id = %upload_session.id,
                    "storage provider does not support object delete"
                ),
                Err(error) => tracing::warn!(
                    event = "drive.upload.completion_db_failure_orphan_delete_skipped",
                    tenant_id = %tenant_id,
                    upload_session_id = %upload_session.id,
                    error = ?error,
                    "object store unavailable; orphan object delete skipped"
                ),
            },
            Err(error) => tracing::warn!(
                event = "drive.upload.completion_db_failure_orphan_delete_skipped",
                tenant_id = %tenant_id,
                upload_session_id = %upload_session.id,
                error = ?error,
                "storage provider inactive; orphan object delete skipped"
            ),
        }
    }

    tracing::warn!(
        event = "drive.upload.completion_db_failure_recovered",
        tenant_id = %tenant_id,
        upload_session_id = %upload_session.id,
        object_key = %upload_session.object_key,
        "upload DB commit failed after storage finalize; session reset and orphan cleanup attempted"
    );
    let _ = state;
}

pub(crate) fn upload_session_state_as_str(state: &DriveUploadSessionState) -> &'static str {
    match state {
        DriveUploadSessionState::Created => "created",
        DriveUploadSessionState::Uploading => "uploading",
        DriveUploadSessionState::Completing => "completing",
        DriveUploadSessionState::Completed => "completed",
        DriveUploadSessionState::Aborted => "aborted",
        DriveUploadSessionState::Expired => "expired",
    }
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
    .map_err(internal_sql_error(
        "update uploader dr_drive_upload_session storage target failed",
    ))?;

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
    .map_err(internal_sql_error(
        "update uploader dr_drive_upload_item storage target failed",
    ))?;
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
    .map_err(internal_sql_error("find dr_drive_upload_item failed"))?;
    row.map(|row| map_uploader_upload_item_row(&row))
        .transpose()
}
pub(crate) async fn find_uploader_upload_item_by_session(
    pool: &PgPool,
    tenant_id: &str,
    upload_session_id: &str,
) -> Result<Option<DriveUploadItem>, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {DRIVE_UPLOAD_ITEM_SELECT_COLUMNS}
         FROM dr_drive_upload_item
         WHERE tenant_id=$1 AND upload_session_id=$2",
    )))
    .bind(tenant_id)
    .bind(upload_session_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "find dr_drive_upload_item by session failed",
    ))?;
    row.map(|row| map_uploader_upload_item_row(&row))
        .transpose()
}
pub(crate) async fn complete_uploader_upload_item(
    connection: &mut PgConnection,
    completed: CompletedUploaderUploadItem<'_>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    sqlx::query(
        "UPDATE dr_drive_upload_item
         SET status='completed',
             checksum_sha256_hex=$1,
             uploaded_bytes=$2,
             uploaded_parts_count=$3,
             updated_by=$4,
             updated_at=CURRENT_TIMESTAMP
         WHERE tenant_id=$5
           AND id=$6
           AND upload_session_id=$7",
    )
    .bind(completed.checksum_sha256_hex)
    .bind(completed.content_length)
    .bind(completed.uploaded_parts_count)
    .bind(completed.operator_id)
    .bind(completed.tenant_id)
    .bind(&completed.upload_item.id)
    .bind(&completed.upload_session.id)
    .execute(&mut *connection)
    .await
    .map_err(internal_sql_error("complete dr_drive_upload_item failed"))?;

    record_uploader_upload_completed_operation(
        connection,
        UploaderUploadCompletedOperation {
            tenant_id: completed.tenant_id,
            upload_item: completed.upload_item,
            storage_object: completed.storage_object,
            upload_session: completed.upload_session,
            content_type: completed.content_type,
            content_length: completed.content_length,
            checksum_sha256_hex: completed.checksum_sha256_hex,
            operator_id: completed.operator_id,
            before_content_state: completed.before_content_state,
        },
    )
    .await
}
pub(crate) async fn record_uploader_upload_completed_operation(
    connection: &mut PgConnection,
    operation: UploaderUploadCompletedOperation<'_>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    sqlx::query(
        "INSERT INTO dr_drive_file_sensitive_operation (
            id, tenant_id, organization_id, user_id, space_id, node_id,
            storage_object_id, upload_item_id, operation_type, operation_reason,
            content_type, content_type_group, content_length, checksum_sha256_hex,
            object_bucket, object_key, before_lifecycle_status, after_lifecycle_status,
            operator_id, maintenance_job_id, request_id, trace_id, object_delete_status
         ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, 'upload_completed', 'user_request',
            $9, $10, $11, $12,
            $13, $14, $15, 'active',
            $16, NULL, NULL, NULL, 'not_required'
         )",
    )
    .bind(sensitive_operation_id(
        operation.tenant_id,
        "upload_completed",
        &operation.upload_item.id,
        &operation.storage_object.id,
    ))
    .bind(operation.tenant_id)
    .bind(&operation.upload_item.organization_id)
    .bind(&operation.upload_item.user_id)
    .bind(&operation.upload_item.space_id)
    .bind(&operation.upload_item.node_id)
    .bind(&operation.storage_object.id)
    .bind(&operation.upload_item.id)
    .bind(operation.content_type)
    .bind(&operation.upload_item.content_type_group)
    .bind(operation.content_length)
    .bind(operation.checksum_sha256_hex)
    .bind(&operation.upload_session.bucket)
    .bind(&operation.upload_session.object_key)
    .bind(operation.before_content_state)
    .bind(operation.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(internal_sql_error(
        "insert dr_drive_file_sensitive_operation upload_completed failed",
    ))?;
    Ok(())
}

pub(crate) struct CompletedUploaderUploadItem<'a> {
    pub(crate) tenant_id: &'a str,
    pub(crate) upload_item: &'a DriveUploadItem,
    pub(crate) storage_object: &'a CompletedStorageObjectInsertPlan,
    pub(crate) upload_session: &'a UploadSessionRecord,
    pub(crate) content_type: &'a str,
    pub(crate) content_length: i64,
    pub(crate) checksum_sha256_hex: &'a str,
    pub(crate) uploaded_parts_count: i64,
    pub(crate) operator_id: &'a str,
    pub(crate) before_content_state: &'a str,
}

pub(crate) struct UploaderUploadCompletedOperation<'a> {
    tenant_id: &'a str,
    upload_item: &'a DriveUploadItem,
    storage_object: &'a CompletedStorageObjectInsertPlan,
    upload_session: &'a UploadSessionRecord,
    content_type: &'a str,
    content_length: i64,
    checksum_sha256_hex: &'a str,
    operator_id: &'a str,
    before_content_state: &'a str,
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

pub(crate) fn sensitive_operation_id(
    tenant_id: &str,
    operation_type: &str,
    upload_item_id: &str,
    storage_object_id: &str,
) -> String {
    let raw = format!("{tenant_id}:{operation_type}:{upload_item_id}:{storage_object_id}");
    let digest = sha256_hex(raw.as_bytes());
    digest
        .strip_prefix("sha256:")
        .expect("sha256_hex should include prefix")
        .to_string()
}
