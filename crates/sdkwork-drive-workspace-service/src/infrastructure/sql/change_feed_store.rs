use sqlx::postgres::PgRow;
use sqlx::PgPool;
use sqlx::Row;

use crate::domain::change::DriveChangeRecord;
use crate::infrastructure::sql::acl_predicate::reader_inherited_permission_exists_sql;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlChangeFeedStore {
    pool: PgPool,
}

impl SqlChangeFeedStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_changes_for_space_owner(
        &self,
        tenant_id: &str,
        space_id: &str,
        after_sequence: i64,
        limit: i64,
    ) -> Result<Vec<DriveChangeRecord>, DriveServiceError> {
        let rows = sqlx::query(
            "SELECT sequence_no, tenant_id, space_id, node_id, event_type, actor_id,
                    CAST(created_at AS TEXT) AS created_at
             FROM dr_drive_change_log
             WHERE tenant_id=$1 AND space_id=$2 AND sequence_no > $3
             ORDER BY sequence_no ASC
             LIMIT $4",
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(after_sequence)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("list dr_drive_change_log failed: {error}"))
        })?;

        rows.iter().map(map_change_row).collect()
    }

    pub async fn list_changes_for_reader(
        &self,
        tenant_id: &str,
        space_id: &str,
        after_sequence: i64,
        subject_type: &str,
        subject_id: &str,
        limit: i64,
    ) -> Result<Vec<DriveChangeRecord>, DriveServiceError> {
        let reader_acl_predicate = reader_inherited_permission_exists_sql("n", "$4", "$5");
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT cl.sequence_no, cl.tenant_id, cl.space_id, cl.node_id, cl.event_type, cl.actor_id,
                    CAST(cl.created_at AS TEXT) AS created_at
             FROM dr_drive_change_log cl
             LEFT JOIN dr_drive_node n
               ON n.tenant_id = cl.tenant_id
              AND n.id = cl.node_id
             WHERE cl.tenant_id=$1
               AND cl.space_id=$2
               AND cl.sequence_no > $3
               AND (cl.node_id IS NULL OR ({reader_acl_predicate}))
             ORDER BY cl.sequence_no ASC
             LIMIT $6",
        )))
        .bind(tenant_id)
        .bind(space_id)
        .bind(after_sequence)
        .bind(subject_type)
        .bind(subject_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("list dr_drive_change_log failed: {error}"))
        })?;

        rows.iter().map(map_change_row).collect()
    }

    pub async fn query_start_page_token(
        &self,
        tenant_id: &str,
        space_id: Option<&str>,
    ) -> Result<i64, DriveServiceError> {
        if let Some(space_id) = space_id {
            sqlx::query_scalar(
                "SELECT COALESCE(MAX(sequence_no), 0)
                 FROM dr_drive_change_log
                 WHERE tenant_id=$1 AND space_id=$2",
            )
            .bind(tenant_id)
            .bind(space_id)
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query_scalar(
                "SELECT COALESCE(MAX(sequence_no), 0)
                 FROM dr_drive_change_log
                 WHERE tenant_id=$1",
            )
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "compute dr_drive_change_log start page token failed: {error}"
            ))
        })
    }
}

fn map_change_row(row: &PgRow) -> Result<DriveChangeRecord, DriveServiceError> {
    Ok(DriveChangeRecord {
        sequence_no: row.get("sequence_no"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        node_id: row.get("node_id"),
        event_type: row.get("event_type"),
        actor_id: row.get("actor_id"),
        created_at: row.get("created_at"),
    })
}
