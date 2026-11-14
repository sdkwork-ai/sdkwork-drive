use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_contract::api::pagination_cursor::encode_offset_cursor;
use sdkwork_drive_security::DriveAppContext;
use sdkwork_drive_workspace_service::application::admin_file_service::{
    DriveAdminFileService, ListAdminFilesCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::admin_file_store::SqlAdminFileStore;

use crate::dto::{FileItemResponse, ListFilesQuery};
use crate::error::{map_service_error, ProblemDetail};
use crate::mappers::map_admin_file;
use crate::response::{success_list_page_simple, DriveListHttpResponse};
use crate::state::BackendState;
use crate::tenant_context::authenticated_tenant_id;
use crate::validators::{normalize_optional_text, parse_offset_page};

/// Tenant-scoped admin file list.
///
/// The tenant always comes from the authenticated context, never from a query
/// parameter: an operator can narrow the result with `spaceId`/`ownerSubjectId`
/// but cannot read another tenant's nodes. Filters are validated by the
/// workspace service, which owns the node vocabulary, so this handler stays a
/// translation layer between HTTP and the read model.
pub(crate) async fn list_files(
    State(state): State<BackendState>,
    Extension(app_context): Extension<DriveAppContext>,
    Query(query): Query<ListFilesQuery>,
) -> Result<DriveListHttpResponse<FileItemResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = authenticated_tenant_id(&app_context);
    let page = parse_offset_page(query.page_size, query.page_token)?;
    let service = DriveAdminFileService::new(SqlAdminFileStore::new(state.pool));
    let result = service
        .list_files(ListAdminFilesCommand {
            tenant_id,
            space_id: normalize_optional_text(query.space_id),
            space_type: normalize_optional_text(query.space_type),
            node_type: normalize_optional_text(query.node_type),
            lifecycle_status: normalize_optional_text(query.lifecycle_status),
            content_type_group: normalize_optional_text(query.content_type_group),
            keyword: normalize_optional_text(query.keyword),
            owner_subject_id: normalize_optional_text(query.owner_subject_id),
            offset: page.offset,
            limit: page.limit,
        })
        .await
        .map_err(map_service_error)?;

    let next_page_token = result.next_offset.and_then(encode_offset_cursor);
    let items = result
        .items
        .into_iter()
        .map(map_admin_file)
        .collect::<Vec<_>>();
    Ok(success_list_page_simple(items, page, next_page_token))
}
