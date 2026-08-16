use crate::dto::{LabelSummaryResponse, NodeLabelResponse, NodePropertyResponse};
use crate::error::{internal_sql_error, not_found_problem, ProblemDetail};
use crate::mappers::{map_label_summary_row, map_node_label_row, map_node_property_row};
use axum::http::StatusCode;
use axum::Json;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::HashMap;

pub(crate) const UI_FOLDER_COLOR_PROPERTY_KEY: &str = "ui.folderColor";
const UI_FOLDER_COLOR_VISIBILITY: &str = "private";

pub(crate) async fn find_node_property(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    property_key: &str,
    visibility: &str,
) -> Result<NodePropertyResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT id, tenant_id, node_id, property_key, property_value, visibility,
                lifecycle_status, version
         FROM dr_drive_node_property
         WHERE tenant_id=$1
           AND node_id=$2
           AND property_key=$3
           AND visibility=$4
           AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(node_id)
    .bind(property_key)
    .bind(visibility)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find dr_drive_node_property failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("property not found"));
    };
    Ok(map_node_property_row(&row))
}

pub(crate) async fn enrich_drive_nodes_with_folder_colors(
    pool: &PgPool,
    tenant_id: &str,
    nodes: &mut [crate::dto::DriveNodeResponse],
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let folder_ids = nodes
        .iter()
        .filter(|node| node.node_type == "folder")
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    if folder_ids.is_empty() {
        return Ok(());
    }

    let colors = load_ui_folder_colors_for_nodes(pool, tenant_id, &folder_ids).await?;
    for node in nodes.iter_mut() {
        if node.node_type == "folder" {
            node.folder_color = colors.get(&node.id).cloned();
        }
    }
    Ok(())
}

pub(crate) async fn present_drive_node(
    pool: &PgPool,
    tenant_id: &str,
    mut node: crate::dto::DriveNodeResponse,
) -> Result<crate::dto::DriveNodeResponse, (StatusCode, Json<ProblemDetail>)> {
    enrich_drive_nodes_with_folder_colors(pool, tenant_id, std::slice::from_mut(&mut node)).await?;
    Ok(node)
}

pub(crate) async fn present_node_list(
    pool: &PgPool,
    tenant_id: &str,
    mut items: Vec<crate::dto::DriveNodeResponse>,
    page: crate::dto::PageRequest,
    next_page_token: Option<String>,
    incomplete_page: bool,
) -> Result<
    axum::Json<
        sdkwork_utils_rust::SdkWorkApiResponse<
            crate::response::DriveListPageData<crate::dto::DriveNodeResponse>,
        >,
    >,
    (
        axum::http::StatusCode,
        axum::Json<crate::error::ProblemDetail>,
    ),
> {
    enrich_drive_nodes_with_folder_colors(pool, tenant_id, &mut items).await?;
    Ok(crate::response::success_list_page(
        items,
        page,
        next_page_token,
        incomplete_page,
    ))
}

async fn load_ui_folder_colors_for_nodes(
    pool: &PgPool,
    tenant_id: &str,
    node_ids: &[String],
) -> Result<HashMap<String, String>, (StatusCode, Json<ProblemDetail>)> {
    let mut colors = HashMap::new();
    for chunk in node_ids.chunks(100) {
        if chunk.is_empty() {
            continue;
        }
        let placeholders = (0..chunk.len())
            .map(|index| format!("${}", index + 2))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT node_id, property_value
             FROM dr_drive_node_property
             WHERE tenant_id=$1
               AND property_key='{UI_FOLDER_COLOR_PROPERTY_KEY}'
               AND visibility='{UI_FOLDER_COLOR_VISIBILITY}'
               AND lifecycle_status='active'
               AND node_id IN ({placeholders})"
        );
        let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).bind(tenant_id);
        for node_id in chunk {
            query = query.bind(node_id);
        }
        let rows = query
            .fetch_all(pool)
            .await
            .map_err(internal_sql_error("load ui.folderColor properties failed"))?;
        for row in rows {
            let node_id: String = row.get("node_id");
            let property_value: String = row.get("property_value");
            if !property_value.trim().is_empty() {
                colors.insert(node_id, property_value);
            }
        }
    }
    Ok(colors)
}

pub(crate) async fn find_label(
    pool: &PgPool,
    tenant_id: &str,
    label_id: &str,
) -> Result<LabelSummaryResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT id AS label_id, tenant_id, label_key, display_name, color, description,
                lifecycle_status AS label_lifecycle_status,
                version AS label_version
         FROM dr_drive_label
         WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(label_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find dr_drive_label failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("label not found"));
    };
    Ok(map_label_summary_row(&row))
}

pub(crate) async fn find_node_label(
    pool: &PgPool,
    tenant_id: &str,
    node_id: &str,
    label_id: &str,
) -> Result<NodeLabelResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT nl.id, nl.tenant_id, nl.node_id, nl.label_id,
                nl.lifecycle_status, nl.version,
                l.label_key, l.display_name, l.color, l.description,
                l.lifecycle_status AS label_lifecycle_status,
                l.version AS label_version
         FROM dr_drive_node_label nl
         INNER JOIN dr_drive_label l
            ON l.tenant_id=nl.tenant_id
           AND l.id=nl.label_id
           AND l.lifecycle_status='active'
         WHERE nl.tenant_id=$1
           AND nl.node_id=$2
           AND nl.label_id=$3
           AND nl.lifecycle_status='active'",
    )
    .bind(tenant_id)
    .bind(node_id)
    .bind(label_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find dr_drive_node_label failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("node label not found"));
    };
    Ok(map_node_label_row(&row))
}
