use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_security::DriveAppContext;
use sdkwork_drive_workspace_service::application::sandbox_admin_service::{
    CreateSandboxAdminGrantCommand, CreateSandboxAdminVolumeCommand, DriveSandboxAdminService,
    InitialSandboxUserGrant, ListSandboxAdminGrantsCommand, ListSandboxAdminVolumesCommand,
    UpdateSandboxAdminGrantCommand, UpdateSandboxAdminVolumeCommand,
};
use sdkwork_drive_workspace_service::domain::sandbox_admin::{
    SandboxAdminGrant, SandboxAdminVolume,
};
use sdkwork_drive_workspace_service::infrastructure::sql::sandbox_admin_store::SqlSandboxAdminStore;
use sdkwork_utils_rust::OffsetListPageParams;

use crate::error::{invalid_json_problem, invalid_query_problem, map_service_error, ProblemDetail};
use crate::response::{
    no_content, success_offset_list_page, success_resource, DriveListHttpResponse,
    DriveResourceHttpResponse,
};
use crate::sandbox_admin_dto::{
    CreateSandboxGrantRequest, CreateSandboxVolumeRequest, ListSandboxGrantsQuery,
    ListSandboxVolumesQuery, SandboxGrantResponse, SandboxVolumeResponse,
    UpdateSandboxGrantRequest, UpdateSandboxVolumeRequest,
};
use crate::state::BackendState;

pub(crate) async fn list_sandbox_volumes(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    query: Result<Query<ListSandboxVolumesQuery>, QueryRejection>,
) -> Result<DriveListHttpResponse<SandboxVolumeResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Query(query) = query.map_err(invalid_query_problem)?;
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    let page = service
        .list_volumes(
            &context,
            ListSandboxAdminVolumesCommand {
                lifecycle_status: query.lifecycle_status,
                provider_kind: query.provider_kind,
                page: query.page,
                page_size: query.page_size,
            },
        )
        .await
        .map_err(map_service_error)?;
    let params = OffsetListPageParams {
        page: page.page,
        page_size: page.page_size,
        offset: (page.page - 1) * page.page_size,
    };
    Ok(success_offset_list_page(
        page.items.into_iter().map(map_volume).collect(),
        page.total_items,
        params,
    ))
}

pub(crate) async fn get_sandbox_volume(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    Path(sandbox_id): Path<String>,
) -> Result<DriveResourceHttpResponse<SandboxVolumeResponse>, (StatusCode, Json<ProblemDetail>)> {
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    let volume = service
        .get_volume(&context, &sandbox_id)
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(map_volume(volume)))
}

pub(crate) async fn create_sandbox_volume(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    payload: Result<Json<CreateSandboxVolumeRequest>, JsonRejection>,
) -> Result<
    (StatusCode, DriveResourceHttpResponse<SandboxVolumeResponse>),
    (StatusCode, Json<ProblemDetail>),
> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    let volume = service
        .create_volume(
            &context,
            CreateSandboxAdminVolumeCommand {
                display_name: payload.display_name,
                provider_kind: payload.provider_kind,
                provider_root_ref: payload.provider_root_ref,
                default_access: payload.default_access,
                initial_user_grant: payload.initial_user_grant.map(|grant| {
                    InitialSandboxUserGrant {
                        enabled: grant.enabled,
                        access_level: grant.access_level,
                    }
                }),
            },
        )
        .await
        .map_err(map_service_error)?;
    Ok((StatusCode::CREATED, success_resource(map_volume(volume))))
}

pub(crate) async fn update_sandbox_volume(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    Path(sandbox_id): Path<String>,
    payload: Result<Json<UpdateSandboxVolumeRequest>, JsonRejection>,
) -> Result<DriveResourceHttpResponse<SandboxVolumeResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    let volume = service
        .update_volume(
            &context,
            UpdateSandboxAdminVolumeCommand {
                sandbox_id,
                display_name: payload.display_name,
                provider_root_ref: payload.provider_root_ref,
                lifecycle_status: payload.lifecycle_status,
                default_access: payload.default_access,
                expected_version: payload.expected_version,
            },
        )
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(map_volume(volume)))
}

