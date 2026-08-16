use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::dto::{ChangeResponse, ChangesQuery, StartPageTokenQuery, StartPageTokenResponse};
use crate::error::{internal_problem, map_service_error, ProblemDetail};
use crate::response::{success_cursor_list_page, success_envelope, DriveListHttpResponse};
use crate::space_repository::{validate_space_exists, validate_space_exists_for_change_history};
use crate::state::AppState;
use crate::validators::{parse_change_page_request, require_query_value};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::api::pagination_cursor::encode_change_sequence_cursor;
use sdkwork_drive_workspace_service::application::change_feed_service::{
    ListChangesCommand, QueryStartPageTokenCommand, SqlDriveChangeFeedService,
};
use sdkwork_drive_workspace_service::domain::change::DriveChangeRecord;
use sdkwork_utils_rust::SdkWorkApiResponse;

pub(crate) async fn list_changes(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<ChangesQuery>,
) -> Result<DriveListHttpResponse<ChangeResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_change_page_request(query.page_size, query.cursor)?;
    let space_id = require_query_value(query.space_id, "spaceId")?;
    validate_space_exists_for_change_history(&state.pool, &tenant_id, &space_id).await?;
    acl::ensure_space_change_feed_reader(&state.pool, &ctx, &space_id).await?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let is_space_owner = acl::is_subject_space_owner(
        &state.pool,
        &tenant_id,
        &space_id,
        &subject_type,
        &subject_id,
    )
    .await?;
    let service = SqlDriveChangeFeedService::new(state.pool.clone());
    let tenant_id_for_fetch = tenant_id.clone();
    let space_id_for_fetch = space_id.clone();
    let subject_type_for_fetch = subject_type.clone();
    let subject_id_for_fetch = subject_id.clone();
    let (items, next_page_token) =
        acl::paginate_cursor_limited_changes(page, move |scan_cursor, batch_limit| {
            let service = service.clone();
            let tenant_id = tenant_id_for_fetch.clone();
            let space_id = space_id_for_fetch.clone();
            let subject_type = subject_type_for_fetch.clone();
            let subject_id = subject_id_for_fetch.clone();
            async move {
                let records = service
                    .list_changes(ListChangesCommand {
                        tenant_id,
                        space_id,
                        after_sequence: scan_cursor,
                        limit: batch_limit as i64,
                        subject_type: if is_space_owner {
                            None
                        } else {
                            Some(subject_type)
                        },
                        subject_id: if is_space_owner {
                            None
                        } else {
                            Some(subject_id)
                        },
                        is_space_owner,
                    })
                    .await
                    .map_err(map_service_error)?;
                Ok(records.into_iter().map(map_change_record).collect())
            }
        })
        .await?;
    Ok(success_cursor_list_page(
        items,
        page.limit as i32,
        next_page_token,
    ))
}

pub(crate) async fn get_changes_start_page_token(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<StartPageTokenQuery>,
) -> Result<Json<SdkWorkApiResponse<StartPageTokenResponse>>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let space_id = require_query_value(query.space_id, "spaceId")?;
    validate_space_exists(&state.pool, &tenant_id, &space_id).await?;
    acl::ensure_space_change_feed_reader(&state.pool, &ctx, &space_id).await?;
    let start_page_token =
        query_start_page_token(&state.pool, &tenant_id, Some(space_id.as_str())).await?;
    Ok(success_envelope(StartPageTokenResponse {
        start_page_token: encode_change_sequence_cursor(start_page_token)
            .ok_or_else(|| internal_problem("change start page token is invalid"))?,
    }))
}

pub(crate) async fn query_start_page_token(
    pool: &sqlx::PgPool,
    tenant_id: &str,
    space_id: Option<&str>,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    SqlDriveChangeFeedService::new(pool.clone())
        .query_start_page_token(QueryStartPageTokenCommand {
            tenant_id: tenant_id.to_string(),
            space_id: space_id.map(str::to_string),
        })
        .await
        .map_err(map_service_error)
}

fn map_change_record(record: DriveChangeRecord) -> ChangeResponse {
    ChangeResponse {
        sequence_no: record.sequence_no,
        tenant_id: record.tenant_id,
        space_id: record.space_id,
        node_id: record.node_id,
        event_type: record.event_type,
        actor_id: record.actor_id,
        created_at: record.created_at,
    }
}
