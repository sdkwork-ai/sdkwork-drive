use async_trait::async_trait;
use sqlx::{PgConnection, PgPool, Row};

use crate::infrastructure::sql::begin_transaction_sql;
use crate::ports::workspace_store::{
    DriveWorkspaceNodeRecord, DriveWorkspaceStore, NewDriveWorkspaceNodeRecord,
    NewDriveWorkspaceObjectRecord,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlDriveWorkspaceStore {
    pool: PgPool,
}

impl SqlDriveWorkspaceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveWorkspaceStore for SqlDriveWorkspaceStore {
    async fn find_child_node(
        &self,
        tenant_id: &str,
        space_id: &str,
        parent_node_id: Option<&str>,
        node_name: &str,
    ) -> Result<Option<DriveWorkspaceNodeRecord>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT
                n.id,
                n.parent_node_id,
                n.node_type,
                n.node_name,
                CAST(n.updated_at AS TEXT) AS updated_at,
                o.content_type,
                o.content_length,
                (
                    SELECT COUNT(1)
                    FROM dr_drive_node child
                    WHERE child.tenant_id=n.tenant_id
                      AND child.space_id=n.space_id
                      AND child.parent_node_id=n.id
                      AND child.lifecycle_status='active'
                ) AS children_count
             FROM dr_drive_node n
             LEFT JOIN dr_drive_storage_object o
               ON o.tenant_id=n.tenant_id
              AND o.node_id=n.id
              AND o.lifecycle_status='active'
              AND o.version_no=(
                  SELECT MAX(o2.version_no)
                  FROM dr_drive_storage_object o2
                  WHERE o2.tenant_id=n.tenant_id
                    AND o2.node_id=n.id
                    AND o2.lifecycle_status='active'
              )
             WHERE n.tenant_id=$1
               AND n.space_id=$2
               AND n.node_name=$3
               AND n.lifecycle_status='active'
               AND (
                   (n.parent_node_id IS NULL AND $4 IS NULL)
                   OR n.parent_node_id=$4
               )
             LIMIT 1",
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(node_name)
        .bind(parent_node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find workspace child node failed: {error}"))
        })?;

        row.map(|row| map_workspace_node_row(&row)).transpose()
    }

    async fn ensure_node(
        &self,
        record: NewDriveWorkspaceNodeRecord,
    ) -> Result<DriveWorkspaceNodeRecord, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire workspace node transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin workspace node transaction failed: {error}"
                ))
            })?;

        let result = ensure_workspace_node_in_transaction(&mut connection, &record).await;
        match result {
            Ok(node) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit workspace node transaction failed: {error}"
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

    async fn ensure_object_ref(
        &self,
        record: NewDriveWorkspaceObjectRecord,
    ) -> Result<(), DriveServiceError> {
        if self.active_object_matches(&record).await? {
            return Ok(());
        }

        for _ in 0..5 {
            let next_version_no: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(version_no), 0) + 1
                 FROM dr_drive_storage_object
                 WHERE tenant_id=$1 AND node_id=$2",
            )
            .bind(&record.tenant_id)
            .bind(&record.node_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "read workspace object version failed: {error}"
                ))
            })?;

            let result = self
                .replace_active_object_ref(&record, next_version_no)
                .await?;

            if result {
                return Ok(());
            }
            if self.active_object_matches(&record).await? {
                return Ok(());
            }
        }

        Err(DriveServiceError::Conflict(
            "workspace storage object version changed concurrently".to_string(),
        ))
    }

    async fn mark_node_content_ready(
        &self,
        tenant_id: &str,
        node_id: &str,
        operator_id: &str,
    ) -> Result<(), DriveServiceError> {
        let row = sqlx::query(
            "SELECT content_state, head_content_type
             FROM dr_drive_node
             WHERE tenant_id=$1 AND id=$2 AND lifecycle_status='active'",
        )
        .bind(tenant_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "read workspace node content state failed: {error}"
            ))
        })?;
        let Some(row) = row else {
            return Err(DriveServiceError::NotFound(
                "workspace node not found".to_string(),
            ));
        };
        let content_state: String = row.get("content_state");
        let head_content_type: Option<String> = row.try_get("head_content_type").ok().flatten();
        if content_state == "ready" && head_content_type.is_some() {
            return Ok(());
        }

        super::node_head_metadata::sync_file_node_head_from_active_storage(
            &self.pool,
            tenant_id,
            node_id,
            operator_id,
        )
        .await
    }

    async fn list_children(
        &self,
        tenant_id: &str,
        space_id: &str,
        parent_node_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DriveWorkspaceNodeRecord>, DriveServiceError> {
        let rows = sqlx::query(
            "SELECT
                n.id,
                n.parent_node_id,
                n.node_type,
                n.node_name,
                CAST(n.updated_at AS TEXT) AS updated_at,
                o.content_type,
                o.content_length,
                (
                    SELECT COUNT(1)
                    FROM dr_drive_node child
                    WHERE child.tenant_id=n.tenant_id
                      AND child.space_id=n.space_id
                      AND child.parent_node_id=n.id
                      AND child.lifecycle_status='active'
                ) AS children_count
             FROM dr_drive_node n
             LEFT JOIN dr_drive_storage_object o
               ON o.tenant_id=n.tenant_id
              AND o.node_id=n.id
              AND o.lifecycle_status='active'
              AND o.version_no=(
                  SELECT MAX(o2.version_no)
                  FROM dr_drive_storage_object o2
                  WHERE o2.tenant_id=n.tenant_id
                    AND o2.node_id=n.id
                    AND o2.lifecycle_status='active'
              )
             WHERE n.tenant_id=$1
               AND n.space_id=$2
               AND n.lifecycle_status='active'
               AND (
                   (n.parent_node_id IS NULL AND $3 IS NULL)
                   OR n.parent_node_id=$3
               )
             ORDER BY
               CASE n.node_type WHEN 'folder' THEN 0 ELSE 1 END,
               lower(n.node_name),
               n.node_name,
               n.id
             LIMIT $4 OFFSET $5",
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(parent_node_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("list workspace child nodes failed: {error}"))
        })?;

        rows.iter().map(map_workspace_node_row).collect()
    }

    async fn find_node(
        &self,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
    ) -> Result<Option<DriveWorkspaceNodeRecord>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT
                n.id,
                n.parent_node_id,
                n.node_type,
                n.node_name,
                CAST(n.updated_at AS TEXT) AS updated_at,
                o.content_type,
                o.content_length,
                (
                    SELECT COUNT(1)
                    FROM dr_drive_node child
                    WHERE child.tenant_id=n.tenant_id
                      AND child.space_id=n.space_id
                      AND child.parent_node_id=n.id
                      AND child.lifecycle_status='active'
                ) AS children_count
             FROM dr_drive_node n
             LEFT JOIN dr_drive_storage_object o
               ON o.tenant_id=n.tenant_id
              AND o.node_id=n.id
              AND o.lifecycle_status='active'
              AND o.version_no=(
                  SELECT MAX(o2.version_no)
                  FROM dr_drive_storage_object o2
                  WHERE o2.tenant_id=n.tenant_id
                    AND o2.node_id=n.id
                    AND o2.lifecycle_status='active'
              )
             WHERE n.tenant_id=$1
               AND n.space_id=$2
               AND n.id=$3
               AND n.lifecycle_status='active'
             LIMIT 1",
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find workspace node failed: {error}"))
        })?;

        row.map(|row| map_workspace_node_row(&row)).transpose()
    }
}

