use crate::acl;
use crate::acl_sql;
use crate::app_context::DriveRequestContext;
use crate::constants::MAX_FAVORITE_CHECK_NODE_IDS;
use crate::dto::{
    CheckFavoriteNodesRequest, FavoriteNodeCheckItem, FavoriteNodeQuery, FavoriteNodeRequest,
    FavoriteNodeResponse, NodeViewQuery, SubjectNodeViewQuery,
};
use crate::error::{internal_sql_error, problem, ProblemDetail, SdkWorkResultCode};
use crate::mappers::map_node_row;
use crate::metadata_repository::present_node_list;
use crate::node_repository::find_active_node;
use crate::response::{
    current_trace_id, no_content, success_envelope, DriveListHttpResponse,
    DriveNodeListHttpResponse,
};
use crate::route_change::record_change;
use crate::space_repository::validate_space_exists;
use crate::state::AppState;
use crate::time::current_epoch_ms;
use crate::validators::{
    normalize_optional_text, parse_page_request, resolve_aliased_node_list_order_by,
    resolve_node_list_order_by, validate_subject_type,
};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_drive_workspace_service::infrastructure::sql::{
    NODE_API_SELECT_COLUMNS, NODE_API_SELECT_JOIN_COLUMNS,
};
use sdkwork_utils_rust::{PageInfo, PageMode, SdkWorkApiResponse, SdkWorkPageData};
use sqlx::Row;

