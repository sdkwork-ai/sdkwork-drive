use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::dto::{
    DriveNodeResponse, FileVersionResponse, NodeCommandRequest, NodeMutationQuery, PageQuery,
};
use crate::error::{
    internal_sql_error, not_found_problem, problem, ProblemDetail, SdkWorkResultCode,
};
use crate::mappers::map_file_version_row;
use crate::node_repository::{find_active_node, find_node};
use crate::response::success_resource;
use crate::response::{no_content, success_list_page_simple, DriveListHttpResponse};
use crate::route_change::record_change;
use crate::state::AppState;
use crate::validators::{next_page_token, parse_page_request};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_contract::drive::domain_events as drive_events;
use sdkwork_drive_workspace_service::infrastructure::sql::begin_transaction_sql;
use sdkwork_drive_workspace_service::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};
use sqlx::Row;

pub(crate) async fn list_versions(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(node_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<DriveListHttpResponse<FileVersionResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let page = parse_page_request(query.page_size, query.page_token)?;
    let node = find_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    let logical_version_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node_version
         WHERE tenant_id=$1 AND node_id=$2",
    )
    .bind(&tenant_id)
    .bind(&node_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_sql_error(
        "count dr_drive_node_version rows failed",
    ))?;
    let rows = if logical_version_count > 0 {
        sqlx::query(
            "SELECT id, tenant_id, node_id, version_no, storage_object_id, content_type, content_length,
                    checksum_sha256_hex, lifecycle_status, CAST(created_at AS TEXT) AS created_at
             FROM dr_drive_node_version
             WHERE tenant_id=$1 AND node_id=$2
             ORDER BY version_no DESC
             LIMIT $3 OFFSET $4",
        )
        .bind(&tenant_id)
        .bind(&node_id)
        .bind(page.limit + 1)
        .bind(page.offset)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_sql_error(
            "list dr_drive_node_version rows failed",
        ))?
    } else {
        sqlx::query(
            "SELECT id, tenant_id, node_id, version_no, NULL AS storage_object_id, content_type, content_length,
                    checksum_sha256_hex, lifecycle_status, CAST(created_at AS TEXT) AS created_at
             FROM dr_drive_storage_object
             WHERE tenant_id=$1 AND node_id=$2
             ORDER BY version_no DESC
             LIMIT $3 OFFSET $4",
        )
        .bind(&tenant_id)
        .bind(&node_id)
        .bind(page.limit + 1)
        .bind(page.offset)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_sql_error(
            "list dr_drive_storage_object versions failed",
        ))?
    };
    let mut items = rows.iter().map(map_file_version_row).collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);

    Ok(success_list_page_simple(items, page, next_page_token))
}
pub(crate) async fn get_version(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, version_id)): Path<(String, String)>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<FileVersionResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let node = find_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "reader").await?;
    let logical_row = sqlx::query(
        "SELECT id, tenant_id, node_id, version_no, storage_object_id, content_type, content_length,
                checksum_sha256_hex, lifecycle_status, CAST(created_at AS TEXT) AS created_at
         FROM dr_drive_node_version
         WHERE tenant_id=$1 AND node_id=$2 AND id=$3",
    )
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&version_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_sql_error("read dr_drive_node_version failed"))?;
    if let Some(row) = logical_row {
        return Ok(success_resource(map_file_version_row(&row)));
    }
    let row = sqlx::query(
        "SELECT id, tenant_id, node_id, version_no, NULL AS storage_object_id, content_type, content_length,
                checksum_sha256_hex, lifecycle_status, CAST(created_at AS TEXT) AS created_at
         FROM dr_drive_storage_object
         WHERE tenant_id=$1 AND node_id=$2 AND id=$3",
    )
    .bind(&tenant_id)
    .bind(&node_id)
    .bind(&version_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_sql_error(
        "read dr_drive_storage_object version failed",
    ))?;
    let Some(row) = row else {
        return Err(not_found_problem("version not found"));
    };
    Ok(success_resource(map_file_version_row(&row)))
}
pub(crate) async fn restore_version(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, version_id)): Path<(String, String)>,
    Json(_payload): Json<NodeCommandRequest>,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<DriveNodeResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "writer").await?;
    let mut connection = state.pool.acquire().await.map_err(|error| {
        crate::error::internal_problem(format!(
            "acquire version restore transaction connection failed: {error}"
        ))
    })?;
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .map_err(internal_sql_error(
            "begin version restore transaction failed",
        ))?;
    let restore_result: Result<(), (StatusCode, Json<ProblemDetail>)> = async {
        ensure_managed_website_node_mutation_allowed(&mut connection, &tenant_id, &node_id)
            .await
            .map_err(crate::error::map_service_error)?;
        let logical_row = sqlx::query(
            "SELECT storage_object_id
             FROM dr_drive_node_version
             WHERE tenant_id=$1 AND node_id=$2 AND id=$3",
        )
        .bind(&tenant_id)
        .bind(&node_id)
        .bind(&version_id)
        .fetch_optional(&mut *connection)
        .await
        .map_err(internal_sql_error(
            "find dr_drive_node_version restore target failed",
        ))?;
        if let Some(row) = logical_row {
            let storage_object_id: Option<String> = row.get("storage_object_id");
            let affected = sqlx::query(
                "UPDATE dr_drive_node_version
                 SET lifecycle_status='active', updated_by=$1, updated_at=CURRENT_TIMESTAMP
                 WHERE tenant_id=$2 AND node_id=$3 AND id=$4",
            )
            .bind(&operator_id)
            .bind(&tenant_id)
            .bind(&node_id)
            .bind(&version_id)
            .execute(&mut *connection)
            .await
            .map_err(internal_sql_error("restore dr_drive_node_version failed"))?
            .rows_affected();
            if affected == 0 {
                return Err(not_found_problem("version not found"));
            }
            if let Some(storage_object_id) = storage_object_id {
                sqlx::query(
                    "UPDATE dr_drive_storage_object
                     SET lifecycle_status='active', updated_by=$1, updated_at=CURRENT_TIMESTAMP
                     WHERE tenant_id=$2 AND node_id=$3 AND id=$4",
                )
                .bind(&operator_id)
                .bind(&tenant_id)
                .bind(&node_id)
                .bind(&storage_object_id)
                .execute(&mut *connection)
                .await
                .map_err(internal_sql_error(
                    "restore dr_drive_storage_object from node version failed",
                ))?;
            }
        } else {
            let affected = sqlx::query(
                "UPDATE dr_drive_storage_object
                 SET lifecycle_status='active', updated_by=$1, updated_at=CURRENT_TIMESTAMP
                 WHERE tenant_id=$2 AND node_id=$3 AND id=$4",
            )
            .bind(&operator_id)
            .bind(&tenant_id)
            .bind(&node_id)
            .bind(&version_id)
            .execute(&mut *connection)
            .await
            .map_err(internal_sql_error(
                "restore dr_drive_storage_object version failed",
            ))?
            .rows_affected();
            if affected == 0 {
                return Err(not_found_problem("version not found"));
            }
        }
        Ok(())
    }
    .await;
    match restore_result {
        Ok(()) => sqlx::query("COMMIT")
            .execute(&mut *connection)
            .await
            .map_err(internal_sql_error(
                "commit version restore transaction failed",
            ))?,
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            return Err(error);
        }
    };
    drop(connection);
    record_change(
        &state.pool,
        &tenant_id,
        &node.space_id,
        Some(&node_id),
        drive_events::file_version::RESTORED,
        &operator_id,
    )
    .await?;
    Ok(success_resource(
        crate::metadata_repository::present_drive_node(
            &state.pool,
            &tenant_id,
            find_node(&state.pool, &tenant_id, &node_id).await?,
        )
        .await?,
    ))
}
pub(crate) async fn delete_version(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((node_id, version_id)): Path<(String, String)>,
    Query(_query): Query<NodeMutationQuery>,
) -> Result<StatusCode, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_active_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, &ctx, &node.space_id, &node_id, "writer").await?;
    let mut connection = state.pool.acquire().await.map_err(|error| {
        crate::error::internal_problem(format!(
            "acquire version delete transaction connection failed: {error}"
        ))
    })?;
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .map_err(internal_sql_error(
            "begin version delete transaction failed",
        ))?;
    let delete_result: Result<u64, (StatusCode, Json<ProblemDetail>)> = async {
        ensure_managed_website_node_mutation_allowed(&mut connection, &tenant_id, &node_id)
            .await
            .map_err(crate::error::map_service_error)?;
        let logical_row = sqlx::query(
            "SELECT lifecycle_status, storage_object_id
             FROM dr_drive_node_version
             WHERE tenant_id=$1 AND node_id=$2 AND id=$3",
        )
        .bind(&tenant_id)
        .bind(&node_id)
        .bind(&version_id)
        .fetch_optional(&mut *connection)
        .await
        .map_err(internal_sql_error("find dr_drive_node_version failed"))?;
        if let Some(row) = logical_row {
            let current_status: String = row.get("lifecycle_status");
            let storage_object_id: Option<String> = row.get("storage_object_id");
            if current_status == "active" {
                let active_version_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(1)
                     FROM dr_drive_node_version
                     WHERE tenant_id=$1 AND node_id=$2 AND lifecycle_status='active'",
                )
                .bind(&tenant_id)
                .bind(&node_id)
                .fetch_one(&mut *connection)
                .await
                .map_err(internal_sql_error(
                    "count active dr_drive_node_version rows failed",
                ))?;
                if active_version_count <= 1 {
                    return Err(problem(
                        StatusCode::CONFLICT,
                        "conflict",
                        "cannot delete the only active version",
                        SdkWorkResultCode::Conflict,
                    ));
                }
            }
            let affected = sqlx::query(
                "UPDATE dr_drive_node_version
                 SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP
                 WHERE tenant_id=$2 AND node_id=$3 AND id=$4 AND lifecycle_status != 'deleted'",
            )
            .bind(&operator_id)
            .bind(&tenant_id)
            .bind(&node_id)
            .bind(&version_id)
            .execute(&mut *connection)
            .await
            .map_err(internal_sql_error("delete dr_drive_node_version failed"))?
            .rows_affected();
            if let Some(storage_object_id) = storage_object_id.filter(|_| affected > 0) {
                sqlx::query(
                    "UPDATE dr_drive_storage_object
                     SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP
                     WHERE tenant_id=$2 AND node_id=$3 AND id=$4 AND lifecycle_status != 'deleted'",
                )
                .bind(&operator_id)
                .bind(&tenant_id)
                .bind(&node_id)
                .bind(&storage_object_id)
                .execute(&mut *connection)
                .await
                .map_err(internal_sql_error(
                    "delete dr_drive_storage_object from node version failed",
                ))?;
            }
            return Ok(affected);
        }

        let current_status: Option<String> = sqlx::query_scalar(
            "SELECT lifecycle_status
             FROM dr_drive_storage_object
             WHERE tenant_id=$1 AND node_id=$2 AND id=$3",
        )
        .bind(&tenant_id)
        .bind(&node_id)
        .bind(&version_id)
        .fetch_optional(&mut *connection)
        .await
        .map_err(internal_sql_error(
            "find dr_drive_storage_object version failed",
        ))?;
        let Some(current_status) = current_status else {
            return Err(not_found_problem("version not found"));
        };
        if current_status == "active" {
            let active_version_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(1)
                 FROM dr_drive_storage_object
                 WHERE tenant_id=$1 AND node_id=$2 AND lifecycle_status='active'",
            )
            .bind(&tenant_id)
            .bind(&node_id)
            .fetch_one(&mut *connection)
            .await
            .map_err(internal_sql_error(
                "count active dr_drive_storage_object versions failed",
            ))?;
            if active_version_count <= 1 {
                return Err(problem(
                    StatusCode::CONFLICT,
                    "conflict",
                    "cannot delete the only active version",
                    SdkWorkResultCode::Conflict,
                ));
            }
        }
        let affected = sqlx::query(
            "UPDATE dr_drive_storage_object
             SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP
             WHERE tenant_id=$2 AND node_id=$3 AND id=$4 AND lifecycle_status != 'deleted'",
        )
        .bind(&operator_id)
        .bind(&tenant_id)
        .bind(&node_id)
        .bind(&version_id)
        .execute(&mut *connection)
        .await
        .map_err(internal_sql_error(
            "delete dr_drive_storage_object version failed",
        ))?
        .rows_affected();
        if affected == 0 && current_status != "deleted" {
            return Err(not_found_problem("version not found"));
        }
        Ok(affected)
    }
    .await;
    let affected = match delete_result {
        Ok(affected) => {
            sqlx::query("COMMIT")
                .execute(&mut *connection)
                .await
                .map_err(internal_sql_error(
                    "commit version delete transaction failed",
                ))?;
            affected
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            return Err(error);
        }
    };
    drop(connection);
    if affected > 0 {
        record_change(
            &state.pool,
            &tenant_id,
            &node.space_id,
            Some(&node_id),
            drive_events::file_version::DELETED,
            &operator_id,
        )
        .await?;
    }
    let _deleted = affected > 0;
    Ok(no_content())
}
