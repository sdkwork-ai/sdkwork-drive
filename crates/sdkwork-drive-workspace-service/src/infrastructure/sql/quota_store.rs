use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;

use crate::domain::quota::{DriveQuotaSummary, DriveTenantQuotaPolicy};
use crate::ports::quota_store::DriveQuotaStore;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlQuotaStore {
    pool: PgPool,
}

impl SqlQuotaStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveQuotaStore for SqlQuotaStore {
    async fn summarize_tenant_quota(
        &self,
        tenant_id: &str,
    ) -> Result<DriveQuotaSummary, DriveServiceError> {
        let row = sqlx::query(
            "SELECT
                CAST(COALESCE(SUM(content_length), 0) AS BIGINT) AS total_bytes,
                CAST(COUNT(1) AS BIGINT) AS object_count
             FROM dr_drive_storage_object
             WHERE tenant_id=$1
               AND lifecycle_status='active'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "summarize dr_drive_storage_object quota failed: {error}"
            ))
        })?;

        Ok(DriveQuotaSummary {
            tenant_id: tenant_id.to_string(),
            total_bytes: row.get("total_bytes"),
            object_count: row.get("object_count"),
        })
    }

    async fn get_tenant_quota_policy(
        &self,
        tenant_id: &str,
    ) -> Result<Option<DriveTenantQuotaPolicy>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT tenant_id, max_bytes
             FROM dr_drive_tenant_quota
             WHERE tenant_id=$1",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("load dr_drive_tenant_quota failed: {error}"))
        })?;

        Ok(row.map(|row| DriveTenantQuotaPolicy {
            tenant_id: row.get("tenant_id"),
            max_bytes: row.get("max_bytes"),
        }))
    }

    async fn upsert_tenant_quota_policy(
        &self,
        tenant_id: &str,
        max_bytes: Option<i64>,
        updated_by: &str,
    ) -> Result<DriveTenantQuotaPolicy, DriveServiceError> {
        if let Some(max_bytes) = max_bytes {
            if max_bytes <= 0 {
                return Err(DriveServiceError::Validation(
                    "max_bytes must be greater than 0 when set".to_string(),
                ));
            }
        }

        sqlx::query(
            "INSERT INTO dr_drive_tenant_quota (
                tenant_id, max_bytes, updated_by, updated_at, version
             ) VALUES ($1, $2, $3, CURRENT_TIMESTAMP, 1)
             ON CONFLICT (tenant_id) DO UPDATE SET
                max_bytes=EXCLUDED.max_bytes,
                updated_by=EXCLUDED.updated_by,
                updated_at=CURRENT_TIMESTAMP,
                version=dr_drive_tenant_quota.version + 1",
        )
        .bind(tenant_id)
        .bind(max_bytes)
        .bind(updated_by)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("upsert dr_drive_tenant_quota failed: {error}"))
        })?;

        Ok(DriveTenantQuotaPolicy {
            tenant_id: tenant_id.to_string(),
            max_bytes,
        })
    }

    async fn clear_tenant_quota_policy(&self, tenant_id: &str) -> Result<(), DriveServiceError> {
        sqlx::query("DELETE FROM dr_drive_tenant_quota WHERE tenant_id=$1")
            .bind(tenant_id)
            .execute(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("delete dr_drive_tenant_quota failed: {error}"))
            })?;
        Ok(())
    }
}
