use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::collaboration_repository::{find_comment, find_comment_reply};
use crate::dto::{
    CommentReplyResponse, CommentResponse, CreateCommentReplyRequest, CreateCommentRequest,
    NodeMutationQuery, PageQuery, UpdateCommentReplyRequest, UpdateCommentRequest,
};
use crate::error::{internal_sql_error, not_found_problem, ProblemDetail};
use crate::mappers::{map_comment_reply_row, map_comment_row};
use crate::node_repository::{find_active_node, find_node};
use crate::response::{
    no_content, success_created_resource, success_list_page_simple, success_resource,
    DriveListHttpResponse,
};
use crate::route_change::record_change;
use crate::state::AppState;
use crate::validators::{
    next_page_token, normalize_optional_text, parse_page_request, require_non_empty_text,
};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};

pub(crate) async fn list_comments(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<DriveListHttpResponse<CommentResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_page_request(query.page_size, query.page_token)?;
    let node = find_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    let rows = sqlx::query(
        "SELECT id, tenant_id, node_id, content, anchor, resolved, lifecycle_status,
                version, created_by, updated_by, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at
         FROM dr_drive_node_comment
         WHERE tenant_id=$1 AND node_id=$2 AND lifecycle_status='active'
         ORDER BY created_at DESC, id DESC
         LIMIT $3 OFFSET $4",
    )
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(page.limit + 1)
    .bind(page.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_sql_error("list dr_drive_node_comment failed"))?;
    let mut items = rows
        .iter()
        .map(map_comment_row)
        .map(CommentResponse::from)
        .collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn get_comment(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, comment_id)): Path<(String, String)>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<CommentResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let node = find_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    let comment = find_comment(&state.pool, &tenant_id, &node_id, &comment_id).await?;
    Ok(success_resource(CommentResponse::from(comment)))
}

pub(crate) async fn create_comment(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<CommentResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;

    let comment_id = require_non_empty_text(payload.id, "id")?;
    let content = require_non_empty_text(payload.content, "content")?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "commenter").await?;
    let anchor = normalize_optional_text(payload.anchor);

    sqlx::query(
        "INSERT INTO dr_drive_node_comment (
            id, tenant_id, node_id, content, anchor, resolved, lifecycle_status,
            version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $7)",
    )
    .bind(&comment_id)
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&content)
    .bind(anchor.as_deref())
    .bind(false)
    .bind(&operator_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("insert dr_drive_node_comment failed"))?;

    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&node_id),
        drive_events::comment::CREATED,
        &operator_id,
    )
    .await?;

    let comment = find_comment(&state.pool, &tenant_id, &node_id, &comment_id).await?;
    Ok(success_created_resource(CommentResponse::from(comment)))
}

pub(crate) async fn update_comment(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, comment_id)): Path<(String, String)>,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<CommentResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "writer").await?;
    let current = find_comment(&state.pool, &tenant_id, &node_id, &comment_id).await?;
    let content = match payload.content {
        Some(value) => require_non_empty_text(value, "content")?,
        None => current.content,
    };
    let anchor = payload
        .anchor
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or(current.anchor);
    let resolved = payload.resolved.unwrap_or(current.resolved);

    let affected = sqlx::query(
        "UPDATE dr_drive_node_comment
         SET content=$1, anchor=$2, resolved=$3, updated_by=$4,
             updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$5 AND node_id=$6 AND id=$7 AND lifecycle_status='active'",
    )
    .bind(&content)
    .bind(anchor.as_deref())
    .bind(if resolved { 1_i64 } else { 0_i64 })
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&comment_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("update dr_drive_node_comment failed"))?
    .rows_affected();
    if affected == 0 {
        return Err(not_found_problem("comment not found"));
    }
    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&node_id),
        drive_events::comment::UPDATED,
        &operator_id,
    )
    .await?;

    let comment = find_comment(&state.pool, &tenant_id, &node_id, &comment_id).await?;
    Ok(success_resource(CommentResponse::from(comment)))
}

pub(crate) async fn delete_comment(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, comment_id)): Path<(String, String)>,
    Query(_query): Query<NodeMutationQuery>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "writer").await?;
    let affected = sqlx::query(
        "UPDATE dr_drive_node_comment
         SET lifecycle_status='deleted', updated_by=$1,
             updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$2 AND node_id=$3 AND id=$4 AND lifecycle_status='active'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&comment_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("delete dr_drive_node_comment failed"))?
    .rows_affected();
    if affected > 0 {
        sqlx::query(
            "UPDATE dr_drive_node_comment_reply
             SET lifecycle_status='deleted', updated_by=$1,
                 updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$2 AND node_id=$3 AND comment_id=$4 AND lifecycle_status='active'",
        )
        .bind(&operator_id)
        .bind(&tenant_id)
        .bind(&node_id)
        .bind(&comment_id)
        .execute(&state.pool)
        .await
        .map_err(internal_sql_error(
            "delete dr_drive_node_comment_reply cascade failed",
        ))?;
        record_change(
            &state.pool,
            &tenant_id,
            &node.space_id,
            Some(&node_id),
            drive_events::comment::DELETED,
            &operator_id,
        )
        .await?;
    }
    let _deleted = affected > 0;
    Ok(no_content())
}

