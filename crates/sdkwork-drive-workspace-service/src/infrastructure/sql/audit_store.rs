use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::PgPool;
use sqlx::Row;

use crate::domain::audit::DriveAuditEvent;
use crate::infrastructure::sql::runtime_id::next_drive_runtime_id;
use crate::infrastructure::sql::sql_error::normalize_timestamp_text;
use crate::ports::audit_store::{
    AuditEventPage, DriveAuditStore, ListAuditEventsQuery, NewDriveAuditEvent,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlAuditStore {
    pool: PgPool,
}

impl SqlAuditStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveAuditStore for SqlAuditStore {
    async fn append_event(
        &self,
        new_event: &NewDriveAuditEvent,
    ) -> Result<DriveAuditEvent, DriveServiceError> {
        let inserted_id = next_drive_runtime_id("drive audit event")?;
        sqlx::query(
            "INSERT INTO dr_drive_audit_event (
                id, tenant_id, action, resource_type, resource_id, operator_id, request_id, trace_id
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(inserted_id)
        .bind(&new_event.tenant_id)
        .bind(&new_event.action)
        .bind(&new_event.resource_type)
        .bind(&new_event.resource_id)
        .bind(&new_event.operator_id)
        .bind(&new_event.request_id)
        .bind(&new_event.trace_id)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("insert dr_drive_audit_event failed: {error}"))
        })?;

        let row = sqlx::query(
            "SELECT id, tenant_id, action, resource_type, resource_id, operator_id, request_id, trace_id,
                    CAST(created_at AS TEXT) AS created_at
             FROM dr_drive_audit_event
             WHERE id=$1",
        )
        .bind(inserted_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("read inserted dr_drive_audit_event failed: {error}"))
        })?;
        map_row_to_audit_event(&row)
    }

    async fn list_events(
        &self,
        query: &ListAuditEventsQuery,
    ) -> Result<AuditEventPage, DriveServiceError> {
        let offset = i64::from((query.page - 1) * query.page_size);
        let limit = i64::from(query.page_size);

        let rows = sqlx::query(
            "SELECT id, tenant_id, action, resource_type, resource_id, operator_id, request_id, trace_id,
                    CAST(created_at AS TEXT) AS created_at
             FROM dr_drive_audit_event
             WHERE ($1 IS NULL OR tenant_id = $1)
               AND ($2 IS NULL OR action = $2)
               AND ($3 IS NULL OR resource_type = $3)
               AND ($4 IS NULL OR resource_id = $4)
               AND ($5 IS NULL OR request_id = $5)
               AND ($6 IS NULL OR trace_id = $6)
             ORDER BY id DESC
             LIMIT $7 OFFSET $8",
        )
            .bind(query.tenant_id.as_deref())
            .bind(query.action.as_deref())
            .bind(query.resource_type.as_deref())
            .bind(query.resource_id.as_deref())
            .bind(query.request_id.as_deref())
            .bind(query.trace_id.as_deref())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("list dr_drive_audit_event failed: {error}"))
            })?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(1) AS total_count
             FROM dr_drive_audit_event
             WHERE ($1 IS NULL OR tenant_id = $1)
               AND ($2 IS NULL OR action = $2)
               AND ($3 IS NULL OR resource_type = $3)
               AND ($4 IS NULL OR resource_id = $4)
               AND ($5 IS NULL OR request_id = $5)
               AND ($6 IS NULL OR trace_id = $6)",
        )
        .bind(query.tenant_id.as_deref())
        .bind(query.action.as_deref())
        .bind(query.resource_type.as_deref())
        .bind(query.resource_id.as_deref())
        .bind(query.request_id.as_deref())
        .bind(query.trace_id.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("count dr_drive_audit_event failed: {error}"))
        })?;

        let items = rows
            .iter()
            .map(map_row_to_audit_event)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AuditEventPage {
            items,
            page: query.page,
            page_size: query.page_size,
            total,
        })
    }
}

fn map_row_to_audit_event(row: &PgRow) -> Result<DriveAuditEvent, DriveServiceError> {
    let action: String = row.get("action");
    if action.trim().is_empty() {
        return Err(DriveServiceError::Internal(
            "dr_drive_audit_event.action is empty".to_string(),
        ));
    }

    Ok(DriveAuditEvent {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        action,
        resource_type: row.get("resource_type"),
        resource_id: row.get("resource_id"),
        operator_id: row.get("operator_id"),
        request_id: row.get("request_id"),
        trace_id: row.get("trace_id"),
        created_at: normalize_timestamp_text(row.get("created_at")),
    })
}
