use crate::acl;
use crate::acl_sql;
use crate::app_context::DriveRequestContext;
use crate::dto::SearchQuery;
use crate::error::{internal_sql_error, ProblemDetail};
use crate::mappers::map_node_row;
use crate::metadata_repository::present_node_list;
use crate::response::DriveNodeListHttpResponse;
use crate::space_repository::validate_space_exists;
use crate::state::AppState;
use crate::validators::{normalize_optional_text, parse_page_request};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_workspace_service::infrastructure::sql::NODE_API_SELECT_COLUMNS;

pub(crate) async fn search_nodes(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<SearchQuery>,
) -> Result<DriveNodeListHttpResponse, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_page_request(query.page_size, query.page_token)?;
    let needle = format!("%{}%", query.q.unwrap_or_default().trim());
    let (subject_type, subject_id) = ctx.resolve_subject()?;
    let space_id = normalize_optional_text(query.space_id);
    if let Some(space_id) = space_id.as_deref() {
        validate_space_exists(&state.pool, &tenant_id, space_id).await?;
        acl::ensure_list_parent_reader(&state.pool, &ctx, space_id, None).await?;
    }

    let pool = state.pool.clone();
    let tenant_id_for_fetch = tenant_id.clone();
    let space_id_for_fetch = space_id.clone();
    let needle_for_fetch = needle.clone();
    let subject_type_for_fetch = subject_type.clone();
    let subject_id_for_fetch = subject_id.clone();

    let (items, next_page_token, incomplete_page) = if let Some(space_id) =
        space_id_for_fetch.clone()
    {
        let is_space_owner = acl::is_subject_space_owner(
            &state.pool,
            &tenant_id,
            &space_id,
            &subject_type,
            &subject_id,
        )
        .await?;
        let reader_acl_predicate =
            acl_sql::reader_inherited_permission_exists_sql("dr_drive_node", "$4", "$5");
        let (items, next_page_token) = if is_space_owner {
            acl::paginate_offset_limited_items(
                page,
                move |scan_offset, batch_limit| {
                    let pool = pool.clone();
                    let tenant_id = tenant_id_for_fetch.clone();
                    let space_id = space_id.clone();
                    let needle = needle_for_fetch.clone();
                    async move {
                        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                            "SELECT {NODE_API_SELECT_COLUMNS}
                             FROM dr_drive_node
                             WHERE tenant_id=$1
                               AND space_id=$2
                               AND node_name LIKE $3
                               AND lifecycle_status='active'
                               AND content_state='ready'
                             ORDER BY updated_at DESC, id ASC
                             LIMIT $4 OFFSET $5",
                        )))
                        .bind(&tenant_id)
                        .bind(&space_id)
                        .bind(&needle)
                        .bind(batch_limit as i64)
                        .bind(scan_offset)
                        .fetch_all(&pool)
                        .await
                        .map_err(internal_sql_error("search dr_drive_node failed"))?;
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
                    let needle = needle_for_fetch.clone();
                    let subject_type = subject_type_for_fetch.clone();
                    let subject_id = subject_id_for_fetch.clone();
                    let reader_acl_predicate = reader_acl_predicate.clone();
                    async move {
                        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                            "SELECT {NODE_API_SELECT_COLUMNS}
                             FROM dr_drive_node
                             WHERE tenant_id=$1
                               AND space_id=$2
                               AND node_name LIKE $3
                               AND lifecycle_status='active'
                               AND content_state='ready'
                               AND ({reader_acl_predicate})
                             ORDER BY updated_at DESC, id ASC
                             LIMIT $6 OFFSET $7",
                        )))
                        .bind(&tenant_id)
                        .bind(&space_id)
                        .bind(&needle)
                        .bind(&subject_type)
                        .bind(&subject_id)
                        .bind(batch_limit as i64)
                        .bind(scan_offset)
                        .fetch_all(&pool)
                        .await
                        .map_err(internal_sql_error("search dr_drive_node failed"))?;
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
                let needle = needle_for_fetch.clone();
                let subject_type = subject_type_for_fetch.clone();
                let subject_id = subject_id_for_fetch.clone();
                let reader_acl_predicate = reader_acl_predicate.clone();
                async move {
                    let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                        "SELECT {NODE_API_SELECT_COLUMNS}
                         FROM dr_drive_node
                         WHERE tenant_id=$1
                           AND node_name LIKE $4
                           AND lifecycle_status='active'
                           AND content_state='ready'
                           AND ({reader_acl_predicate})
                         ORDER BY updated_at DESC, id ASC
                         LIMIT $5 OFFSET $6",
                    )))
                    .bind(&tenant_id)
                    .bind(&subject_type)
                    .bind(&subject_id)
                    .bind(&needle)
                    .bind(batch_limit as i64)
                    .bind(scan_offset)
                    .fetch_all(&pool)
                    .await
                    .map_err(internal_sql_error("search dr_drive_node failed"))?;
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
