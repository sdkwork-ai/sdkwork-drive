use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::Row;

use crate::app_context::DriveRequestContext;
use crate::audit::record_audit_event;
use crate::dto::{
    BusinessStorageBindingResponse, ListBusinessStorageBindingsQuery,
    ListStorageLogicalBucketsQuery, StorageLogicalBucketResponse,
    UpdateBusinessStorageBindingRequest, UpdateStorageLogicalBucketRequest,
};
use crate::error::{invalid_json_problem, map_service_error, validation_problem, ProblemDetail};
use crate::provider_lookup::get_provider;
use crate::response::{
    success_item, success_list_page_simple, StorageItemHttpResponse, StorageListHttpResponse,
};
use crate::state::AdminStorageState;
use crate::validators::{
    next_page_token, normalize_optional_text, parse_offset_page, require_non_empty_text,
    validate_object_prefix, validate_storage_binding_space_type,
};

const LOGICAL_BUCKET_SCOPE_TYPES: &[&str] = &["tenant", "space_type", "logical_scope"];

/// Tenant-scoped logical storage buckets, including the provider currently
/// bound to each one.
pub(crate) async fn list_storage_logical_buckets(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<ListStorageLogicalBucketsQuery>,
) -> Result<StorageListHttpResponse<StorageLogicalBucketResponse>, (StatusCode, Json<ProblemDetail>)>
{
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let rows = sqlx::query(
        "SELECT bucket.id, bucket.tenant_id, bucket.bucket_key, bucket.display_name,
                bucket.object_prefix, bucket.lifecycle_status, bucket.version,
                binding.id AS provider_binding_id,
                binding.provider_id,
                binding.storage_root_prefix
         FROM dr_drive_logical_bucket bucket
         LEFT JOIN dr_drive_logical_bucket_provider_binding binding
           ON binding.tenant_id = bucket.tenant_id
          AND binding.logical_bucket_id = bucket.id
          AND binding.lifecycle_status = 'active'
         WHERE bucket.tenant_id = $1
           AND bucket.lifecycle_status <> 'deleted'
         ORDER BY bucket.bucket_key ASC
         LIMIT $2 OFFSET $3",
    )
    .bind(&tenant_id)
    .bind(page.limit + 1)
    .bind(page.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "list dr_drive_logical_bucket failed: {error}"
        )))
    })?;

    let mut items = rows.iter().map(map_logical_bucket_row).collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

/// Update a logical bucket and/or move it to another provider configuration.
pub(crate) async fn update_storage_logical_bucket(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(logical_bucket_id): Path<String>,
    payload: Result<Json<UpdateStorageLogicalBucketRequest>, JsonRejection>,
) -> Result<StorageItemHttpResponse<StorageLogicalBucketResponse>, (StatusCode, Json<ProblemDetail>)>
{
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let logical_bucket_id = require_non_empty_text(logical_bucket_id, "logicalBucketId")?;

    let existing = find_logical_bucket(&state, &tenant_id, &logical_bucket_id).await?;
    let display_name = match payload.display_name {
        Some(value) => {
            let value = require_non_empty_text(value, "displayName")?;
            if value.chars().count() > 255 {
                return Err(validation_problem(
                    "displayName must be at most 255 characters",
                ));
            }
            value
        }
        None => existing.display_name,
    };
    let object_prefix = match payload.object_prefix {
        Some(value) => validate_object_prefix(Some(value), "objectPrefix")?.unwrap_or_default(),
        None => existing.object_prefix,
    };
    if object_prefix.len() > 512 {
        return Err(validation_problem(
            "objectPrefix must be at most 512 UTF-8 bytes",
        ));
    }
    let lifecycle_status = match normalize_optional_text(payload.lifecycle_status) {
        Some(status) if matches!(status.as_str(), "active" | "archived") => status,
        Some(_) => {
            return Err(validation_problem(
                "lifecycleStatus must be active or archived",
            ))
        }
        None => existing.lifecycle_status,
    };

    if let Some(provider_id) = normalize_optional_text(payload.provider_id) {
        let provider = get_provider(&state, &tenant_id, &provider_id).await?;
        if provider.status != "active" {
            return Err(map_service_error(DriveServiceError::Conflict(
                "logical bucket provider must be active".to_string(),
            )));
        }
        let storage_root_prefix = match normalize_optional_text(payload.storage_root_prefix) {
            Some(prefix) => {
                let prefix = validate_object_prefix(Some(prefix), "storageRootPrefix")?
                    .ok_or_else(|| validation_problem("storageRootPrefix is required"))?;
                if prefix.len() > 512 {
                    return Err(validation_problem(
                        "storageRootPrefix must be at most 512 UTF-8 bytes",
                    ));
                }
                prefix
            }
            None => existing
                .storage_root_prefix
                .clone()
                .unwrap_or_else(|| format!("sdkwork-drive/v1/tenants/{tenant_id}")),
        };
        let provider_binding_id = existing.provider_binding_id.clone().unwrap_or_else(|| {
            format!(
                "logical-bucket-provider:{tenant_id}:{}",
                existing.bucket_key
            )
        });
        sqlx::query(
            "INSERT INTO dr_drive_logical_bucket_provider_binding (
                id, tenant_id, logical_bucket_id, provider_id, storage_root_prefix,
                lifecycle_status, version, created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, 'active', 1, $6, $6)
             ON CONFLICT(id) DO UPDATE SET
                provider_id = excluded.provider_id,
                storage_root_prefix = excluded.storage_root_prefix,
                lifecycle_status = 'active',
                version = dr_drive_logical_bucket_provider_binding.version + 1,
                updated_by = excluded.updated_by,
                updated_at = CURRENT_TIMESTAMP",
        )
        .bind(&provider_binding_id)
        .bind(&tenant_id)
        .bind(&logical_bucket_id)
        .bind(&provider.id)
        .bind(&storage_root_prefix)
        .bind(&operator_id)
        .execute(&state.pool)
        .await
        .map_err(|error| {
            map_service_error(DriveServiceError::Internal(format!(
                "upsert dr_drive_logical_bucket_provider_binding failed: {error}"
            )))
        })?;
    }

    sqlx::query(
        "UPDATE dr_drive_logical_bucket
         SET display_name = $1,
             object_prefix = $2,
             lifecycle_status = $3,
             version = version + 1,
             updated_by = $4,
             updated_at = CURRENT_TIMESTAMP
         WHERE tenant_id = $5 AND id = $6 AND lifecycle_status <> 'deleted'",
    )
    .bind(&display_name)
    .bind(&object_prefix)
    .bind(&lifecycle_status)
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&logical_bucket_id)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "update dr_drive_logical_bucket failed: {error}"
        )))
    })?;

    record_audit_event(
        &state,
        admin_audit::logical_bucket::UPDATED,
        "storage_logical_bucket",
        &logical_bucket_id,
        &operator_id,
        &tenant_id,
    )
    .await?;

    let updated = find_logical_bucket(&state, &tenant_id, &logical_bucket_id).await?;
    Ok(success_item(updated))
}

