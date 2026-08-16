use crate::app_context::DriveRequestContext;
use crate::audit::record_audit_event;
use crate::dto::{
    DefaultStorageProviderBindingQuery, DeleteDefaultStorageProviderBindingQuery,
    ListStorageProviderBindingsQuery, SetDefaultStorageProviderBindingRequest,
    StorageProviderBindingResponse,
};
use crate::error::{
    invalid_json_problem, map_service_error, not_found_binding_problem, ProblemDetail,
};
use crate::provider_lookup::get_provider;
use crate::provider_mappers::map_storage_provider;
use crate::response::{no_content, success_list_page_simple, StorageListHttpResponse};
use crate::state::AdminStorageState;
use crate::validators::{
    default_storage_provider_binding_id, next_page_token, normalize_optional_text,
    normalize_storage_root_prefix, parse_offset_page, require_non_empty_text,
    resolve_storage_provider_binding_target, storage_provider_binding_purpose,
    storage_provider_binding_scope, validate_storage_binding_lifecycle_status,
    StorageProviderBindingTarget,
};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProvider;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::Row;

pub(crate) async fn get_default_storage_provider_binding(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<DefaultStorageProviderBindingQuery>,
) -> Result<Json<StorageProviderBindingResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let target = resolve_storage_provider_binding_target(query.space_id, query.space_type)?;
    let binding = find_storage_provider_binding(&state, &tenant_id, &target)
        .await?
        .ok_or_else(not_found_binding_problem)?;
    Ok(Json(binding))
}

pub(crate) async fn list_storage_provider_bindings(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<ListStorageProviderBindingsQuery>,
) -> Result<
    StorageListHttpResponse<StorageProviderBindingResponse>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let space_id = normalize_optional_text(query.space_id);
    let provider_id = normalize_optional_text(query.provider_id);
    let lifecycle_status =
        normalize_optional_text(query.lifecycle_status).unwrap_or_else(|| "active".to_string());
    validate_storage_binding_lifecycle_status(&lifecycle_status)?;

    let rows = sqlx::query(
        "SELECT id, tenant_id, space_id, provider_id, binding_scope, purpose,
                storage_root_prefix, lifecycle_status, version
         FROM dr_drive_storage_provider_binding
         WHERE tenant_id=$1
           AND lifecycle_status=$2
           AND ($3 IS NULL OR space_id=$3)
           AND ($4 IS NULL OR provider_id=$4)
         ORDER BY
           CASE binding_scope
             WHEN 'space' THEN 0
             WHEN 'space_type' THEN 1
             ELSE 2
           END ASC,
           COALESCE(space_id, purpose, '') ASC,
           id ASC
         LIMIT $5 OFFSET $6",
    )
    .bind(&tenant_id)
    .bind(&lifecycle_status)
    .bind(space_id.as_deref())
    .bind(provider_id.as_deref())
    .bind(page.limit + 1)
    .bind(page.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "list dr_drive_storage_provider_binding failed: {error}"
        )))
    })?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(map_storage_provider_binding_row(&state, &row).await?);
    }
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn set_default_storage_provider_binding(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    payload: Result<Json<SetDefaultStorageProviderBindingRequest>, JsonRejection>,
) -> Result<Json<StorageProviderBindingResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let provider_id = require_non_empty_text(payload.provider_id, "providerId")?;
    let operator_id = ctx.resolve_operator_id()?;
    let target = resolve_storage_provider_binding_target(payload.space_id, payload.space_type)?;
    let storage_root_prefix =
        normalize_storage_root_prefix(payload.storage_root_prefix, &tenant_id, &target)?;
    let binding_scope = storage_provider_binding_scope(&target);
    let purpose = storage_provider_binding_purpose(&target);
    let provider = get_provider(&state, &provider_id).await?;
    if provider.status != "active" {
        return Err(map_service_error(DriveServiceError::Conflict(
            "default storage provider must be active".to_string(),
        )));
    }
    if let StorageProviderBindingTarget::Space(space_id) = &target {
        validate_space_exists(&state, &tenant_id, space_id).await?;
    }
    let binding_id = default_storage_provider_binding_id(&tenant_id, &target);
    let space_id = match &target {
        StorageProviderBindingTarget::Space(space_id) => Some(space_id.as_str()),
        _ => None,
    };
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', 1, $8, $8)
         ON CONFLICT(id) DO UPDATE SET
            provider_id=excluded.provider_id,
            binding_scope=excluded.binding_scope,
            purpose=excluded.purpose,
            storage_root_prefix=excluded.storage_root_prefix,
            lifecycle_status='active',
            version=dr_drive_storage_provider_binding.version + 1,
            updated_by=excluded.updated_by,
            updated_at=CURRENT_TIMESTAMP",
    )
    .bind(&binding_id)
    .bind(&tenant_id)
    .bind(space_id)
    .bind(&provider_id)
    .bind(binding_scope)
    .bind(&purpose)
    .bind(&storage_root_prefix)
    .bind(&operator_id)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "upsert dr_drive_storage_provider_binding failed: {error}"
        )))
    })?;
    let binding = find_storage_provider_binding(&state, &tenant_id, &target)
        .await?
        .ok_or_else(not_found_binding_problem)?;
    let audit_resource_id = binding_audit_resource_id(&tenant_id, &target);
    record_audit_event(
        &state,
        admin_audit::storage_provider_binding::DEFAULT_SET,
        "storage_provider_binding",
        audit_resource_id.as_str(),
        &operator_id,
    )
    .await?;
    Ok(Json(binding))
}