pub(crate) async fn list_recent_nodes(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<NodeViewQuery>,
) -> Result<DriveNodeListHttpResponse, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_page_request(query.page_size, query.page_token)?;
    let order_by = resolve_node_list_order_by(
        query.sort_by.clone(),
        query.sort_order.clone(),
        "updated_at DESC, id ASC",
    )?;
    let order_by_for_fetch = order_by.clone();
    let space_id = normalize_optional_text(query.space_id);
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    if let Some(space_id) = space_id.as_deref() {
        validate_space_exists(&state.pool, &tenant_id, space_id).await?;
        acl::ensure_list_parent_reader(&state.pool, &ctx, space_id, None).await?;
    }
    let pool = state.pool.clone();
    let tenant_id_for_fetch = tenant_id.clone();
    let subject_type_for_fetch = subject_type.clone();
    let subject_id_for_fetch = subject_id.clone();

    let (items, next_page_token, incomplete_page) = if let Some(space_id) = space_id.clone() {
        let is_space_owner = acl::is_subject_space_owner(
            &state.pool,
            &tenant_id,
            &space_id,
            &subject_type,
            &subject_id,
        )
        .await?;
        let reader_acl_predicate =
            acl_sql::reader_inherited_permission_exists_sql("dr_drive_node", "$3", "$4");
        let (items, next_page_token) = if is_space_owner {
            acl::paginate_offset_limited_items(
                page,
                move |scan_offset, batch_limit| {
                    let pool = pool.clone();
                    let tenant_id = tenant_id_for_fetch.clone();
                    let space_id = space_id.clone();
                    let order_by = order_by_for_fetch.clone();
                    async move {
                        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                            "SELECT {NODE_API_SELECT_COLUMNS}
                             FROM dr_drive_node
                             WHERE tenant_id=$1
                               AND space_id=$2
                               AND lifecycle_status='active'
                               AND content_state='ready'
                             ORDER BY {order_by}
                             LIMIT $3 OFFSET $4",
                        )))
                        .bind(&tenant_id)
                        .bind(&space_id)
                        .bind(batch_limit as i64)
                        .bind(scan_offset)
                        .fetch_all(&pool)
                        .await
                        .map_err(internal_sql_error("list recent dr_drive_node failed"))?;
                        Ok(rows)
                    }
                },
                map_node_row,
            )
            .await?
        } else {
            acl::paginate_offset_limited_items(
                page,
                move |scan_offset, batch_limit| {
                    let pool = pool.clone();
                    let tenant_id = tenant_id_for_fetch.clone();
                    let space_id = space_id.clone();
                    let subject_type = subject_type_for_fetch.clone();
                    let subject_id = subject_id_for_fetch.clone();
                    let reader_acl_predicate = reader_acl_predicate.clone();
                    let order_by = order_by_for_fetch.clone();
                    async move {
                        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                            "SELECT {NODE_API_SELECT_COLUMNS}
                             FROM dr_drive_node
                             WHERE tenant_id=$1
                               AND space_id=$2
                               AND lifecycle_status='active'
                               AND content_state='ready'
                               AND ({reader_acl_predicate})
                             ORDER BY {order_by}
                             LIMIT $5 OFFSET $6",
                        )))
                        .bind(&tenant_id)
                        .bind(&space_id)
                        .bind(&subject_type)
                        .bind(&subject_id)
                        .bind(batch_limit as i64)
                        .bind(scan_offset)
                        .fetch_all(&pool)
                        .await
                        .map_err(internal_sql_error("list recent dr_drive_node failed"))?;
                        Ok(rows)
                    }
                },
                map_node_row,
            )
            .await?
        };
        (items, next_page_token, false)
    } else {
        let reader_acl_predicate = acl_sql::node_reader_visible_sql("dr_drive_node", "$2", "$3");
        let (items, next_page_token) = acl::paginate_offset_limited_items(
            page,
            move |scan_offset, batch_limit| {
                let pool = pool.clone();
                let tenant_id = tenant_id_for_fetch.clone();
                let subject_type = subject_type_for_fetch.clone();
                let subject_id = subject_id_for_fetch.clone();
                let reader_acl_predicate = reader_acl_predicate.clone();
                let order_by = order_by_for_fetch.clone();
                async move {
                    let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                        "SELECT {NODE_API_SELECT_COLUMNS}
                         FROM dr_drive_node
                         WHERE tenant_id=$1
                           AND lifecycle_status='active'
                           AND content_state='ready'
                           AND ({reader_acl_predicate})
                         ORDER BY {order_by}
                         LIMIT $4 OFFSET $5",
                    )))
                    .bind(&tenant_id)
                    .bind(&subject_type)
                    .bind(&subject_id)
                    .bind(batch_limit as i64)
                    .bind(scan_offset)
                    .fetch_all(&pool)
                    .await
                    .map_err(internal_sql_error("list recent dr_drive_node failed"))?;
                    Ok(rows)
                }
            },
            map_node_row,
        )
        .await?;
        (items, next_page_token, false)
    };

    present_node_list(
        &state.pool,
        &tenant_id,
        items,
        page,
        next_page_token,
        incomplete_page,
    )
    .await
}
pub(crate) async fn list_shared_with_me(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<SubjectNodeViewQuery>,
) -> Result<DriveNodeListHttpResponse, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let page = parse_page_request(query.page_size, query.page_token)?;
    let order_by = resolve_aliased_node_list_order_by(
        query.sort_by.clone(),
        query.sort_order.clone(),
        "n",
        "n.updated_at DESC, n.id ASC",
    )?;
    let order_by_for_fetch = order_by.clone();
    let space_id = normalize_optional_text(query.space_id);
    if let Some(space_id) = space_id.as_deref() {
        validate_space_exists(&state.pool, &tenant_id, space_id).await?;
        acl::ensure_subject_space_scoped_reader(
            &state.pool,
            &tenant_id,
            space_id,
            None,
            &subject_type,
            &subject_id,
        )
        .await?;
    }
    let pool = state.pool.clone();
    let tenant_id_for_fetch = tenant_id.clone();
    let subject_type_for_fetch = subject_type.clone();
    let subject_id_for_fetch = subject_id.clone();
    let now_epoch_ms = current_epoch_ms();

    if let Some(space_id) = space_id.clone() {
        if acl::is_subject_space_owner(
            &state.pool,
            &tenant_id,
            &space_id,
            &subject_type,
            &subject_id,
        )
        .await?
        {
            return present_node_list(&state.pool, &tenant_id, Vec::new(), page, None, false).await;
        }
    }

    let shared_with_me_predicate_for_space =
        acl_sql::shared_with_me_visible_sql("n", "$3", "$4", "$5");
    let shared_with_me_predicate_for_tenant =
        acl_sql::shared_with_me_visible_sql("n", "$2", "$3", "$4");
    let (items, next_page_token) = if let Some(space_id) = space_id.clone() {
        acl::paginate_offset_limited_items(
            page,
            move |scan_offset, batch_limit| {
                let pool = pool.clone();
                let tenant_id = tenant_id_for_fetch.clone();
                let space_id = space_id.clone();
                let subject_type = subject_type_for_fetch.clone();
                let subject_id = subject_id_for_fetch.clone();
                let shared_with_me_predicate = shared_with_me_predicate_for_space.clone();
                let order_by = order_by_for_fetch.clone();
                async move {
                    let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                        "SELECT {NODE_API_SELECT_JOIN_COLUMNS}
                         FROM dr_drive_node n
                         WHERE n.tenant_id=$1
                           AND n.space_id=$2
                           AND n.lifecycle_status='active'
                           AND n.content_state='ready'
                           AND {shared_with_me_predicate}
                         ORDER BY {order_by}
                         LIMIT $6 OFFSET $7",
                    )))
                    .bind(&tenant_id)
                    .bind(&space_id)
                    .bind(&subject_type)
                    .bind(&subject_id)
                    .bind(now_epoch_ms)
                    .bind(batch_limit as i64)
                    .bind(scan_offset)
                    .fetch_all(&pool)
                    .await
                    .map_err(internal_sql_error("list shared dr_drive_node failed"))?;
                    Ok(rows)
                }
            },
            map_node_row,
        )
        .await?
    } else {
        acl::paginate_offset_limited_items(
            page,
            move |scan_offset, batch_limit| {
                let pool = pool.clone();
                let tenant_id = tenant_id_for_fetch.clone();
                let subject_type = subject_type_for_fetch.clone();
                let subject_id = subject_id_for_fetch.clone();
                let shared_with_me_predicate = shared_with_me_predicate_for_tenant.clone();
                let order_by = order_by_for_fetch.clone();
                async move {
                    let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                        "SELECT {NODE_API_SELECT_JOIN_COLUMNS}
                         FROM dr_drive_node n
                         WHERE n.tenant_id=$1
                           AND n.lifecycle_status='active'
                           AND n.content_state='ready'
                           AND {shared_with_me_predicate}
                         ORDER BY {order_by}
                         LIMIT $5 OFFSET $6",
                    )))
                    .bind(&tenant_id)
                    .bind(&subject_type)
                    .bind(&subject_id)
                    .bind(now_epoch_ms)
                    .bind(batch_limit as i64)
                    .bind(scan_offset)
                    .fetch_all(&pool)
                    .await
                    .map_err(internal_sql_error("list shared dr_drive_node failed"))?;
                    Ok(rows)
                }
            },
            map_node_row,
        )
        .await?
    };

    present_node_list(&state.pool, &tenant_id, items, page, next_page_token, false).await
}
pub(crate) async fn list_favorite_nodes(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<SubjectNodeViewQuery>,
) -> Result<DriveNodeListHttpResponse, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let page = parse_page_request(query.page_size, query.page_token)?;
    let order_by = resolve_aliased_node_list_order_by(
        query.sort_by.clone(),
        query.sort_order.clone(),
        "n",
        "f.updated_at DESC, n.id ASC",
    )?;
    let order_by_for_fetch = order_by.clone();
    let space_id = normalize_optional_text(query.space_id);
    if let Some(space_id) = space_id.as_deref() {
        validate_space_exists(&state.pool, &tenant_id, space_id).await?;
        acl::ensure_subject_space_scoped_reader(
            &state.pool,
            &tenant_id,
            space_id,
            None,
            &subject_type,
            &subject_id,
        )
        .await?;
    }
    let pool = state.pool.clone();
    let tenant_id_for_fetch = tenant_id.clone();
    let subject_type_for_fetch = subject_type.clone();
    let subject_id_for_fetch = subject_id.clone();

    let (items, next_page_token, incomplete_page) = if let Some(space_id) = space_id.clone() {
        let is_space_owner = acl::is_subject_space_owner(
            &state.pool,
            &tenant_id,
            &space_id,
            &subject_type,
            &subject_id,
        )
        .await?;
        let reader_acl_predicate = acl_sql::reader_inherited_permission_exists_sql("n", "$3", "$4");
        let (items, next_page_token) = if is_space_owner {
            acl::paginate_offset_limited_items(
                page,
                move |scan_offset, batch_limit| {
                    let pool = pool.clone();
                    let tenant_id = tenant_id_for_fetch.clone();
                    let space_id = space_id.clone();
                    let subject_type = subject_type_for_fetch.clone();
                    let subject_id = subject_id_for_fetch.clone();
                    let order_by = order_by_for_fetch.clone();
                    async move {
                        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                            "SELECT {NODE_API_SELECT_JOIN_COLUMNS}
                             FROM dr_drive_node_favorite f
                             INNER JOIN dr_drive_node n
                                ON n.tenant_id=f.tenant_id
                               AND n.id=f.node_id
                               AND n.lifecycle_status='active'
                               AND n.content_state='ready'
                             WHERE f.tenant_id=$1
                               AND n.space_id=$2
                               AND f.subject_type=$3
                               AND f.subject_id=$4
                               AND f.lifecycle_status='active'
                             ORDER BY {order_by}
                             LIMIT $5 OFFSET $6",
                        )))
                        .bind(&tenant_id)
                        .bind(&space_id)
                        .bind(&subject_type)
                        .bind(&subject_id)
                        .bind(batch_limit as i64)
                        .bind(scan_offset)
                        .fetch_all(&pool)
                        .await
                        .map_err(internal_sql_error("list favorite dr_drive_node failed"))?;
                        Ok(rows)
                    }
                },
                map_node_row,
            )
            .await?
        } else {
            acl::paginate_offset_limited_items(
                page,
                move |scan_offset, batch_limit| {
                    let pool = pool.clone();
                    let tenant_id = tenant_id_for_fetch.clone();
                    let space_id = space_id.clone();
                    let subject_type = subject_type_for_fetch.clone();
                    let subject_id = subject_id_for_fetch.clone();
                    let reader_acl_predicate = reader_acl_predicate.clone();
                    let order_by = order_by_for_fetch.clone();
                    async move {
                        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                            "SELECT {NODE_API_SELECT_JOIN_COLUMNS}
                             FROM dr_drive_node_favorite f
                             INNER JOIN dr_drive_node n
                                ON n.tenant_id=f.tenant_id
                               AND n.id=f.node_id
                               AND n.lifecycle_status='active'
                               AND n.content_state='ready'
                             WHERE f.tenant_id=$1
                               AND n.space_id=$2
                               AND f.subject_type=$3
                               AND f.subject_id=$4
                               AND f.lifecycle_status='active'
                               AND ({reader_acl_predicate})
                             ORDER BY {order_by}
                             LIMIT $5 OFFSET $6",
                        )))
                        .bind(&tenant_id)
                        .bind(&space_id)
                        .bind(&subject_type)
                        .bind(&subject_id)
                        .bind(batch_limit as i64)
                        .bind(scan_offset)
                        .fetch_all(&pool)
                        .await
                        .map_err(internal_sql_error("list favorite dr_drive_node failed"))?;
                        Ok(rows)
                    }
                },
                map_node_row,
            )
            .await?
        };
        (items, next_page_token, false)
    } else {
        let reader_acl_predicate = acl_sql::node_reader_visible_sql("n", "$2", "$3");
        let (items, next_page_token) = acl::paginate_offset_limited_items(
            page,
            move |scan_offset, batch_limit| {
                let pool = pool.clone();
                let tenant_id = tenant_id_for_fetch.clone();
                let subject_type = subject_type_for_fetch.clone();
                let subject_id = subject_id_for_fetch.clone();
                let reader_acl_predicate = reader_acl_predicate.clone();
                let order_by = order_by_for_fetch.clone();
                async move {
                    let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                        "SELECT {NODE_API_SELECT_JOIN_COLUMNS}
                         FROM dr_drive_node_favorite f
                         INNER JOIN dr_drive_node n
                            ON n.tenant_id=f.tenant_id
                           AND n.id=f.node_id
                           AND n.lifecycle_status='active'
                           AND n.content_state='ready'
                         WHERE f.tenant_id=$1
                           AND f.subject_type=$2
                           AND f.subject_id=$3
                           AND f.lifecycle_status='active'
                           AND ({reader_acl_predicate})
                         ORDER BY {order_by}
                         LIMIT $4 OFFSET $5",
                    )))
                    .bind(&tenant_id)
                    .bind(&subject_type)
                    .bind(&subject_id)
                    .bind(batch_limit as i64)
                    .bind(scan_offset)
                    .fetch_all(&pool)
                    .await
                    .map_err(internal_sql_error("list favorite dr_drive_node failed"))?;
                    Ok(rows)
                }
            },
            map_node_row,
        )
        .await?;
        (items, next_page_token, false)
    };

    present_node_list(
        &state.pool,
        &tenant_id,
        items,
        page,
        next_page_token,
        incomplete_page,
    )
    .await
}
pub(crate) async fn set_favorite(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Json(_payload): Json<FavoriteNodeRequest>,
) -> Result<Json<SdkWorkApiResponse<FavoriteNodeResponse>>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    validate_subject_type(&subject_type)?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    let favorite_id = build_favorite_id(&tenant_id, &subject_type, &subject_id, &node_id);
    sqlx::query(
        "INSERT INTO dr_drive_node_favorite (
            id, tenant_id, node_id, subject_type, subject_id,
            lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, $5, 'active', 1, $6, $6)
         ON CONFLICT(tenant_id, subject_type, subject_id, node_id)
         DO UPDATE SET lifecycle_status='active',
                       updated_by=excluded.updated_by,
                       updated_at=CURRENT_TIMESTAMP,
                       version=dr_drive_node_favorite.version + 1",
    )
    .bind(favorite_id)
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&subject_type)
    .bind(&subject_id)
    .bind(&operator_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("upsert dr_drive_node_favorite failed"))?;
    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&node_id),
        drive_events::favorite::CREATED,
        &operator_id,
    )
    .await?;
    Ok(success_envelope(FavoriteNodeResponse { favorited: true }))
}
pub(crate) async fn unset_favorite(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Query(_query): Query<FavoriteNodeQuery>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    validate_subject_type(&subject_type)?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    let affected = sqlx::query(
        "UPDATE dr_drive_node_favorite
         SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
         WHERE tenant_id=$2
           AND node_id=$3
           AND subject_type=$4
           AND subject_id=$5
           AND lifecycle_status='active'",
    )
    .bind(&operator_id)
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&subject_type)
    .bind(&subject_id)
    .execute(&state.pool)
    .await
    .map_err(internal_sql_error("delete dr_drive_node_favorite failed"))?
    .rows_affected();
    if affected > 0 {
        record_change(
            &state.pool,
            &tenant_id,
            &node.space_id,
            Some(&node_id),
            drive_events::favorite::DELETED,
            &operator_id,
        )
        .await?;
    }
    Ok(no_content())
}