/// Business scopes (tenant, space type, logical scope) bound to a logical bucket.
pub(crate) async fn list_business_storage_bindings(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<ListBusinessStorageBindingsQuery>,
) -> Result<
    StorageListHttpResponse<BusinessStorageBindingResponse>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let scope_type = normalize_optional_text(query.scope_type);
    if let Some(scope_type) = scope_type.as_deref() {
        validate_scope_type(scope_type)?;
    }
    let rows = sqlx::query(
        "SELECT binding.id, binding.tenant_id, binding.scope_type, binding.scope_id,
                binding.logical_bucket_id, binding.write_prefix,
                binding.lifecycle_status, binding.version,
                bucket.bucket_key AS logical_bucket_key,
                bucket.display_name AS logical_bucket_name,
                provider_binding.provider_id,
                provider_binding.storage_root_prefix
         FROM dr_drive_business_storage_binding binding
         INNER JOIN dr_drive_logical_bucket bucket
           ON bucket.id = binding.logical_bucket_id
          AND bucket.tenant_id = binding.tenant_id
          AND bucket.lifecycle_status <> 'deleted'
         LEFT JOIN dr_drive_logical_bucket_provider_binding provider_binding
           ON provider_binding.tenant_id = bucket.tenant_id
          AND provider_binding.logical_bucket_id = bucket.id
          AND provider_binding.lifecycle_status = 'active'
         WHERE binding.tenant_id = $1
           AND binding.lifecycle_status <> 'deleted'
           AND ($2 IS NULL OR binding.scope_type = $2)
         ORDER BY
           CASE binding.scope_type
             WHEN 'tenant' THEN 0
             WHEN 'space_type' THEN 1
             ELSE 2
           END ASC,
           binding.scope_id ASC
         LIMIT $3 OFFSET $4",
    )
    .bind(&tenant_id)
    .bind(scope_type)
    .bind(page.limit + 1)
    .bind(page.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "list dr_drive_business_storage_binding failed: {error}"
        )))
    })?;

    let mut items = rows
        .iter()
        .map(map_business_binding_row)
        .collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

/// Assign a business scope to a logical bucket. The provider is resolved
/// through that bucket, so this call never names a provider.
pub(crate) async fn update_business_storage_binding(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((scope_type, scope_id)): Path<(String, String)>,
    payload: Result<Json<UpdateBusinessStorageBindingRequest>, JsonRejection>,
) -> Result<
    StorageItemHttpResponse<BusinessStorageBindingResponse>,
    (StatusCode, Json<ProblemDetail>),
> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let scope_type = require_non_empty_text(scope_type, "scopeType")?;
    validate_scope_type(&scope_type)?;
    let scope_id = require_non_empty_text(scope_id, "scopeId")?;
    validate_scope_id(&scope_type, &scope_id)?;
    if scope_type == "tenant" && scope_id != tenant_id {
        return Err(validation_problem(
            "scopeId must equal tenantId for the tenant scope",
        ));
    }
    let logical_bucket_id = require_non_empty_text(payload.logical_bucket_id, "logicalBucketId")?;

    let bucket = find_logical_bucket(&state, &tenant_id, &logical_bucket_id).await?;
    if bucket.lifecycle_status != "active" {
        return Err(map_service_error(DriveServiceError::Conflict(
            "logical bucket must be active".to_string(),
        )));
    }

    let existing_binding_id = business_binding_id(&tenant_id, &scope_type, &scope_id);
    let existing_write_prefix: Option<String> = sqlx::query_scalar(
        "SELECT write_prefix FROM dr_drive_business_storage_binding
         WHERE tenant_id = $1 AND id = $2 AND lifecycle_status <> 'deleted'",
    )
    .bind(&tenant_id)
    .bind(&existing_binding_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "read existing business storage binding failed: {error}"
        )))
    })?;
    let write_prefix = match payload.write_prefix {
        Some(value) => validate_object_prefix(Some(value), "writePrefix")?.unwrap_or_default(),
        None => {
            existing_write_prefix.unwrap_or_else(|| default_write_prefix(&scope_type, &scope_id))
        }
    };
    if write_prefix.len() > 512 {
        return Err(validation_problem(
            "writePrefix must be at most 512 UTF-8 bytes",
        ));
    }

    sqlx::query(
        "INSERT INTO dr_drive_business_storage_binding (
            id, tenant_id, scope_type, scope_id, logical_bucket_id, write_prefix,
            lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $7)
         ON CONFLICT(id) DO UPDATE SET
            logical_bucket_id = excluded.logical_bucket_id,
            write_prefix = excluded.write_prefix,
            lifecycle_status = 'active',
            version = dr_drive_business_storage_binding.version + 1,
            updated_by = excluded.updated_by,
            updated_at = CURRENT_TIMESTAMP",
    )
    .bind(&existing_binding_id)
    .bind(&tenant_id)
    .bind(&scope_type)
    .bind(&scope_id)
    .bind(&logical_bucket_id)
    .bind(&write_prefix)
    .bind(&operator_id)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "upsert dr_drive_business_storage_binding failed: {error}"
        )))
    })?;

    record_audit_event(
        &state,
        admin_audit::business_storage_binding::UPDATED,
        "business_storage_binding",
        &existing_binding_id,
        &operator_id,
        &tenant_id,
    )
    .await?;

    let binding = find_business_binding(&state, &tenant_id, &existing_binding_id).await?;
    Ok(success_item(binding))
}

/// Remove a business scope's assignment. Writes fall through to the next rule
/// (space type then tenant logical binding), so deletion never makes a tenant
/// unwritable by itself.
pub(crate) async fn delete_business_storage_binding(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((scope_type, scope_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let scope_type = require_non_empty_text(scope_type, "scopeType")?;
    validate_scope_type(&scope_type)?;
    let scope_id = require_non_empty_text(scope_id, "scopeId")?;
    validate_scope_id(&scope_type, &scope_id)?;
    let binding_id = business_binding_id(&tenant_id, &scope_type, &scope_id);
    let result = sqlx::query(
        "UPDATE dr_drive_business_storage_binding
         SET lifecycle_status = 'deleted',
             version = version + 1,
             updated_by = $1,
             updated_at = CURRENT_TIMESTAMP
         WHERE tenant_id = $2 AND id = $3 AND lifecycle_status <> 'deleted'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&binding_id)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "delete dr_drive_business_storage_binding failed: {error}"
        )))
    })?;
    if result.rows_affected() == 0 {
        return Err(map_service_error(DriveServiceError::NotFound(
            "business storage binding not found".to_string(),
        )));
    }
    record_audit_event(
        &state,
        admin_audit::business_storage_binding::DELETED,
        "business_storage_binding",
        &binding_id,
        &operator_id,
        &tenant_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn find_logical_bucket(
    state: &AdminStorageState,
    tenant_id: &str,
    logical_bucket_id: &str,
) -> Result<StorageLogicalBucketResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT bucket.id, bucket.tenant_id, bucket.bucket_key, bucket.display_name,
                bucket.object_prefix, bucket.lifecycle_status, bucket.version,
                binding.id AS provider_binding_id,
                binding.provider_id,
                binding.storage_root_prefix
         FROM dr_drive_logical_bucket bucket
         LEFT JOIN dr_drive_logical_bucket_provider_binding binding
           ON binding.tenant_id = bucket.tenant_id
          AND binding.logical_bucket_id = bucket.id
          AND binding.lifecycle_status = 'active'
         WHERE bucket.tenant_id = $1
           AND bucket.id = $2
           AND bucket.lifecycle_status <> 'deleted'",
    )
    .bind(tenant_id)
    .bind(logical_bucket_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "read dr_drive_logical_bucket failed: {error}"
        )))
    })?;
    let Some(row) = row else {
        return Err(map_service_error(DriveServiceError::NotFound(
            "logical bucket not found".to_string(),
        )));
    };
    Ok(map_logical_bucket_row(&row))
}

