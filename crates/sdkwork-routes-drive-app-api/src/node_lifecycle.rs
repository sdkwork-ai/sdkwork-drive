use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::dto::{DriveNodeResponse, NodeCommandRequest};
use crate::error::{
    internal_sql_error, not_found_problem, problem, ProblemDetail, SdkWorkResultCode,
};
use crate::node_repository::{collect_node_subtree, find_node};
use crate::response::success_resource;
use crate::route_change::{
    notify_committed, record_node_eligibility_changed_on_connection,
    resolve_node_location_on_connection,
};
use crate::space_repository::ensure_git_repository_space_root_accepts_node_type;
use crate::state::AppState;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_contract::drive::events::DriveNodeEligibility;
use sdkwork_drive_workspace_service::infrastructure::sql::begin_transaction_sql;
use sdkwork_drive_workspace_service::infrastructure::sql::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};
use sqlx::PgPool;
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) async fn set_node_lifecycle(
    state: AppState,
    ctx: &DriveRequestContext,
    node_id: String,
    _payload: NodeCommandRequest,
    lifecycle_status: &str,
    event_type: &str,
) -> Result<
    Json<SdkWorkApiResponse<SdkWorkResourceData<crate::dto::DriveNodeResponse>>>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let node = find_node(&state.pool, &tenant_id, &node_id).await?;
    acl::ensure_ctx_node_role(&state.pool, ctx, &node.space_id, &node_id, "writer").await?;
    let nodes_to_update = collect_node_subtree(&state.pool, &tenant_id, &node).await?;
    if lifecycle_status == "active" {
        ensure_restorable_subtree(&state.pool, &tenant_id, &nodes_to_update).await?;
    }
    let (old_eligibility, new_eligibility, reason) = match lifecycle_status {
        "active" => (
            DriveNodeEligibility::Ineligible,
            DriveNodeEligibility::Eligible,
            "NODE_RESTORED",
        ),
        "trashed" => (
            DriveNodeEligibility::Eligible,
            DriveNodeEligibility::Ineligible,
            "NODE_TRASHED",
        ),
        _ => {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                format!("unsupported node lifecycle transition event {event_type}"),
                SdkWorkResultCode::ValidationError,
            ));
        }
    };

    let mut connection = state.pool.acquire().await.map_err(|error| {
        crate::error::internal_problem(format!(
            "acquire node lifecycle transaction connection failed: {error}"
        ))
    })?;
    sqlx::query(begin_transaction_sql())
        .execute(&mut *connection)
        .await
        .map_err(internal_sql_error(
            "begin node lifecycle transaction failed",
        ))?;
    let tx_result: Result<u64, (StatusCode, Json<ProblemDetail>)> = async {
        ensure_managed_website_node_mutation_allowed(&mut connection, &tenant_id, &node_id)
            .await
            .map_err(crate::error::map_service_error)?;
        if lifecycle_status == "active" {
            rename_restoring_nodes_for_live_name_uniqueness(
                &mut connection,
                &tenant_id,
                &operator_id,
                &nodes_to_update,
            )
            .await?;
        }
        let mut locations = Vec::with_capacity(nodes_to_update.len());
        for node_to_update in &nodes_to_update {
            locations.push(
                resolve_node_location_on_connection(
                    &mut connection,
                    &tenant_id,
                    &node_to_update.space_id,
                    &node_to_update.id,
                )
                .await?,
            );
        }

        let mut updated_count = 0_u64;
        for (node_to_update, location) in nodes_to_update.iter().zip(&locations) {
            let affected = sqlx::query(
                "UPDATE dr_drive_node
                 SET lifecycle_status=$1, updated_by=$2, updated_at=CURRENT_TIMESTAMP, version=version + 1
                 WHERE tenant_id=$3 AND id=$4 AND lifecycle_status != 'deleted'",
            )
            .bind(lifecycle_status)
            .bind(&operator_id)
            .bind(&tenant_id)
            .bind(&node_to_update.id)
            .execute(&mut *connection)
            .await
            .map_err(internal_sql_error("update dr_drive_node lifecycle failed"))?
            .rows_affected();
            updated_count += affected;
            if affected > 0 {
                record_node_eligibility_changed_on_connection(
                    &mut connection,
                    &tenant_id,
                    ctx.organization_id.as_deref(),
                    &node_to_update.space_id,
                    &node_to_update.id,
                    &ctx.request_id,
                    &operator_id,
                    old_eligibility,
                    new_eligibility,
                    reason,
                    location,
                )
                .await?;
            }
        }
        Ok(updated_count)
    }
    .await;

    match tx_result {
        Ok(0) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            return Err(not_found_problem("node not found"));
        }
        Ok(_) => {
            sqlx::query("COMMIT")
                .execute(&mut *connection)
                .await
                .map_err(internal_sql_error(
                    "commit node lifecycle transaction failed",
                ))?;
            notify_committed(state.pool.clone());
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
            return Err(error);
        }
    }
    drop(connection);
    Ok(success_resource(
        crate::metadata_repository::present_drive_node(
            &state.pool,
            &tenant_id,
            find_node(&state.pool, &tenant_id, &node_id).await?,
        )
        .await?,
    ))
}