pub(crate) async fn check_favorite_nodes(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Json(payload): Json<CheckFavoriteNodesRequest>,
) -> Result<DriveListHttpResponse<FavoriteNodeCheckItem>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    validate_subject_type(&subject_type)?;

    let mut node_ids = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for raw in &payload.node_ids {
        let trimmed = raw.trim();
        if trimmed.is_empty() || !seen.insert(trimmed.to_string()) {
            continue;
        }
        node_ids.push(trimmed.to_string());
    }
    if node_ids.len() > MAX_FAVORITE_CHECK_NODE_IDS {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("nodeIds must contain at most {MAX_FAVORITE_CHECK_NODE_IDS} unique entries"),
            SdkWorkResultCode::ValidationError,
        ));
    }

    let favorited_ids = if node_ids.is_empty() {
        std::collections::BTreeSet::new()
    } else {
        let placeholders = (0..node_ids.len())
            .map(|index| format!("${}", index + 4))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT node_id
             FROM dr_drive_node_favorite
             WHERE tenant_id=$1
               AND subject_type=$2
               AND subject_id=$3
               AND lifecycle_status='active'
               AND node_id IN ({placeholders})"
        );
        let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .bind(&tenant_id)
            .bind(&subject_type)
            .bind(&subject_id);
        for node_id in &node_ids {
            query = query.bind(node_id);
        }
        let rows = query
            .fetch_all(&state.pool)
            .await
            .map_err(internal_sql_error("check dr_drive_node_favorite failed"))?;
        rows.iter()
            .map(|row| row.get::<String, _>("node_id"))
            .collect::<std::collections::BTreeSet<_>>()
    };

    let items = node_ids
        .into_iter()
        .map(|node_id| FavoriteNodeCheckItem {
            favorited: favorited_ids.contains(&node_id),
            node_id,
        })
        .collect::<Vec<_>>();
    let item_count = items.len() as i32;
    Ok(Json(SdkWorkApiResponse::success(
        SdkWorkPageData {
            items,
            page_info: PageInfo {
                mode: PageMode::Offset,
                page: Some(1),
                page_size: Some(item_count),
                total_items: Some(item_count.to_string()),
                total_pages: Some(1),
                next_cursor: None,
                has_more: Some(false),
            },
        },
        current_trace_id(),
    )))
}

fn build_favorite_id(
    tenant_id: &str,
    subject_type: &str,
    subject_id: &str,
    node_id: &str,
) -> String {
    format!(
        "fav:{}:{}:{}:{}",
        tenant_id, subject_type, subject_id, node_id
    )
}