async fn find_business_binding(
    state: &AdminStorageState,
    tenant_id: &str,
    binding_id: &str,
) -> Result<BusinessStorageBindingResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT binding.id, binding.tenant_id, binding.scope_type, binding.scope_id,
                binding.logical_bucket_id, binding.write_prefix,
                binding.lifecycle_status, binding.version,
                bucket.bucket_key AS logical_bucket_key,
                bucket.display_name AS logical_bucket_name,
                provider_binding.provider_id,
                provider_binding.storage_root_prefix
         FROM dr_drive_business_storage_binding binding
         INNER JOIN dr_drive_logical_bucket bucket
           ON bucket.id = binding.logical_bucket_id
          AND bucket.tenant_id = binding.tenant_id
          AND bucket.lifecycle_status <> 'deleted'
         LEFT JOIN dr_drive_logical_bucket_provider_binding provider_binding
           ON provider_binding.tenant_id = bucket.tenant_id
          AND provider_binding.logical_bucket_id = bucket.id
          AND provider_binding.lifecycle_status = 'active'
         WHERE binding.tenant_id = $1
           AND binding.id = $2
           AND binding.lifecycle_status <> 'deleted'",
    )
    .bind(tenant_id)
    .bind(binding_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "read dr_drive_business_storage_binding failed: {error}"
        )))
    })?;
    let Some(row) = row else {
        return Err(map_service_error(DriveServiceError::NotFound(
            "business storage binding not found".to_string(),
        )));
    };
    Ok(map_business_binding_row(&row))
}

fn map_logical_bucket_row(row: &sqlx::postgres::PgRow) -> StorageLogicalBucketResponse {
    StorageLogicalBucketResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        bucket_key: row.get("bucket_key"),
        display_name: row.get("display_name"),
        object_prefix: row.get("object_prefix"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        provider_id: row.try_get("provider_id").ok().flatten(),
        provider_binding_id: row.try_get("provider_binding_id").ok().flatten(),
        storage_root_prefix: row.try_get("storage_root_prefix").ok().flatten(),
    }
}

fn map_business_binding_row(row: &sqlx::postgres::PgRow) -> BusinessStorageBindingResponse {
    BusinessStorageBindingResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        scope_type: row.get("scope_type"),
        scope_id: row.get("scope_id"),
        logical_bucket_id: row.get("logical_bucket_id"),
        logical_bucket_key: row.get("logical_bucket_key"),
        logical_bucket_name: row.get("logical_bucket_name"),
        write_prefix: row.get("write_prefix"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        provider_id: row.try_get("provider_id").ok().flatten(),
        storage_root_prefix: row.try_get("storage_root_prefix").ok().flatten(),
    }
}

fn validate_scope_type(scope_type: &str) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if LOGICAL_BUCKET_SCOPE_TYPES.contains(&scope_type) {
        Ok(())
    } else {
        Err(validation_problem("scopeType is invalid"))
    }
}

fn validate_scope_id(
    scope_type: &str,
    scope_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    match scope_type {
        "space_type" => validate_storage_binding_space_type(scope_id),
        "tenant" => {
            if scope_id.is_empty() {
                return Err(validation_problem("scopeId is required"));
            }
            Ok(())
        }
        "logical_scope" => {
            let valid = scope_id.len() <= 64
                && scope_id.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
                });
            if valid {
                Ok(())
            } else {
                Err(validation_problem("scopeId is invalid"))
            }
        }
        _ => Err(validation_problem("scopeType is invalid")),
    }
}

fn business_binding_id(tenant_id: &str, scope_type: &str, scope_id: &str) -> String {
    format!("business-storage:{tenant_id}:{scope_type}:{scope_id}")
}

fn default_write_prefix(scope_type: &str, scope_id: &str) -> String {
    match scope_type {
        "space_type" => format!("space-types/{scope_id}"),
        "tenant" => "tenant".to_string(),
        _ => format!("logical-scopes/{scope_id}"),
    }
}
