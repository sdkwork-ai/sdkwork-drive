use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use sdkwork_drive_workspace_service::application::website_root_service::{
    DriveWebsiteRootService, GetWebsiteRootCommand,
};
use sdkwork_drive_workspace_service::application::website_sync_service::{
    AbortWebsiteSyncCommand, ActivateWebsiteGenerationCommand, CreateWebsiteSyncCommand,
    DriveWebsiteSyncService, FinalizeWebsiteSyncCommand, GetWebsiteSyncCommand,
};
use sdkwork_drive_workspace_service::domain::website_root::DriveWebsiteRoot;
use sdkwork_drive_workspace_service::domain::website_sync::{
    DriveWebsiteGeneration, DriveWebsiteSync,
};
use sdkwork_drive_workspace_service::infrastructure::sql::website_root_store::SqlWebsiteRootStore;
use sdkwork_drive_workspace_service::infrastructure::sql::website_sync_store::SqlWebsiteSyncStore;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};

use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::dto::{
    ActivateWebsiteGenerationRequest, CreateWebsiteSyncRequest,
    WebsiteGenerationActivationResponse, WebsiteGenerationResponse, WebsiteSyncActivationResponse,
    WebsiteSyncResponse, WebsiteSyncVersionRequest,
};
use crate::error::{
    invalid_parameter_problem, map_service_error, missing_required_field_problem, ProblemDetail,
};
use crate::response::success_resource;
use crate::state::AppState;
use crate::website_root_handlers::map_website_root;

type ResourceResponse<T> =
    Result<Json<SdkWorkApiResponse<SdkWorkResourceData<T>>>, (StatusCode, Json<ProblemDetail>)>;

pub(crate) async fn create_website_sync(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(root_uuid): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<CreateWebsiteSyncRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<WebsiteSyncResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let root = load_root_and_ensure_role(&state, &ctx, &root_uuid, "writer").await?;
    let result = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(state.pool.clone()))
        .create_sync(CreateWebsiteSyncCommand {
            tenant_id: ctx.resolve_tenant_id()?,
            website_root_uuid: root_uuid,
            idempotency_key: required_idempotency_key(&headers)?,
            expected_root_version: parse_positive_i64(
                &payload.expected_root_version,
                "expectedRootVersion",
            )?,
            expected_generation: parse_positive_i64(
                &payload.expected_generation,
                "expectedGeneration",
            )?,
            manifest_sha256: payload.manifest_sha256,
            manifest_file_count: parse_positive_i64(
                &payload.manifest_file_count,
                "manifestFileCount",
            )?,
            manifest_total_bytes: parse_non_negative_i64(
                &payload.manifest_total_bytes,
                "manifestTotalBytes",
            )?,
            expires_at: payload.expires_at,
            operator_id: ctx.resolve_operator_id()?,
        })
        .await
        .map_err(map_service_error)?;
    debug_assert_eq!(result.sync.space_id, root.space_id);
    let status = if result.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, success_resource(map_sync(result.sync))))
}

pub(crate) async fn get_website_sync(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((root_uuid, sync_id)): Path<(String, String)>,
) -> ResourceResponse<WebsiteSyncResponse> {
    load_root_and_ensure_role(&state, &ctx, &root_uuid, "reader").await?;
    let sync = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(state.pool.clone()))
        .get_sync(GetWebsiteSyncCommand {
            tenant_id: ctx.resolve_tenant_id()?,
            website_root_uuid: root_uuid,
            sync_id,
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(map_sync(sync)))
}

pub(crate) async fn finalize_website_sync(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((root_uuid, sync_id)): Path<(String, String)>,
    Json(payload): Json<WebsiteSyncVersionRequest>,
) -> ResourceResponse<WebsiteSyncActivationResponse> {
    load_root_and_ensure_role(&state, &ctx, &root_uuid, "writer").await?;
    let activation = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(state.pool.clone()))
        .finalize_sync(FinalizeWebsiteSyncCommand {
            tenant_id: ctx.resolve_tenant_id()?,
            website_root_uuid: root_uuid,
            sync_id,
            expected_sync_version: parse_positive_i64(
                &payload.expected_version,
                "expectedVersion",
            )?,
            operator_id: ctx.resolve_operator_id()?,
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(WebsiteSyncActivationResponse {
        sync: map_sync(activation.sync),
        website_root: map_website_root(activation.website_root),
    }))
}

pub(crate) async fn abort_website_sync(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((root_uuid, sync_id)): Path<(String, String)>,
    Json(payload): Json<WebsiteSyncVersionRequest>,
) -> ResourceResponse<WebsiteSyncResponse> {
    load_root_and_ensure_role(&state, &ctx, &root_uuid, "writer").await?;
    let sync = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(state.pool.clone()))
        .abort_sync(AbortWebsiteSyncCommand {
            tenant_id: ctx.resolve_tenant_id()?,
            website_root_uuid: root_uuid,
            sync_id,
            expected_sync_version: parse_positive_i64(
                &payload.expected_version,
                "expectedVersion",
            )?,
            operator_id: ctx.resolve_operator_id()?,
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(map_sync(sync)))
}

