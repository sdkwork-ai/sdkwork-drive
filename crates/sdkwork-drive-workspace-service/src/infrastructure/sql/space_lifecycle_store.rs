use sqlx::PgConnection;
use sqlx::PgPool;

use crate::infrastructure::sql::begin_transaction_sql;
use crate::infrastructure::sql::managed_website_tree_guard::{
    ensure_managed_website_space_mutation_allowed, ManagedWebsiteTreeSystemOverride,
    ManagedWebsiteTreeSystemOverrideReason,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlSpaceLifecycleStore {
    pool: PgPool,
}

impl SqlSpaceLifecycleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_active_root_folder_id(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<Option<String>, DriveServiceError> {
        sqlx::query_scalar(
            "SELECT id FROM dr_drive_node \
             WHERE tenant_id = $1 AND space_id = $2 AND parent_node_id IS NULL AND lifecycle_status = 'active' \
             ORDER BY created_at ASC, id ASC LIMIT 1",
        )
        .bind(tenant_id)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read team space root folder failed: {error}"))
        })
    }

    pub async fn insert_team_space_root_folder(
        &self,
        root_folder_id: &str,
        tenant_id: &str,
        space_id: &str,
        root_name: &str,
        creator_user_id: &str,
    ) -> Result<(), DriveServiceError> {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
             ) VALUES ($1, $2, $3, NULL, 'folder', $4, 'ready', 'active', 1, $5, $5)",
        )
        .bind(root_folder_id)
        .bind(tenant_id)
        .bind(space_id)
        .bind(root_name)
        .bind(creator_user_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("insert team space root folder failed: {error}"))
        })?;
        Ok(())
    }

    pub async fn retire_space_contents(
        &self,
        tenant_id: &str,
        space_id: &str,
        operator_id: &str,
    ) -> Result<i64, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire retire space contents connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin retire space contents transaction failed: {error}"
                ))
            })?;

        match Self::retire_space_contents_on_connection(
            &mut connection,
            tenant_id,
            space_id,
            operator_id,
        )
        .await
        {
            Ok(deleted_node_count) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit retire space contents transaction failed: {error}"
                        ))
                    })?;
                Ok(deleted_node_count)
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }

    pub async fn retire_space_contents_on_connection(
        connection: &mut PgConnection,
        tenant_id: &str,
        space_id: &str,
        operator_id: &str,
    ) -> Result<i64, DriveServiceError> {
        let system_override =
            match ensure_managed_website_space_mutation_allowed(connection, tenant_id, space_id)
                .await
            {
                Ok(()) => None,
                Err(DriveServiceError::Conflict(_)) => {
                    Some(ManagedWebsiteTreeSystemOverride::authorize(
                        ManagedWebsiteTreeSystemOverrideReason::TenantAuthorizedSpaceRetirement,
                        operator_id,
                    )?)
                }
                Err(error) => return Err(error),
            };

        sqlx::query(
            "UPDATE dr_drive_storage_object
             SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP
             WHERE tenant_id=$2
               AND lifecycle_status != 'deleted'
               AND node_id IN (
                 SELECT id
                 FROM dr_drive_node
                 WHERE tenant_id=$2 AND space_id=$3 AND lifecycle_status != 'deleted'
               )",
        )
        .bind(operator_id)
        .bind(tenant_id)
        .bind(space_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "retire dr_drive_storage_object for space failed: {error}"
            ))
        })?;

        let retired_nodes = sqlx::query(
            "UPDATE dr_drive_node
             SET lifecycle_status='deleted', updated_by=$1, updated_at=CURRENT_TIMESTAMP, version=version + 1
             WHERE tenant_id=$2 AND space_id=$3 AND lifecycle_status != 'deleted'",
        )
        .bind(operator_id)
        .bind(tenant_id)
        .bind(space_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("retire dr_drive_node for space failed: {error}"))
        })?;

        if let Some(system_override) = system_override {
            system_override
                .record_on_connection(connection, tenant_id, "drive_space", space_id)
                .await?;
        }

        Ok(retired_nodes.rows_affected() as i64)
    }
}
