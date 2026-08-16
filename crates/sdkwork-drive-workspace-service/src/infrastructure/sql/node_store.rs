use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::Row;
use sqlx::{PgConnection, PgPool};

use crate::domain::node::{DriveNode, DriveNodeType};
use crate::domain::space::DriveSpaceType;
use crate::infrastructure::sql::begin_transaction_sql;
use crate::infrastructure::sql::sql_error::is_unique_constraint_violation;
use crate::ports::node_store::{DriveNodeStore, NewDriveNode};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlNodeStore {
    pool: PgPool,
}

impl SqlNodeStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveNodeStore for SqlNodeStore {
    async fn find_active_space_type(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<Option<DriveSpaceType>, DriveServiceError> {
        let space_type_raw = sqlx::query_scalar::<_, String>(
            "SELECT space_type
             FROM dr_drive_space
             WHERE tenant_id=$1
               AND id=$2
               AND lifecycle_status='active'",
        )
        .bind(tenant_id)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read dr_drive_space type failed: {error}"))
        })?;

        space_type_raw
            .as_deref()
            .map(|raw| {
                DriveSpaceType::try_from_str(raw).ok_or_else(|| {
                    DriveServiceError::Internal(format!("unknown space_type in database: {raw}"))
                })
            })
            .transpose()
    }

    async fn find_active_node_in_space(
        &self,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
    ) -> Result<Option<DriveNode>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, space_id, space_type, parent_node_id, shortcut_target_node_id,
                    node_type, node_name, lifecycle_status, version
             FROM dr_drive_node
             WHERE tenant_id=$1
               AND space_id=$2
               AND id=$3
               AND lifecycle_status='active'",
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read dr_drive_node parent failed: {error}"))
        })?;

        row.as_ref().map(drive_node_from_row).transpose()
    }

    async fn exists_live_name_in_parent(
        &self,
        tenant_id: &str,
        space_id: &str,
        parent_node_id: Option<&str>,
        node_name: &str,
    ) -> Result<bool, DriveServiceError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(1)
             FROM dr_drive_node
             WHERE tenant_id=$1
               AND space_id=$2
               AND node_name=$3
               AND lifecycle_status='active'
               AND (
                   (parent_node_id IS NULL AND $4 IS NULL)
                   OR parent_node_id = $4
               )",
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(node_name)
        .bind(parent_node_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("check dr_drive_node duplicate failed: {error}"))
        })?;

        Ok(count > 0)
    }

    async fn insert_node(&self, new_node: &NewDriveNode) -> Result<DriveNode, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire node insert transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin node insert transaction failed: {error}"
                ))
            })?;

        let result = insert_node_in_transaction(&mut connection, new_node).await;
        match result {
            Ok(node) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit node insert transaction failed: {error}"
                        ))
                    })?;
                Ok(node)
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }
}

async fn insert_node_in_transaction(
    connection: &mut PgConnection,
    new_node: &NewDriveNode,
) -> Result<DriveNode, DriveServiceError> {
    super::managed_website_tree_guard::ensure_managed_website_parent_mutation_allowed(
        connection,
        &new_node.tenant_id,
        &new_node.space_id,
        new_node.parent_node_id.as_deref(),
    )
    .await?;

    sqlx::query(
        "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, space_type, parent_node_id, shortcut_target_node_id,
                node_type, node_name, content_state, lifecycle_status,
                version, created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'empty', $9, 1, $10, $11)",
    )
    .bind(&new_node.id)
    .bind(&new_node.tenant_id)
    .bind(&new_node.space_id)
    .bind(&new_node.space_type)
    .bind(&new_node.parent_node_id)
    .bind(&new_node.shortcut_target_node_id)
    .bind(&new_node.node_type)
    .bind(&new_node.node_name)
    .bind(&new_node.lifecycle_status)
    .bind(&new_node.created_by)
    .bind(&new_node.updated_by)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        let message = error.to_string();
        if is_unique_constraint_violation(&message) {
            return DriveServiceError::Conflict("node name already exists in parent".to_string());
        }
        DriveServiceError::Internal(format!("insert dr_drive_node failed: {message}"))
    })?;

    let row = sqlx::query(
        "SELECT id, tenant_id, space_id, space_type, parent_node_id, shortcut_target_node_id,
                    node_type, node_name, lifecycle_status, version
             FROM dr_drive_node
             WHERE id=$1",
    )
    .bind(&new_node.id)
    .fetch_one(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("read inserted dr_drive_node failed: {error}"))
    })?;

    drive_node_from_row(&row)
}

fn drive_node_from_row(row: &PgRow) -> Result<DriveNode, DriveServiceError> {
    let node_type_raw: String = row.get("node_type");
    let node_type = DriveNodeType::try_from_str(&node_type_raw).ok_or_else(|| {
        DriveServiceError::Internal(format!("unknown node_type in database: {node_type_raw}"))
    })?;
    let space_type_raw: String = row.get("space_type");
    let space_type = DriveSpaceType::try_from_str(&space_type_raw).ok_or_else(|| {
        DriveServiceError::Internal(format!(
            "unknown node space_type in database: {space_type_raw}"
        ))
    })?;

    Ok(DriveNode {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        space_type,
        parent_node_id: row.get("parent_node_id"),
        shortcut_target_node_id: row.get("shortcut_target_node_id"),
        node_type,
        node_name: row.get("node_name"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
    })
}