pub(crate) async fn activate_website_generation(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((root_uuid, generation)): Path<(String, String)>,
    Json(payload): Json<ActivateWebsiteGenerationRequest>,
) -> ResourceResponse<WebsiteGenerationActivationResponse> {
    load_root_and_ensure_role(&state, &ctx, &root_uuid, "writer").await?;
    let activation = DriveWebsiteSyncService::new(SqlWebsiteSyncStore::new(state.pool.clone()))
        .activate_generation(ActivateWebsiteGenerationCommand {
            tenant_id: ctx.resolve_tenant_id()?,
            website_root_uuid: root_uuid,
            target_generation: parse_positive_i64(&generation, "generation")?,
            expected_root_version: parse_positive_i64(
                &payload.expected_root_version,
                "expectedRootVersion",
            )?,
            expected_generation: parse_positive_i64(
                &payload.expected_generation,
                "expectedGeneration",
            )?,
            operator_id: ctx.resolve_operator_id()?,
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(WebsiteGenerationActivationResponse {
        source_generation: map_generation(activation.source_generation),
        website_root: map_website_root(activation.website_root),
    }))
}

async fn load_root_and_ensure_role(
    state: &AppState,
    ctx: &DriveRequestContext,
    root_uuid: &str,
    role: &str,
) -> Result<DriveWebsiteRoot, (StatusCode, Json<ProblemDetail>)> {
    let root = DriveWebsiteRootService::new(SqlWebsiteRootStore::new(state.pool.clone()))
        .get_root(GetWebsiteRootCommand {
            tenant_id: ctx.resolve_tenant_id()?,
            root_uuid: root_uuid.to_string(),
        })
        .await
        .map_err(map_service_error)?;
    acl::ensure_ctx_node_role(&state.pool, ctx, &root.space_id, &root.active_node_id, role).await?;
    Ok(root)
}

fn required_idempotency_key(
    headers: &HeaderMap,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| missing_required_field_problem("Idempotency-Key header is required"))
}

fn parse_positive_i64(
    value: &str,
    field_name: &str,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    let value = parse_i64(value, field_name)?;
    if value < 1 {
        return Err(invalid_parameter_problem(format!(
            "{field_name} must be positive"
        )));
    }
    Ok(value)
}

fn parse_non_negative_i64(
    value: &str,
    field_name: &str,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    let value = parse_i64(value, field_name)?;
    if value < 0 {
        return Err(invalid_parameter_problem(format!(
            "{field_name} must not be negative"
        )));
    }
    Ok(value)
}

fn parse_i64(value: &str, field_name: &str) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    let value = value.trim();
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid_parameter_problem(format!(
            "{field_name} must be an int64 string"
        )));
    }
    value
        .parse::<i64>()
        .map_err(|_| invalid_parameter_problem(format!("{field_name} exceeds the int64 range")))
}

fn map_sync(sync: DriveWebsiteSync) -> WebsiteSyncResponse {
    WebsiteSyncResponse {
        id: sync.id,
        website_root_uuid: sync.website_root_uuid,
        space_id: sync.space_id,
        expected_root_version: sync.expected_root_version.to_string(),
        expected_generation: sync.expected_generation.to_string(),
        staging_node_id: sync.staging_node_id,
        manifest_sha256: sync.manifest_sha256,
        manifest_file_count: sync.manifest_file_count.to_string(),
        manifest_total_bytes: sync.manifest_total_bytes.to_string(),
        uploaded_file_count: sync.uploaded_file_count.to_string(),
        uploaded_total_bytes: sync.uploaded_total_bytes.to_string(),
        status: sync.status.as_str().to_ascii_uppercase(),
        expires_at: sync.expires_at,
        validated_at: sync.validated_at,
        activated_at: sync.activated_at,
        completed_at: sync.completed_at,
        error_code: sync.error_code,
        error_summary: sync.error_summary,
        version: sync.version.to_string(),
        created_at: sync.created_at,
        updated_at: sync.updated_at,
    }
}

fn map_generation(generation: DriveWebsiteGeneration) -> WebsiteGenerationResponse {
    WebsiteGenerationResponse {
        generation: generation.generation_no.to_string(),
        root_node_id: generation.root_node_id,
        manifest_sha256: generation.manifest_sha256,
        file_count: generation.file_count.to_string(),
        total_bytes: generation.total_bytes.to_string(),
        status: generation.generation_status.to_ascii_uppercase(),
        activated_at: generation.activated_at,
    }
}