async fn rename_restoring_nodes_for_live_name_uniqueness(
    connection: &mut sqlx::PgConnection,
    tenant_id: &str,
    operator_id: &str,
    nodes: &[DriveNodeResponse],
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let mut reserved_names = BTreeSet::new();
    for node in nodes {
        let unique_name =
            resolve_restore_unique_name(connection, tenant_id, node, &reserved_names).await?;
        reserved_names.insert((
            node.space_id.clone(),
            node.parent_node_id.clone(),
            unique_name.clone(),
        ));
        if unique_name == node.node_name {
            continue;
        }
        sqlx::query(
            "UPDATE dr_drive_node
             SET node_name=$1, updated_by=$2, updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$3 AND id=$4 AND lifecycle_status != 'deleted'",
        )
        .bind(&unique_name)
        .bind(operator_id)
        .bind(tenant_id)
        .bind(&node.id)
        .execute(&mut *connection)
        .await
        .map_err(internal_sql_error("rename node before restore failed"))?;
    }
    Ok(())
}

async fn resolve_restore_unique_name(
    connection: &mut sqlx::PgConnection,
    tenant_id: &str,
    node: &DriveNodeResponse,
    reserved_names: &BTreeSet<(String, Option<String>, String)>,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    if restore_name_is_available(connection, tenant_id, node, &node.node_name, reserved_names)
        .await?
    {
        return Ok(node.node_name.clone());
    }

    let (stem, extension) = sdkwork_utils_rust::split_filename_stem_extension(&node.node_name);
    for index in 1..=9_999 {
        let candidate = sdkwork_utils_rust::format_numbered_filename_variant(
            &stem,
            index,
            extension.as_deref(),
        );
        if restore_name_is_available(connection, tenant_id, node, &candidate, reserved_names)
            .await?
        {
            return Ok(candidate);
        }
    }

    Err(problem(
        StatusCode::CONFLICT,
        "conflict",
        "no unique node name is available for restore",
        SdkWorkResultCode::Conflict,
    ))
}

async fn restore_name_is_available(
    connection: &mut sqlx::PgConnection,
    tenant_id: &str,
    node: &DriveNodeResponse,
    candidate: &str,
    reserved_names: &BTreeSet<(String, Option<String>, String)>,
) -> Result<bool, (StatusCode, Json<ProblemDetail>)> {
    if reserved_names.contains(&(
        node.space_id.clone(),
        node.parent_node_id.clone(),
        candidate.to_string(),
    )) {
        return Ok(false);
    }
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_node
         WHERE tenant_id=$1
           AND space_id=$2
           AND node_name=$3
           AND lifecycle_status='active'
           AND ((parent_node_id IS NULL AND $4 IS NULL) OR parent_node_id=$4)
           AND id != $5",
    )
    .bind(tenant_id)
    .bind(&node.space_id)
    .bind(candidate)
    .bind(node.parent_node_id.as_deref())
    .bind(&node.id)
    .fetch_one(&mut *connection)
    .await
    .map_err(internal_sql_error(
        "check restore node name availability failed",
    ))?;
    Ok(count == 0)
}

pub(crate) async fn ensure_restorable_subtree(
    pool: &PgPool,
    tenant_id: &str,
    nodes: &[DriveNodeResponse],
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let restoring_ids = nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    let restoring_types = nodes
        .iter()
        .map(|node| (node.id.as_str(), node.node_type.as_str()))
        .collect::<BTreeMap<_, _>>();

    for node in nodes {
        ensure_git_repository_space_root_accepts_node_type(
            pool,
            tenant_id,
            &node.space_id,
            node.parent_node_id.as_deref(),
            &node.node_type,
        )
        .await?;

        let Some(parent_id) = node.parent_node_id.as_deref() else {
            continue;
        };
        if restoring_ids.contains(parent_id) {
            if restoring_types.get(parent_id).copied() != Some("folder") {
                return Err(problem(
                    StatusCode::CONFLICT,
                    "conflict",
                    "parent node must be active before restore",
                    SdkWorkResultCode::Conflict,
                ));
            }
            continue;
        }

        let row = sqlx::query(
            "SELECT space_id, node_type, lifecycle_status
             FROM dr_drive_node
             WHERE tenant_id=$1 AND id=$2",
        )
        .bind(tenant_id)
        .bind(parent_id)
        .fetch_optional(pool)
        .await
        .map_err(internal_sql_error("validate restore parent failed"))?;
        let Some(row) = row else {
            return Err(problem(
                StatusCode::CONFLICT,
                "conflict",
                "parent node must be active before restore",
                SdkWorkResultCode::Conflict,
            ));
        };
        let parent_space_id: String = row.get("space_id");
        let parent_node_type: String = row.get("node_type");
        let parent_lifecycle_status: String = row.get("lifecycle_status");
        if parent_space_id != node.space_id
            || parent_node_type != "folder"
            || parent_lifecycle_status != "active"
        {
            return Err(problem(
                StatusCode::CONFLICT,
                "conflict",
                "parent node must be active before restore",
                SdkWorkResultCode::Conflict,
            ));
        }
    }

    Ok(())
}
