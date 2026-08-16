use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_workspace_service::application::website_root_service::{
    CreateWebsiteRootCommand, DriveWebsiteRootService, GetWebsiteRootCommand,
    ListWebsiteRootsCommand,
};
use sdkwork_drive_workspace_service::domain::website_root::{
    DriveWebsiteContentMode, DriveWebsiteRoot, DriveWebsiteSourceRootMode,
};
use sdkwork_drive_workspace_service::infrastructure::sql::website_root_store::SqlWebsiteRootStore;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkPageData, SdkWorkResourceData};

use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::dto::{
    CreateWebsiteRootRequest, ListWebsiteRootsQuery, WebsiteRootResponse,
    WebsiteRootSelectorRequest,
};
use crate::error::{map_service_error, problem, ProblemDetail, SdkWorkResultCode};
use crate::response::{success_list_page_simple, success_resource};
use crate::state::AppState;
use crate::validators::{next_page_token, parse_page_request};

pub(crate) async fn create_website_root(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(space_id): Path<String>,
    Json(payload): Json<CreateWebsiteRootRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<WebsiteRootResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    acl::ensure_space_owner(&state.pool, &ctx, &space_id).await?;
    let (source_root_mode, selected_folder_node_id) = match payload.source_root {
        WebsiteRootSelectorRequest::SpaceRoot => (DriveWebsiteSourceRootMode::SpaceRoot, None),
        WebsiteRootSelectorRequest::Folder { folder_node_id } => {
            (DriveWebsiteSourceRootMode::Folder, Some(folder_node_id))
        }
    };
    let content_mode = parse_content_mode(&payload.content_mode)?;
    let operator_id = ctx.resolve_operator_id()?;
    let result = DriveWebsiteRootService::new(SqlWebsiteRootStore::new(state.pool.clone()))
        .create_root(CreateWebsiteRootCommand {
            tenant_id,
            space_id,
            root_key: payload.root_key,
            display_name: payload.display_name,
            source_root_mode,
            selected_folder_node_id,
            content_mode,
            operator_id,
        })
        .await
        .map_err(map_service_error)?;
    let status = if result.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, success_resource(map_website_root(result.root))))
}

pub(crate) async fn list_website_roots(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(space_id): Path<String>,
    Query(query): Query<ListWebsiteRootsQuery>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkPageData<WebsiteRootResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    acl::ensure_subject_space_scoped_reader(
        &state.pool,
        &tenant_id,
        &space_id,
        None,
        &subject_type,
        &subject_id,
    )
    .await?;
    let page = parse_page_request(query.page_size, query.cursor)?;
    let mut roots = DriveWebsiteRootService::new(SqlWebsiteRootStore::new(state.pool.clone()))
        .list_roots(ListWebsiteRootsCommand {
            tenant_id,
            space_id,
            offset: page.offset,
            limit: page.limit + 1,
        })
        .await
        .map_err(map_service_error)?
        .into_iter()
        .map(map_website_root)
        .collect::<Vec<_>>();
    let cursor = next_page_token(&mut roots, page);
    Ok(success_list_page_simple(roots, page, cursor))
}

pub(crate) async fn get_website_root(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(root_uuid): Path<String>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<WebsiteRootResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let root = DriveWebsiteRootService::new(SqlWebsiteRootStore::new(state.pool.clone()))
        .get_root(GetWebsiteRootCommand {
            tenant_id: tenant_id.clone(),
            root_uuid,
        })
        .await
        .map_err(map_service_error)?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    acl::ensure_subject_space_scoped_reader(
        &state.pool,
        &tenant_id,
        &root.space_id,
        None,
        &subject_type,
        &subject_id,
    )
    .await?;
    Ok(success_resource(map_website_root(root)))
}

fn parse_content_mode(
    value: &str,
) -> Result<DriveWebsiteContentMode, (StatusCode, Json<ProblemDetail>)> {
    match value.trim() {
        "LIVE_TREE" => Ok(DriveWebsiteContentMode::LiveTree),
        "ATOMIC_GENERATION" => Ok(DriveWebsiteContentMode::AtomicGeneration),
        _ => Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "contentMode must be LIVE_TREE or ATOMIC_GENERATION",
            SdkWorkResultCode::InvalidParameter,
        )),
    }
}

pub(crate) fn map_website_root(root: DriveWebsiteRoot) -> WebsiteRootResponse {
    WebsiteRootResponse {
        uuid: root.uuid,
        space_id: root.space_id,
        root_key: root.root_key,
        display_name: root.display_name,
        source_root_mode: match root.source_root_mode {
            DriveWebsiteSourceRootMode::SpaceRoot => "SPACE_ROOT",
            DriveWebsiteSourceRootMode::Folder => "FOLDER",
        }
        .to_string(),
        selected_folder_node_id: root.selected_folder_node_id,
        content_mode: match root.content_mode {
            DriveWebsiteContentMode::LiveTree => "LIVE_TREE",
            DriveWebsiteContentMode::AtomicGeneration => "ATOMIC_GENERATION",
        }
        .to_string(),
        active_node_id: root.active_node_id,
        active_generation: root.active_generation.to_string(),
        root_status: root.root_status.to_ascii_uppercase(),
        version: root.version.to_string(),
        created_at: root.created_at,
        updated_at: root.updated_at,
    }
}