pub(crate) async fn delete_sandbox_volume(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    Path(sandbox_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    service
        .delete_volume(&context, &sandbox_id)
        .await
        .map_err(map_service_error)?;
    Ok(no_content())
}

pub(crate) async fn list_sandbox_grants(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    Path(sandbox_id): Path<String>,
    query: Result<Query<ListSandboxGrantsQuery>, QueryRejection>,
) -> Result<DriveListHttpResponse<SandboxGrantResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Query(query) = query.map_err(invalid_query_problem)?;
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    let page = service
        .list_grants(
            &context,
            ListSandboxAdminGrantsCommand {
                sandbox_id,
                page: query.page,
                page_size: query.page_size,
            },
        )
        .await
        .map_err(map_service_error)?;
    let params = OffsetListPageParams {
        page: page.page,
        page_size: page.page_size,
        offset: (page.page - 1) * page.page_size,
    };
    Ok(success_offset_list_page(
        page.items.into_iter().map(map_grant).collect(),
        page.total_items,
        params,
    ))
}

pub(crate) async fn create_sandbox_grant(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    Path(sandbox_id): Path<String>,
    payload: Result<Json<CreateSandboxGrantRequest>, JsonRejection>,
) -> Result<
    (StatusCode, DriveResourceHttpResponse<SandboxGrantResponse>),
    (StatusCode, Json<ProblemDetail>),
> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    let grant = service
        .create_grant(
            &context,
            CreateSandboxAdminGrantCommand {
                sandbox_id,
                subject_type: payload.grantee_type,
                subject_id: payload.grantee_id,
                access_level: payload.access_level,
            },
        )
        .await
        .map_err(map_service_error)?;
    Ok((StatusCode::CREATED, success_resource(map_grant(grant))))
}

pub(crate) async fn get_sandbox_grant(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    Path((sandbox_id, grant_id)): Path<(String, String)>,
) -> Result<DriveResourceHttpResponse<SandboxGrantResponse>, (StatusCode, Json<ProblemDetail>)> {
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    let grant = service
        .get_grant(&context, &sandbox_id, &grant_id)
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(map_grant(grant)))
}

pub(crate) async fn update_sandbox_grant(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    Path((sandbox_id, grant_id)): Path<(String, String)>,
    payload: Result<Json<UpdateSandboxGrantRequest>, JsonRejection>,
) -> Result<DriveResourceHttpResponse<SandboxGrantResponse>, (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    let grant = service
        .update_grant(
            &context,
            UpdateSandboxAdminGrantCommand {
                sandbox_id,
                grant_id,
                access_level: payload.access_level,
            },
        )
        .await
        .map_err(map_service_error)?;
    Ok(success_resource(map_grant(grant)))
}

pub(crate) async fn delete_sandbox_grant(
    State(state): State<BackendState>,
    Extension(context): Extension<DriveAppContext>,
    Path((sandbox_id, grant_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(state.pool));
    service
        .delete_grant(&context, &sandbox_id, &grant_id)
        .await
        .map_err(map_service_error)?;
    Ok(no_content())
}

fn map_volume(volume: SandboxAdminVolume) -> SandboxVolumeResponse {
    SandboxVolumeResponse {
        id: volume.id,
        tenant_id: volume.tenant_id,
        organization_id: volume.organization_id,
        display_name: volume.display_name,
        root_entry_id: volume.root_entry_id,
        provider_kind: volume.provider_kind,
        provider_root_ref: volume.provider_root_ref,
        lifecycle_status: volume.lifecycle_status,
        default_access: volume.default_access,
        version: volume.version,
        created_by: volume.created_by,
        updated_by: volume.updated_by,
        created_at: volume.created_at,
        updated_at: volume.updated_at,
    }
}

fn map_grant(grant: SandboxAdminGrant) -> SandboxGrantResponse {
    SandboxGrantResponse {
        id: grant.id,
        sandbox_id: grant.sandbox_id,
        subject_type: grant.subject_type,
        subject_id: grant.subject_id,
        access_level: grant.access_level,
        granted_by: grant.granted_by,
        created_at: grant.created_at,
    }
}