pub(crate) async fn delete_default_storage_provider_binding(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<DeleteDefaultStorageProviderBindingQuery>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let target = resolve_storage_provider_binding_target(query.space_id, query.space_type)?;
    let binding_id = default_storage_provider_binding_id(&tenant_id, &target);
    let purpose = storage_provider_binding_purpose(&target);
    let result = sqlx::query(
        "UPDATE dr_drive_storage_provider_binding
         SET lifecycle_status='deleted',
             version=version + 1,
             updated_by=$1,
             updated_at=CURRENT_TIMESTAMP
         WHERE tenant_id=$2
           AND id=$3
           AND purpose=$4
           AND lifecycle_status != 'deleted'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&binding_id)
    .bind(&purpose)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "delete dr_drive_storage_provider_binding failed: {error}"
        )))
    })?;
    let deleted = result.rows_affected() > 0;
    if deleted {
        let audit_resource_id = binding_audit_resource_id(&tenant_id, &target);
        record_audit_event(
            &state,
            admin_audit::storage_provider_binding::DEFAULT_DELETED,
            "storage_provider_binding",
            audit_resource_id.as_str(),
            &operator_id,
        )
        .await?;
    }
    let _deleted = deleted;
    Ok(no_content())
}

fn binding_audit_resource_id(tenant_id: &str, target: &StorageProviderBindingTarget) -> String {
    match target {
        StorageProviderBindingTarget::Tenant => tenant_id.to_string(),
        StorageProviderBindingTarget::Space(space_id) => space_id.clone(),
        StorageProviderBindingTarget::SpaceType(space_type) => space_type.clone(),
    }
}

pub(crate) async fn validate_space_exists(
    state: &AdminStorageState,
    tenant_id: &str,
    space_id: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_space
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "validate dr_drive_space failed: {error}"
        )))
    })?;
    if count == 0 {
        return Err(map_service_error(DriveServiceError::NotFound(
            "space not found".to_string(),
        )));
    }
    Ok(())
}

async fn find_storage_provider_binding(
    state: &AdminStorageState,
    tenant_id: &str,
    target: &StorageProviderBindingTarget,
) -> Result<Option<StorageProviderBindingResponse>, (StatusCode, Json<ProblemDetail>)> {
    let binding_id = default_storage_provider_binding_id(tenant_id, target);
    let rows = sqlx::query(
        "SELECT id, tenant_id, space_id, provider_id, binding_scope, purpose,
                storage_root_prefix, lifecycle_status, version
         FROM dr_drive_storage_provider_binding
         WHERE tenant_id=$1
           AND id=$2
           AND lifecycle_status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(&binding_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "find dr_drive_storage_provider_binding failed: {error}"
        )))
    })?;

    let Some(row) = rows.first() else {
        return Ok(None);
    };
    let provider_id: String = row.get("provider_id");
    let provider = get_provider(state, &provider_id).await?;
    Ok(Some(map_storage_provider_binding_row_with_provider(
        row, provider,
    )))
}

pub(crate) async fn map_storage_provider_binding_row(
    state: &AdminStorageState,
    row: &sqlx::postgres::PgRow,
) -> Result<StorageProviderBindingResponse, (StatusCode, Json<ProblemDetail>)> {
    let provider_id: String = row.get("provider_id");
    let provider = get_provider(state, &provider_id).await?;
    Ok(map_storage_provider_binding_row_with_provider(
        row, provider,
    ))
}

fn map_storage_provider_binding_row_with_provider(
    row: &sqlx::postgres::PgRow,
    provider: DriveStorageProvider,
) -> StorageProviderBindingResponse {
    StorageProviderBindingResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        provider_id: provider.id.clone(),
        binding_scope: row.get("binding_scope"),
        purpose: row.get("purpose"),
        storage_root_prefix: row.get("storage_root_prefix"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        storage_provider: map_storage_provider(provider),
    }
}
