use crate::acl;
use crate::acl_sql;
use crate::app_context::DriveRequestContext;
use crate::dto::{
    CreateWatchChannelRequest, DriveWatchChannelResponse, InsertWatchChannel,
    StopWatchChannelRequest, StopWatchChannelResponse, WatchChannelListQuery,
};
use crate::error::{internal_sql_error, ProblemDetail};
use crate::hashing::sha256_raw_hex;
use crate::mappers::map_watch_channel_row;
use crate::node_repository::find_active_node;
use crate::response::{
    success_created_resource, success_envelope, success_list_page_simple, success_resource,
    DriveListHttpResponse,
};
use crate::route_change::record_change;
use crate::space_repository::validate_space_exists;
use crate::state::AppState;
use crate::validators::{
    normalize_optional_text, parse_page_request, require_body_value,
    validate_watch_channel_address, validate_watch_channel_id, validate_watch_channel_type,
    validate_watch_expiration, validate_watch_lifecycle_status, validate_watch_resource_type,
};
use crate::watch_repository::{find_watch_channel, insert_watch_channel};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};

fn hash_watch_token(token: &str) -> String {
    sha256_raw_hex(token.trim().as_bytes())
}

pub(crate) async fn watch_changes(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Json(payload): Json<CreateWatchChannelRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<DriveWatchChannelResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let space_id = require_body_value(payload.space_id, "spaceId")?;
    validate_space_exists(&state.pool, &tenant_id, &space_id).await?;
    acl::ensure_list_parent_reader(&state.pool, &ctx, &space_id, None).await?;
    let operator_id = ctx.resolve_operator_id()?;
    let channel_id = validate_watch_channel_id(&payload.id)?.to_string();
    let address = validate_watch_channel_address(&payload.address)?.to_string();
    let channel_type = validate_watch_channel_type(
        normalize_optional_text(payload.channel_type)
            .as_deref()
            .unwrap_or("web_hook"),
    )?
    .to_string();
    validate_watch_expiration(payload.expiration_epoch_ms)?;
    insert_watch_channel(
        &state.pool,
        InsertWatchChannel {
            id: &channel_id,
            tenant_id: &tenant_id,
            space_id: Some(&space_id),
            node_id: None,
            resource_type: "changes",
            resource_id: Some(&space_id),
            channel_type: &channel_type,
            address: &address,
            token_hash: payload.token.as_deref().map(hash_watch_token),
            expiration_epoch_ms: payload.expiration_epoch_ms,
            operator_id: &operator_id,
        },
    )
    .await?;
    record_change(
        &state.pool,
        &tenant_id,
        &space_id,
        None,
        drive_events::watch_channel::CREATED,
        &operator_id,
    )
    .await?;
    Ok(success_created_resource(
        find_watch_channel(&state.pool, &tenant_id, &channel_id).await?,
    ))
}

pub(crate) async fn watch_node(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Json(payload): Json<CreateWatchChannelRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<DriveWatchChannelResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    let operator_id = ctx.resolve_operator_id()?;
    let channel_id = validate_watch_channel_id(&payload.id)?.to_string();
    let address = validate_watch_channel_address(&payload.address)?.to_string();
    let channel_type = validate_watch_channel_type(
        normalize_optional_text(payload.channel_type)
            .as_deref()
            .unwrap_or("web_hook"),
    )?
    .to_string();
    validate_watch_expiration(payload.expiration_epoch_ms)?;
    insert_watch_channel(
        &state.pool,
        InsertWatchChannel {
            id: &channel_id,
            tenant_id: &tenant_id,
            space_id: Some(&node.space_id),
            node_id: Some(&node_id),
            resource_type: "node",
            resource_id: Some(&node_id),
            channel_type: &channel_type,
            address: &address,
            token_hash: payload.token.as_deref().map(hash_watch_token),
            expiration_epoch_ms: payload.expiration_epoch_ms,
            operator_id: &operator_id,
        },
    )
    .await?;
    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&node_id),
        drive_events::watch_channel::CREATED,
        &operator_id,
    )
    .await?;
    Ok(success_created_resource(
        find_watch_channel(&state.pool, &tenant_id, &channel_id).await?,
    ))
}