impl SqlDriveWorkspaceStore {
    async fn active_object_matches(
        &self,
        record: &NewDriveWorkspaceObjectRecord,
    ) -> Result<bool, DriveServiceError> {
        let row = sqlx::query(
            "SELECT content_type, content_length, checksum_sha256_hex
             FROM dr_drive_storage_object
             WHERE tenant_id=$1
               AND node_id=$2
               AND bucket=$3
               AND object_key=$4
               AND lifecycle_status='active'
             ORDER BY version_no DESC
             LIMIT 1",
        )
        .bind(&record.tenant_id)
        .bind(&record.node_id)
        .bind(&record.bucket)
        .bind(&record.object_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("find workspace storage object failed: {error}"))
        })?;

        let Some(row) = row else {
            return Ok(false);
        };
        Ok(row.get::<String, _>("content_type") == record.content_type
            && row.get::<i64, _>("content_length") == record.content_length
            && row.get::<String, _>("checksum_sha256_hex") == record.checksum_sha256_hex)
    }

    async fn replace_active_object_ref(
        &self,
        record: &NewDriveWorkspaceObjectRecord,
        next_version_no: i64,
    ) -> Result<bool, DriveServiceError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            DriveServiceError::Internal(format!(
                "acquire workspace storage transaction connection failed: {error}"
            ))
        })?;
        sqlx::query(begin_transaction_sql())
            .execute(&mut *connection)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "begin workspace storage object transaction failed: {error}"
                ))
            })?;

        let result = self
            .replace_active_object_ref_in_transaction(&mut connection, record, next_version_no)
            .await;
        match result {
            Ok(true) => {
                sqlx::query("COMMIT")
                    .execute(&mut *connection)
                    .await
                    .map_err(|error| {
                        DriveServiceError::Internal(format!(
                            "commit workspace storage object transaction failed: {error}"
                        ))
                    })?;
                Ok(true)
            }
            Ok(false) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Ok(false)
            }
            Err(error) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
                Err(error)
            }
        }
    }

    async fn replace_active_object_ref_in_transaction(
        &self,
        connection: &mut PgConnection,
        record: &NewDriveWorkspaceObjectRecord,
        next_version_no: i64,
    ) -> Result<bool, DriveServiceError> {
        super::managed_website_tree_guard::ensure_managed_website_node_mutation_allowed(
            connection,
            &record.tenant_id,
            &record.node_id,
        )
        .await?;

        sqlx::query(
            "UPDATE dr_drive_storage_object
             SET lifecycle_status='deleted',
                 updated_by=$5,
                 updated_at=CURRENT_TIMESTAMP
             WHERE tenant_id=$1
               AND node_id=$2
               AND bucket=$3
               AND object_key=$4
               AND lifecycle_status='active'",
        )
        .bind(&record.tenant_id)
        .bind(&record.node_id)
        .bind(&record.bucket)
        .bind(&record.object_key)
        .bind(&record.operator_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("retire workspace storage object failed: {error}"))
        })?;

        let result = sqlx::query(
            "INSERT INTO dr_drive_storage_object (
                id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                scene, source, content_type, content_length, checksum_sha256_hex, lifecycle_status,
                created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL, $8, $9, $10, 'active', $11, $11)
             ON CONFLICT DO NOTHING",
        )
        .bind(&record.id)
        .bind(&record.tenant_id)
        .bind(&record.node_id)
        .bind(next_version_no)
        .bind(&record.storage_provider_id)
        .bind(&record.bucket)
        .bind(&record.object_key)
        .bind(&record.content_type)
        .bind(record.content_length)
        .bind(&record.checksum_sha256_hex)
        .bind(&record.operator_id)
        .execute(&mut *connection)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("ensure workspace storage object failed: {error}"))
        })?;

        Ok(result.rows_affected() > 0)
    }
}

