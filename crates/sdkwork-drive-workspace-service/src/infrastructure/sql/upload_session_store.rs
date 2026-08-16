use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;

use crate::domain::upload::{DriveUploadSession, DriveUploadSessionState};
use crate::infrastructure::sql::sql_error::is_unique_constraint_violation;
use crate::ports::upload_session_store::{DriveUploadSessionStore, NewDriveUploadSession};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlUploadSessionStore {
    pool: PgPool,
}

impl SqlUploadSessionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveUploadSessionStore for SqlUploadSessionStore {
    async fn find_by_idempotency(
        &self,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<DriveUploadSession>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT id, tenant_id, space_id, node_id, bucket, object_key,
                    idempotency_key, storage_provider_id, storage_upload_id, state,
                    expires_at_epoch_ms, version
             FROM dr_drive_upload_session
             WHERE tenant_id=$1
               AND space_id=$2
               AND node_id=$3
               AND idempotency_key=$4
             LIMIT 1",
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(node_id)
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "query dr_drive_upload_session by idempotency failed: {error}"
            ))
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        let state_raw: String = row.get("state");
        let state = DriveUploadSessionState::try_from_str(&state_raw).ok_or_else(|| {
            DriveServiceError::Internal(format!("unknown upload session state: {state_raw}"))
        })?;

        Ok(Some(DriveUploadSession {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            space_id: row.get("space_id"),
            node_id: row.get("node_id"),
            bucket: row.get("bucket"),
            object_key: row.get("object_key"),
            idempotency_key: row.get("idempotency_key"),
            storage_provider_id: row.get("storage_provider_id"),
            storage_upload_id: row.get("storage_upload_id"),
            state,
            expires_at_epoch_ms: row.get("expires_at_epoch_ms"),
            version: row.get("version"),
        }))
    }

    async fn insert_upload_session(
        &self,
        new_session: &NewDriveUploadSession,
    ) -> Result<DriveUploadSession, DriveServiceError> {
        let result = sqlx::query(
            "INSERT INTO dr_drive_upload_session (
                id, tenant_id, space_id, node_id, bucket, object_key,
                idempotency_key, storage_provider_id, storage_upload_id, state,
                expires_at_epoch_ms, version, created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 1, $12, $13)",
        )
        .bind(&new_session.id)
        .bind(&new_session.tenant_id)
        .bind(&new_session.space_id)
        .bind(&new_session.node_id)
        .bind(&new_session.bucket)
        .bind(&new_session.object_key)
        .bind(&new_session.idempotency_key)
        .bind(&new_session.storage_provider_id)
        .bind(&new_session.storage_upload_id)
        .bind(&new_session.state)
        .bind(new_session.expires_at_epoch_ms)
        .bind(&new_session.created_by)
        .bind(&new_session.updated_by)
        .execute(&self.pool)
        .await;

        if let Err(error) = result {
            let message = error.to_string();
            if is_unique_constraint_violation(&message) {
                return Err(DriveServiceError::Conflict(
                    "upload session idempotency key already exists".to_string(),
                ));
            }
            return Err(DriveServiceError::Internal(format!(
                "insert dr_drive_upload_session failed: {message}"
            )));
        }

        let row = sqlx::query(
            "SELECT id, tenant_id, space_id, node_id, bucket, object_key,
                    idempotency_key, storage_provider_id, storage_upload_id, state,
                    expires_at_epoch_ms, version
             FROM dr_drive_upload_session
             WHERE id=$1",
        )
        .bind(&new_session.id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "read inserted dr_drive_upload_session failed: {error}"
            ))
        })?;

        let state_raw: String = row.get("state");
        let state = DriveUploadSessionState::try_from_str(&state_raw).ok_or_else(|| {
            DriveServiceError::Internal(format!("unknown upload session state: {state_raw}"))
        })?;

        Ok(DriveUploadSession {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            space_id: row.get("space_id"),
            node_id: row.get("node_id"),
            bucket: row.get("bucket"),
            object_key: row.get("object_key"),
            idempotency_key: row.get("idempotency_key"),
            storage_provider_id: row.get("storage_provider_id"),
            storage_upload_id: row.get("storage_upload_id"),
            state,
            expires_at_epoch_ms: row.get("expires_at_epoch_ms"),
            version: row.get("version"),
        })
    }
}