pub(crate) async fn list_watch_channels(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<WatchChannelListQuery>,
) -> Result<DriveListHttpResponse<DriveWatchChannelResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_page_request(query.page_size, query.page_token)?;
    let lifecycle_status = validate_watch_lifecycle_status(
        normalize_optional_text(query.lifecycle_status)
            .as_deref()
            .unwrap_or("active"),
    )?
    .to_string();
    let resource_type = match normalize_optional_text(query.resource_type) {
        Some(resource_type) => Some(validate_watch_resource_type(&resource_type)?.to_string()),
        None => None,
    };
    let (subject_type, subject_id) = ctx.resolve_subject()?;

    let pool = state.pool.clone();
    let tenant_id_for_fetch = tenant_id.clone();
    let lifecycle_status_for_fetch = lifecycle_status.clone();
    let resource_type_for_fetch = resource_type.clone();
    let subject_type_for_fetch = subject_type.clone();
    let subject_id_for_fetch = subject_id.clone();
    let reader_visible_predicate =
        acl_sql::watch_channel_reader_visible_sql("dr_drive_watch_channel", "$2", "$3");

    let (items, next_page_token) = acl::paginate_offset_limited_items(
        page,
        move |scan_offset, batch_limit| {
            let pool = pool.clone();
            let tenant_id = tenant_id_for_fetch.clone();
            let lifecycle_status = lifecycle_status_for_fetch.clone();
            let resource_type = resource_type_for_fetch.clone();
            let subject_type = subject_type_for_fetch.clone();
            let subject_id = subject_id_for_fetch.clone();
            let reader_visible_predicate = reader_visible_predicate.clone();
            async move {
                let rows = if let Some(resource_type) = resource_type.as_deref() {
                    sqlx::query(sqlx::AssertSqlSafe(format!(
                        "SELECT id, tenant_id, space_id, node_id, resource_type, resource_id,
                                channel_type, address, expiration_epoch_ms, lifecycle_status, version
                         FROM dr_drive_watch_channel
                         WHERE tenant_id=$1
                           AND lifecycle_status=$4
                           AND resource_type=$5
                           AND ({reader_visible_predicate})
                         ORDER BY created_at ASC, id ASC
                         LIMIT $6 OFFSET $7",
                    )))
                    .bind(&tenant_id)
                    .bind(&subject_type)
                    .bind(&subject_id)
                    .bind(&lifecycle_status)
                    .bind(resource_type)
                    .bind(batch_limit as i64)
                    .bind(scan_offset)
                    .fetch_all(&pool)
                    .await
                } else {
                    sqlx::query(sqlx::AssertSqlSafe(format!(
                        "SELECT id, tenant_id, space_id, node_id, resource_type, resource_id,
                                channel_type, address, expiration_epoch_ms, lifecycle_status, version
                         FROM dr_drive_watch_channel
                         WHERE tenant_id=$1
                           AND lifecycle_status=$4
                           AND ({reader_visible_predicate})
                         ORDER BY created_at ASC, id ASC
                         LIMIT $5 OFFSET $6",
                    )))
                    .bind(&tenant_id)
                    .bind(&subject_type)
                    .bind(&subject_id)
                    .bind(&lifecycle_status)
                    .bind(batch_limit as i64)
                    .bind(scan_offset)
                    .fetch_all(&pool)
                    .await
                }
                .map_err(internal_sql_error("list dr_drive_watch_channel failed"))?;
                Ok(rows)
            }
        },
        map_watch_channel_row,
    )
    .await?;

    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn get_watch_channel(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(channel_id): Path<String>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<DriveWatchChannelResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let channel = find_watch_channel(&state.pool, &tenant_id, &channel_id).await?;
    acl::ensure_watch_channel_role(&state.pool, &ctx, &channel, "reader").await?;
    Ok(success_resource(channel))
}

pub(crate) async fn stop_watch_channel(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(channel_id): Path<String>,
    Json(_payload): Json<StopWatchChannelRequest>,
) -> Result<Json<SdkWorkApiResponse<StopWatchChannelResponse>>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let current = find_watch_channel(&state.pool, &tenant_id, &channel_id).await?;
    acl::ensure_watch_channel_role(&state.pool, &ctx, &current, "writer").await?;
    let affected = sqlx::query(
        "UPDATE dr_drive_watch_channel
         SET lifecycle_status='stopped',
             updated_by=$1,
             updated_at=CURRENT_TIMESTAMP,
             version=version + 1
         WHERE tenant_id=$2
           AND id=$3
           AND lifecycle_status='active'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&channel_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("stop dr_drive_watch_channel failed"))?
    .rows_affected();
    if affected > 0 {
        let space_id = current.space_id.as_deref().unwrap_or(&channel_id);
        record_change(
            &state.pool,
            &tenant_id,
            space_id,
            current.node_id.as_deref(),
            drive_events::watch_channel::STOPPED,
            &operator_id,
        )
        .await?;
    }
    Ok(success_envelope(StopWatchChannelResponse {
        stopped: affected > 0,
        channel: find_watch_channel(&state.pool, &tenant_id, &channel_id).await?,
    }))
}