pub(crate) async fn list_comment_replies(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, comment_id)): Path<(String, String)>,
    Query(query): Query<PageQuery>,
) -> Result<DriveListHttpResponse<CommentReplyResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_page_request(query.page_size, query.page_token)?;
    let node = find_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    find_comment(&state.pool, &tenant_id, &node_id, &comment_id).await?;
    let rows = sqlx::query(
        "SELECT id, tenant_id, node_id, comment_id, content, lifecycle_status,
                version, created_by, updated_by, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at
         FROM dr_drive_node_comment_reply
         WHERE tenant_id=$1 AND node_id=$2 AND comment_id=$3 AND lifecycle_status='active'
         ORDER BY created_at ASC, id ASC
         LIMIT $4 OFFSET $5",
    )
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&comment_id)
    .bind(page.limit + 1)
    .bind(page.offset)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_sql_error(
        "list dr_drive_node_comment_reply failed",
    ))?;
    let mut items = rows
        .iter()
        .map(map_comment_reply_row)
        .map(CommentReplyResponse::from)
        .collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn get_comment_reply(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, comment_id, reply_id)): Path<(String, String, String)>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<CommentReplyResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let node = find_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    find_comment(&state.pool, &tenant_id, &node_id, &comment_id).await?;
    let reply =
        find_comment_reply(&state.pool, &tenant_id, &node_id, &comment_id, &reply_id).await?;
    Ok(success_resource(CommentReplyResponse::from(reply)))
}

pub(crate) async fn create_comment_reply(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, comment_id)): Path<(String, String)>,
    Json(payload): Json<CreateCommentReplyRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<SdkWorkResourceData<CommentReplyResponse>>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;

    let reply_id = require_non_empty_text(payload.id, "id")?;
    let content = require_non_empty_text(payload.content, "content")?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "commenter").await?;
    find_comment(&state.pool, &tenant_id, &node_id, &comment_id).await?;

    sqlx::query(
        "INSERT INTO dr_drive_node_comment_reply (
            id, tenant_id, node_id, comment_id, content, lifecycle_status,
            version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, 'active', 1, $6, $6)",
    )
    .bind(&reply_id)
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&comment_id)
    .bind(&content)
    .bind(&operator_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error(
        "insert dr_drive_node_comment_reply failed",
    ))?;
    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&node_id),
        drive_events::comment_reply::CREATED,
        &operator_id,
    )
    .await?;
    let reply =
        find_comment_reply(&state.pool, &tenant_id, &node_id, &comment_id, &reply_id).await?;
    Ok(success_created_resource(CommentReplyResponse::from(reply)))
}

pub(crate) async fn update_comment_reply(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, comment_id, reply_id)): Path<(String, String, String)>,
    Json(payload): Json<UpdateCommentReplyRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<CommentReplyResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "writer").await?;
    find_comment(&state.pool, &tenant_id, &node_id, &comment_id).await?;
    let current =
        find_comment_reply(&state.pool, &tenant_id, &node_id, &comment_id, &reply_id).await?;
    let content = match payload.content {
        Some(value) => require_non_empty_text(value, "content")?,
        None => current.content,
    };

    let affected = sqlx::query(
        "UPDATE dr_drive_node_comment_reply
         SET content=$1, updated_by=$2, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$3 AND node_id=$4 AND comment_id=$5 AND id=$6 AND lifecycle_status='active'",
    )
    .bind(&content)
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&comment_id)
    .bind(&reply_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("update dr_drive_node_comment_reply failed"))?
    .rows_affected();
    if affected == 0 {
        return Err(not_found_problem("comment reply not found"));
    }
    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&node_id),
        drive_events::comment_reply::UPDATED,
        &operator_id,
    )
    .await?;
    let reply =
        find_comment_reply(&state.pool, &tenant_id, &node_id, &comment_id, &reply_id).await?;
    Ok(success_resource(CommentReplyResponse::from(reply)))
}

pub(crate) async fn delete_comment_reply(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, comment_id, reply_id)): Path<(String, String, String)>,
    Query(_query): Query<NodeMutationQuery>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "writer").await?;
    find_comment(&state.pool, &tenant_id, &node_id, &comment_id).await?;
    let affected = sqlx::query(
        "UPDATE dr_drive_node_comment_reply
         SET lifecycle_status='deleted', updated_by=$1,
             updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$2 AND node_id=$3 AND comment_id=$4 AND id=$5 AND lifecycle_status='active'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&comment_id)
    .bind(&reply_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("delete dr_drive_node_comment_reply failed"))?
    .rows_affected();
    if affected > 0 {
        record_change(
            &state.pool,
            &tenant_id,
            &node.space_id,
            Some(&node_id),
            drive_events::comment_reply::DELETED,
            &operator_id,
        )
        .await?;
    }
    let _deleted = affected > 0;
    Ok(no_content())
}
