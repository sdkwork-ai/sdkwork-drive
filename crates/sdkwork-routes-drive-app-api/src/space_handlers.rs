use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::dto::*;
use crate::error::{
    map_service_error, problem, service_error_kind, ProblemDetail, SdkWorkResultCode,
};
use crate::ids::next_drive_id;
use crate::mappers::*;
use crate::response::{
    no_content, success_created_resource, success_list_page_simple, success_resource,
    DriveListHttpResponse,
};
use crate::space_repository::validate_space_exists;
use crate::state::AppState;
use crate::validators::{next_page_token, normalize_optional_text, parse_page_request};
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_drive_observability::{elapsed_ms, error_kinds, events, has_value, start_timer};
use sdkwork_drive_workspace_service::application::space_lifecycle_service::{
    BootstrapTeamSpaceCreatorAccessCommand, DeleteSpaceWithContentsCommand,
    SqlDriveSpaceLifecycleService,
};
use sdkwork_drive_workspace_service::application::space_service::{
    CreateSpaceCommand, DriveSpaceService, GetSpaceCommand, ListAccessibleSpacesCommand,
    UpdateSpaceCommand,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::infrastructure::sql::space_store::SqlSpaceStore;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};

use crate::route_change::record_change;

pub(crate) async fn list_spaces(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<ListSpacesQuery>,
) -> Result<DriveListHttpResponse<CreateSpaceResponse>, (StatusCode, Json<ProblemDetail>)> {
    let started = start_timer();
    let filter_has_owner_subject_type = has_value(&query.owner_subject_type);
    let filter_has_owner_subject_id = has_value(&query.owner_subject_id);
    let tenant_id = match ctx.resolve_tenant_id() {
        Ok(tenant_id) => tenant_id,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_LIST,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = error_kinds::VALIDATION,
                filter_has_owner_subject_type = filter_has_owner_subject_type,
                filter_has_owner_subject_id = filter_has_owner_subject_id
            );
            return Err(error);
        }
    };

    let page = parse_page_request(query.page_size, query.page_token)?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let owner_subject_type = normalize_optional_text(query.owner_subject_type);
    let owner_subject_id = normalize_optional_text(query.owner_subject_id);
    let space_type = normalize_optional_text(query.space_type);
    let service = DriveSpaceService::new(SqlSpaceStore::new(state.pool.clone()));
    let spaces = service
        .list_accessible_spaces(ListAccessibleSpacesCommand {
            tenant_id: tenant_id.clone(),
            viewer_subject_type: subject_type,
            viewer_subject_id: subject_id,
            owner_subject_type,
            owner_subject_id,
            space_type,
            offset: page.offset,
            limit: page.limit + 1,
        })
        .await
        .map_err(|error| {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_LIST,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = service_error_kind(&error),
                filter_has_owner_subject_type = filter_has_owner_subject_type,
                filter_has_owner_subject_id = filter_has_owner_subject_id
            );
            map_service_error(error)
        })?;

    let mut items = spaces
        .into_iter()
        .map(map_space_response)
        .collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    let latency_ms = elapsed_ms(started);
    sdkwork_drive_observability::observe_route!(
        event = events::APP_SPACES_LIST,
        result = "ok",
        latency_ms = latency_ms,
        filter_has_owner_subject_type = filter_has_owner_subject_type,
        filter_has_owner_subject_id = filter_has_owner_subject_id,
        returned_items = items.len() as u64
    );

    Ok(success_list_page_simple(items, page, next_page_token))
}
pub(crate) async fn create_space(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Json(payload): Json<CreateSpaceRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<CreateSpaceResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let started = start_timer();
    let space_type = match DriveSpaceType::try_from_str(payload.space_type.trim()) {
        Some(space_type) => space_type,
        None => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_CREATE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = error_kinds::VALIDATION,
                input_space_type = payload.space_type.as_str()
            );
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "invalid space type",
                "space_type is invalid",
                SdkWorkResultCode::InvalidParameter,
            ));
        }
    };

    let service = DriveSpaceService::new(SqlSpaceStore::new(state.pool.clone()));
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    ensure_create_space_owner_matches_caller(
        &space_type,
        payload.id.trim(),
        &subject_type,
        &subject_id,
        payload.owner_subject_type.trim(),
        payload.owner_subject_id.trim(),
        ctx.organization_id.as_deref(),
    )?;
    let operator_id = ctx.resolve_operator_id()?;
    let tenant_id_for_bootstrap = tenant_id.clone();
    let operator_id_for_bootstrap = operator_id.clone();
    let created_result = service
        .create_space(CreateSpaceCommand {
            id: payload.id,
            tenant_id,
            owner_subject_type: payload.owner_subject_type,
            owner_subject_id: payload.owner_subject_id,
            display_name: payload.display_name,
            space_type,
            presentation_icon: payload.presentation_icon,
            presentation_color: payload.presentation_color,
            description: payload.description,
            operator_id,
        })
        .await;
    let created = match created_result {
        Ok(created) => created,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_CREATE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = service_error_kind(&error)
            );
            return Err(map_service_error(error));
        }
    };
    let latency_ms = elapsed_ms(started);
    sdkwork_drive_observability::observe_route!(
        event = events::APP_SPACES_CREATE,
        result = "ok",
        latency_ms = latency_ms,
        space_type = created.space_type.as_str(),
        lifecycle_status = created.lifecycle_status.as_str(),
        version = created.version
    );

    record_change(
        &state.pool,
        &created.tenant_id,
        &created.id,
        None,
        drive_events::space::CREATED,
        &operator_id_for_bootstrap,
    )
    .await?;

    if created.space_type == DriveSpaceType::Team {
        SqlDriveSpaceLifecycleService::new(state.pool.clone())
            .bootstrap_team_space_creator_access(BootstrapTeamSpaceCreatorAccessCommand {
                tenant_id: tenant_id_for_bootstrap,
                space_id: created.id.clone(),
                creator_user_id: operator_id_for_bootstrap,
                display_name: created.display_name.clone(),
                root_folder_id: next_drive_id("folder"),
            })
            .await
            .map_err(map_service_error)?;
    }

    Ok(success_created_resource(map_space_response(created)))
}
pub(crate) async fn get_space(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(space_id): Path<String>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<CreateSpaceResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let started = start_timer();
    let tenant_id = match ctx.resolve_tenant_id() {
        Ok(tenant_id) => tenant_id,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_GET,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = error_kinds::VALIDATION,
                space_id = space_id.as_str()
            );
            return Err(error);
        }
    };
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    validate_space_exists(&state.pool, &tenant_id, &space_id).await?;
    acl::ensure_subject_space_scoped_reader(
        &state.pool,
        &tenant_id,
        &space_id,
        None,
        &subject_type,
        &subject_id,
    )
    .await?;
    let service = DriveSpaceService::new(SqlSpaceStore::new(state.pool.clone()));
    let found = match service
        .get_space(GetSpaceCommand {
            tenant_id,
            space_id: space_id.clone(),
        })
        .await
    {
        Ok(found) => found,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_GET,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = service_error_kind(&error),
                space_id = space_id.as_str()
            );
            return Err(map_service_error(error));
        }
    };

    let response = map_space_response(found);
    sdkwork_drive_observability::observe_route!(
        event = events::APP_SPACES_GET,
        result = "ok",
        latency_ms = elapsed_ms(started),
        space_id = response.id.as_str(),
        space_type = response.space_type.as_str(),
        lifecycle_status = response.lifecycle_status.as_str(),
        version = response.version
    );
    Ok(success_resource(response))
}
pub(crate) async fn update_space(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(space_id): Path<String>,
    Json(payload): Json<UpdateSpaceRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<CreateSpaceResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let started = start_timer();
    let tenant_id = match ctx.resolve_tenant_id() {
        Ok(tenant_id) => tenant_id,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_UPDATE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = error_kinds::VALIDATION,
                space_id = space_id.as_str()
            );
            return Err(error);
        }
    };
    let operator_id = ctx.resolve_operator_id()?;
    validate_space_exists(&state.pool, &tenant_id, &space_id).await?;
    acl::ensure_parent_writer(&state.pool, &ctx, &space_id, None).await?;
    let service = DriveSpaceService::new(SqlSpaceStore::new(state.pool.clone()));
    let updated = match service
        .update_space(UpdateSpaceCommand {
            tenant_id: tenant_id.clone(),
            space_id: space_id.clone(),
            display_name: payload.display_name,
            presentation_icon: payload.presentation_icon,
            presentation_color: payload.presentation_color,
            description: payload.description,
            operator_id: operator_id.clone(),
        })
        .await
    {
        Ok(updated) => updated,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_UPDATE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = service_error_kind(&error),
                space_id = space_id.as_str()
            );
            return Err(map_service_error(error));
        }
    };
    record_change(
        &state.pool,
        &tenant_id,
        &space_id,
        None,
        drive_events::space::UPDATED,
        &operator_id,
    )
    .await?;

    let response = map_space_response(updated);
    sdkwork_drive_observability::observe_route!(
        event = events::APP_SPACES_UPDATE,
        result = "ok",
        latency_ms = elapsed_ms(started),
        space_id = response.id.as_str(),
        lifecycle_status = response.lifecycle_status.as_str(),
        version = response.version
    );
    Ok(success_resource(response))
}
pub(crate) async fn delete_space(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(space_id): Path<String>,
    Query(_query): Query<NodeMutationQuery>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let started = start_timer();
    let tenant_id = match ctx.resolve_tenant_id() {
        Ok(tenant_id) => tenant_id,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_DELETE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = error_kinds::VALIDATION,
                space_id = space_id.as_str()
            );
            return Err(error);
        }
    };
    let operator_id = ctx.resolve_operator_id()?;
    validate_space_exists(&state.pool, &tenant_id, &space_id).await?;
    acl::ensure_space_owner(&state.pool, &ctx, &space_id).await?;
    let lifecycle_service = SqlDriveSpaceLifecycleService::new(state.pool.clone());
    let delete_result = match lifecycle_service
        .delete_space_with_contents(DeleteSpaceWithContentsCommand {
            tenant_id: tenant_id.clone(),
            space_id: space_id.clone(),
            operator_id: operator_id.clone(),
        })
        .await
    {
        Ok(result) => result,
        Err(error) => {
            sdkwork_drive_observability::observe_route!(
                event = events::APP_SPACES_DELETE,
                result = "err",
                latency_ms = elapsed_ms(started),
                error_kind = service_error_kind(&error),
                space_id = space_id.as_str()
            );
            return Err(map_service_error(error));
        }
    };

    let response_space = map_space_response(delete_result.space);
    sdkwork_drive_observability::observe_route!(
        event = events::APP_SPACES_DELETE,
        result = "ok",
        latency_ms = elapsed_ms(started),
        space_id = response_space.id.as_str(),
        lifecycle_status = response_space.lifecycle_status.as_str(),
        version = response_space.version,
        deleted_node_count = delete_result.deleted_node_count
    );
    Ok(no_content())
}
fn ensure_create_space_owner_matches_caller(
    space_type: &DriveSpaceType,
    space_id: &str,
    subject_type: &str,
    subject_id: &str,
    owner_subject_type: &str,
    owner_subject_id: &str,
    organization_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let owner_must_match_caller = matches!(
        space_type,
        DriveSpaceType::Personal | DriveSpaceType::GitRepository | DriveSpaceType::Rtc
    );
    if owner_must_match_caller
        && (subject_type != owner_subject_type || subject_id != owner_subject_id)
    {
        return Err(acl::permission_denied_problem());
    }

    if *space_type == DriveSpaceType::Team {
        let organization_id = organization_id
            .map(str::trim)
            .filter(|value| !value.is_empty() && *value != "0")
            .ok_or_else(acl::permission_denied_problem)?;
        match owner_subject_type {
            "group" => {
                if owner_subject_id != space_id.trim() {
                    return Err(acl::permission_denied_problem());
                }
                if !space_id.starts_with(&format!("{organization_id}:"))
                    && !space_id.starts_with(&format!("{organization_id}-"))
                {
                    return Err(acl::permission_denied_problem());
                }
            }
            "organization" => {
                if owner_subject_id != organization_id {
                    return Err(acl::permission_denied_problem());
                }
            }
            _ => {
                return Err(problem(
                    StatusCode::BAD_REQUEST,
                    "validation failed",
                    "team spaces must bind ownerSubjectType to group or organization",
                    SdkWorkResultCode::InvalidParameter,
                ));
            }
        }
    }

    Ok(())
}
