use crate::constants::MAX_LIFECYCLE_SUBTREE_NODES;
use crate::dto::DriveNodeResponse;
use crate::error::{
    internal_sql_error, not_found_problem, problem, ProblemDetail, SdkWorkResultCode,
};
use crate::mappers::map_node_row;
use crate::space_repository::validate_space_exists;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_workspace_service::infrastructure::sql::NODE_API_SELECT_COLUMNS;
use sqlx::PgPool;
use std::collections::{BTreeSet, VecDeque};

pub(crate) async fn find_node(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
) -> Result<DriveNodeResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "SELECT {NODE_API_SELECT_COLUMNS}
         FROM dr_drive_node
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status != 'deleted'",
    )))
    .bind(tenant_id)
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find dr_drive_node failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("node not found"));
    };
    Ok(map_node_row(&row))
}

pub(crate) async fn find_active_node(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
) -> Result<DriveNodeResponse, (StatusCode, Json<ProblemDetail>)> {
    let node = find_node(pool, tenant_id, node_id).await?;
    if node.lifecycle_status != "active" {
        return Err(not_found_problem("node not found"));
    }
    validate_space_exists(pool, tenant_id, &node.space_id).await?;
    Ok(node)
}

pub(crate) async fn collect_node_subtree(
    pool: &PgPool,
    tenant_id: &str,
    root: &DriveNodeResponse,
) -> Result<Vec<DriveNodeResponse>, (StatusCode, Json<ProblemDetail>)> {
    let mut nodes = vec![root.clone()];
    let mut queue = VecDeque::from([(root.id.clone(), 0_usize)]);
    let mut visited = BTreeSet::from([root.id.clone()]);

    while let Some((parent_id, depth)) = queue.pop_front() {
        if depth > 128 {
            return Err(problem(
                StatusCode::CONFLICT,
                "conflict",
                "node hierarchy exceeds maximum lifecycle depth",
                SdkWorkResultCode::Conflict,
            ));
        }

        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {NODE_API_SELECT_COLUMNS}
             FROM dr_drive_node
             WHERE tenant_id=$1
               AND parent_node_id=$2
               AND lifecycle_status != 'deleted'
             ORDER BY node_type ASC, node_name ASC, id ASC",
        )))
        .bind(tenant_id)
        .bind(&parent_id)
        .fetch_all(pool)
        .await
        .map_err(internal_sql_error("collect dr_drive_node subtree failed"))?;

        for row in rows {
            let child = map_node_row(&row);
            if !visited.insert(child.id.clone()) {
                return Err(problem(
                    StatusCode::CONFLICT,
                    "conflict",
                    "node hierarchy contains a cycle",
                    SdkWorkResultCode::Conflict,
                ));
            }
            queue.push_back((child.id.clone(), depth + 1));
            nodes.push(child);
            if nodes.len() > MAX_LIFECYCLE_SUBTREE_NODES {
                return Err(problem(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "subtree too large",
                    format!(
                        "node subtree exceeds the maximum of {MAX_LIFECYCLE_SUBTREE_NODES} nodes"
                    ),
                    SdkWorkResultCode::ValidationError,
                ));
            }
        }
    }

    Ok(nodes)
}

pub(crate) async fn resolve_node_path(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
) -> Result<Vec<DriveNodeResponse>, (StatusCode, Json<ProblemDetail>)> {
    let mut current_node_id = Some(node_id.to_string());
    let mut reversed = Vec::<DriveNodeResponse>::new();
    for _ in 0..128 {
        let Some(next_node_id) = current_node_id.take() else {
            reversed.reverse();
            return Ok(reversed);
        };
        let node = find_node(pool, tenant_id, &next_node_id).await?;
        current_node_id = node.parent_node_id.clone();
        reversed.push(node);
    }
    Err(problem(
        StatusCode::CONFLICT,
        "conflict",
        "node parent chain exceeds maximum depth",
        SdkWorkResultCode::Conflict,
    ))
}