async fn ensure_workspace_node_in_transaction(
    connection: &mut PgConnection,
    record: &NewDriveWorkspaceNodeRecord,
) -> Result<DriveWorkspaceNodeRecord, DriveServiceError> {
    if let Some(existing) = find_workspace_child_node_in_transaction(
        connection,
        &record.tenant_id,
        &record.space_id,
        record.parent_node_id.as_deref(),
        &record.node_name,
    )
    .await?
    {
        return Ok(existing);
    }

    super::managed_website_tree_guard::ensure_managed_website_parent_mutation_allowed(
        connection,
        &record.tenant_id,
        &record.space_id,
        record.parent_node_id.as_deref(),
    )
    .await?;

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', 1, $8, $8)
        ON CONFLICT DO NOTHING",
    )
    .bind(&record.id)
    .bind(&record.tenant_id)
    .bind(&record.space_id)
    .bind(record.parent_node_id.as_deref())
    .bind(&record.node_type)
    .bind(&record.node_name)
    .bind(&record.content_state)
    .bind(&record.operator_id)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("ensure workspace node failed: {error}"))
    })?;

    find_workspace_child_node_in_transaction(
        connection,
        &record.tenant_id,
        &record.space_id,
        record.parent_node_id.as_deref(),
        &record.node_name,
    )
    .await?
    .ok_or_else(|| {
        DriveServiceError::Internal(format!(
            "ensured workspace node is not readable: {}",
            record.node_name
        ))
    })
}

async fn find_workspace_child_node_in_transaction(
    connection: &mut PgConnection,
    tenant_id: &str,
    space_id: &str,
    parent_node_id: Option<&str>,
    node_name: &str,
) -> Result<Option<DriveWorkspaceNodeRecord>, DriveServiceError> {
    let row = sqlx::query(
        "SELECT
            n.id,
            n.parent_node_id,
            n.node_type,
            n.node_name,
            CAST(n.updated_at AS TEXT) AS updated_at,
            o.content_type,
            o.content_length,
            (
                SELECT COUNT(1)
                FROM dr_drive_node child
                WHERE child.tenant_id=n.tenant_id
                  AND child.space_id=n.space_id
                  AND child.parent_node_id=n.id
                  AND child.lifecycle_status='active'
            ) AS children_count
         FROM dr_drive_node n
         LEFT JOIN dr_drive_storage_object o
           ON o.tenant_id=n.tenant_id
          AND o.node_id=n.id
          AND o.lifecycle_status='active'
          AND o.version_no=(
              SELECT MAX(o2.version_no)
              FROM dr_drive_storage_object o2
              WHERE o2.tenant_id=n.tenant_id
                AND o2.node_id=n.id
                AND o2.lifecycle_status='active'
          )
         WHERE n.tenant_id=$1
           AND n.space_id=$2
           AND n.node_name=$3
           AND n.lifecycle_status='active'
           AND (
               (n.parent_node_id IS NULL AND $4 IS NULL)
               OR n.parent_node_id=$4
           )
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(node_name)
    .bind(parent_node_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("find workspace child node failed: {error}"))
    })?;

    row.map(|row| map_workspace_node_row(&row)).transpose()
}

fn map_workspace_node_row(
    row: &sqlx::postgres::PgRow,
) -> Result<DriveWorkspaceNodeRecord, DriveServiceError> {
    Ok(DriveWorkspaceNodeRecord {
        id: row.get("id"),
        parent_node_id: row.get("parent_node_id"),
        node_type: row.get("node_type"),
        node_name: row.get("node_name"),
        updated_at: row.get("updated_at"),
        content_type: row.get("content_type"),
        content_length: row.get("content_length"),
        children_count: row
            .try_get::<i64, _>("children_count")
            .or_else(|_| row.try_get::<i32, _>("children_count").map(i64::from))
            .map_err(|error| {
                DriveServiceError::Internal(format!(
                    "decode workspace children_count failed: {error}"
                ))
            })?,
    })
}
