use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::Row;
use sqlx::{PgConnection, PgPool};

use crate::domain::node_version::{
    CreateDriveNodeVersionCommand, DriveNodeVersion, DriveNodeVersionChangeSource,
    DriveNodeVersionKind,
};
use crate::infrastructure::sql::begin_transaction_sql;
use crate::infrastructure::sql::sql_error::is_unique_constraint_violation;
use crate::ports::node_version_store::DriveNodeVersionStore;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlDriveNodeVersionStore {
    pool: PgPool,
}

impl SqlDriveNodeVersionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveNodeVersionStore for SqlDriveNodeVersionStore {
    async fn create(
        &self,
        command: CreateDriveNodeVersionCommand,
    ) -> Result<DriveNodeVersion, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire node version transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin node version transaction failed: {error}"
                ))
            })?;

        let result = create_node_version_in_transaction(&mut connection, &command).await;
        match result {
            Ok(version) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit node version transaction failed: {error}"
                        ))
                    })?;
                Ok(version)
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }

    async fn find_by_id(
        &self,
        tenant_id: &str,
        node_id: &str,
        version_id: &str,
    ) -> Result<Option<DriveNodeVersion>, DriveServiceError> {
        let sql = node_version_select_sql("WHERE tenant_id=$1 AND node_id=$2 AND id=$3");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .bind(tenant_id)
            .bind(node_id)
            .bind(version_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("read dr_drive_node_version failed: {error}"))
            })?;

        row.as_ref().map(map_row_to_node_version).transpose()
    }

    async fn list_by_node(
        &self,
        tenant_id: &str,
        node_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DriveNodeVersion>, DriveServiceError> {
        let rows = sqlx::query(
            "SELECT id, tenant_id, space_id, node_id, version_no, storage_object_id,
                    content_type, content_length, checksum_sha256_hex,
                    version_kind, version_label, change_source, change_summary,
                    restored_from_version_id, app_id, app_resource_type, app_resource_id,
                    scene, source, lifecycle_status, created_by, updated_by,
                    CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at
             FROM dr_drive_node_version
             WHERE tenant_id=$1 AND node_id=$2
             ORDER BY version_no DESC
             LIMIT $3 OFFSET $4",
        )
        .bind(tenant_id)
        .bind(node_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("list dr_drive_node_version failed: {error}"))
        })?;

        rows.iter().map(map_row_to_node_version).collect()
    }
}

async fn create_node_version_in_transaction(
    connection: &mut PgConnection,
    command: &CreateDriveNodeVersionCommand,
) -> Result<DriveNodeVersion, DriveServiceError> {
    super::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed(
        connection,
        &command.tenant_id,
        &command.node_id,
    )
    .await?;

    let result = sqlx::query(
        "INSERT INTO dr_drive_node_version (
            id, tenant_id, space_id, node_id, version_no, storage_object_id,
            content_type, content_length, checksum_sha256_hex,
            version_kind, version_label, change_source, change_summary,
            restored_from_version_id, app_id, app_resource_type, app_resource_id,
            scene, source, lifecycle_status, created_by, updated_by
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
            $13, $14, $15, $16, $17, $18, $19, 'active', $20, $20
         )",
    )
    .bind(&command.id)
    .bind(&command.tenant_id)
    .bind(&command.space_id)
    .bind(&command.node_id)
    .bind(command.version_no)
    .bind(command.storage_object_id.as_deref())
    .bind(&command.content_type)
    .bind(command.content_length)
    .bind(&command.checksum_sha256_hex)
    .bind(command.version_kind.as_str())
    .bind(command.version_label.as_deref())
    .bind(command.change_source.as_str())
    .bind(command.change_summary.as_deref())
    .bind(command.restored_from_version_id.as_deref())
    .bind(command.app_id.as_deref())
    .bind(command.app_resource_type.as_deref())
    .bind(command.app_resource_id.as_deref())
    .bind(command.scene.as_deref())
    .bind(command.source.as_deref())
    .bind(&command.operator_id)
    .execute(&mut *connection)
    .await;

    if let Err(error) = result {
        let message = error.to_string();
        if is_unique_constraint_violation(&message) {
            return Err(DriveServiceError::Conflict(
                "node version already exists for tenant/node/version".to_string(),
            ));
        }
        return Err(DriveServiceError::Internal(format!(
            "insert dr_drive_node_version failed: {message}"
        )));
    }

    let sql = node_version_select_sql("WHERE tenant_id=$1 AND node_id=$2 AND id=$3");
    let row = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
        .bind(&command.tenant_id)
        .bind(&command.node_id)
        .bind(&command.id)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "read inserted dr_drive_node_version failed: {error}"
            ))
        })?;

    row.as_ref()
        .map(map_row_to_node_version)
        .transpose()?
        .ok_or_else(|| {
            DriveServiceError::Internal("read inserted dr_drive_node_version failed".to_string())
        })
}

fn node_version_select_sql(predicate: &str) -> String {
    format!(
        "SELECT id, tenant_id, space_id, node_id, version_no, storage_object_id,
                content_type, content_length, checksum_sha256_hex,
                version_kind, version_label, change_source, change_summary,
                restored_from_version_id, app_id, app_resource_type, app_resource_id,
                scene, source, lifecycle_status, created_by, updated_by,
                CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at
         FROM dr_drive_node_version
         {predicate}"
    )
}

fn map_row_to_node_version(row: &PgRow) -> Result<DriveNodeVersion, DriveServiceError> {
    let version_kind_raw: String = row.get("version_kind");
    let version_kind = DriveNodeVersionKind::try_from_str(&version_kind_raw).ok_or_else(|| {
        DriveServiceError::Internal(format!(
            "unknown dr_drive_node_version version_kind: {version_kind_raw}"
        ))
    })?;
    let change_source_raw: String = row.get("change_source");
    let change_source =
        DriveNodeVersionChangeSource::try_from_str(&change_source_raw).ok_or_else(|| {
            DriveServiceError::Internal(format!(
                "unknown dr_drive_node_version change_source: {change_source_raw}"
            ))
        })?;

    Ok(DriveNodeVersion {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        node_id: row.get("node_id"),
        version_no: row.get("version_no"),
        storage_object_id: row.get("storage_object_id"),
        content_type: row.get("content_type"),
        content_length: row.get("content_length"),
        checksum_sha256_hex: row.get("checksum_sha256_hex"),
        version_kind,
        version_label: row.get("version_label"),
        change_source,
        change_summary: row.get("change_summary"),
        restored_from_version_id: row.get("restored_from_version_id"),
        app_id: row.get("app_id"),
        app_resource_type: row.get("app_resource_type"),
        app_resource_id: row.get("app_resource_id"),
        scene: row.get("scene"),
        source: row.get("source"),
        lifecycle_status: row.get("lifecycle_status"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
