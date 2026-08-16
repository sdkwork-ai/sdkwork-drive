use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::PgPool;
use sqlx::Row;

use crate::domain::storage_provider_kind::DriveStorageProviderKindRegistry;
use crate::ports::storage_provider_kind_store::{
    DriveStorageProviderKindStore, NewDriveStorageProviderKind, StorageProviderKindConfigCount,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlStorageProviderKindStore {
    pool: PgPool,
}

impl SqlStorageProviderKindStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveStorageProviderKindStore for SqlStorageProviderKindStore {
    async fn list_storage_provider_kinds(
        &self,
    ) -> Result<Vec<DriveStorageProviderKindRegistry>, DriveServiceError> {
        let rows = sqlx::query(
            "SELECT provider_kind, display_name, enabled, sort_order, version
             FROM dr_drive_storage_provider_kind
             ORDER BY sort_order ASC, provider_kind ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "list dr_drive_storage_provider_kind failed: {error}"
            ))
        })?;
        rows.iter().map(map_row_to_kind).collect()
    }

    async fn find_storage_provider_kind(
        &self,
        provider_kind: &str,
    ) -> Result<Option<DriveStorageProviderKindRegistry>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT provider_kind, display_name, enabled, sort_order, version
             FROM dr_drive_storage_provider_kind
             WHERE provider_kind=$1",
        )
        .bind(provider_kind)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "find dr_drive_storage_provider_kind failed: {error}"
            ))
        })?;
        let Some(row) = row else {
            return Ok(None);
        };
        map_row_to_kind(&row).map(Some)
    }

    async fn initialize_storage_provider_kinds(
        &self,
        kinds: &[NewDriveStorageProviderKind],
    ) -> Result<Vec<DriveStorageProviderKindRegistry>, DriveServiceError> {
        for kind in kinds {
            let result = sqlx::query(
                "INSERT INTO dr_drive_storage_provider_kind
                    (provider_kind, display_name, enabled, sort_order, version)
                 VALUES ($1, $2, TRUE, $3, 1)
                 ON CONFLICT (provider_kind) DO NOTHING",
            )
            .bind(&kind.provider_kind)
            .bind(&kind.display_name)
            .bind(kind.sort_order)
            .execute(&self.pool)
            .await;
            if let Err(error) = result {
                return Err(DriveServiceError::Internal(format!(
                    "initialize dr_drive_storage_provider_kind failed: {error}"
                )));
            }
        }
        self.list_storage_provider_kinds().await
    }

    async fn set_storage_provider_kind_enabled(
        &self,
        provider_kind: &str,
        enabled: bool,
    ) -> Result<DriveStorageProviderKindRegistry, DriveServiceError> {
        let updated = sqlx::query(
            "UPDATE dr_drive_storage_provider_kind
             SET enabled=$2,
                 version=version + 1,
                 updated_at=CURRENT_TIMESTAMP
             WHERE provider_kind=$1",
        )
        .bind(provider_kind)
        .bind(enabled)
        .execute(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "update dr_drive_storage_provider_kind failed: {error}"
            ))
        })?;
        if updated.rows_affected() == 0 {
            return Err(DriveServiceError::NotFound(
                "storage provider kind not found".to_string(),
            ));
        }
        let row = sqlx::query(
            "SELECT provider_kind, display_name, enabled, sort_order, version
             FROM dr_drive_storage_provider_kind
             WHERE provider_kind=$1",
        )
        .bind(provider_kind)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "read updated dr_drive_storage_provider_kind failed: {error}"
            ))
        })?;
        map_row_to_kind(&row)
    }

    async fn count_storage_provider_configs_by_kind(
        &self,
    ) -> Result<Vec<StorageProviderKindConfigCount>, DriveServiceError> {
        let rows = sqlx::query(
            "SELECT provider_kind, COUNT(1) AS config_count
             FROM dr_drive_storage_provider
             WHERE status != 'deleted'
             GROUP BY provider_kind",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!(
                "count dr_drive_storage_provider by kind failed: {error}"
            ))
        })?;
        rows.iter()
            .map(|row| {
                let provider_kind: String = row.get("provider_kind");
                let config_count: i64 = row.get("config_count");
                Ok(StorageProviderKindConfigCount {
                    provider_kind,
                    config_count,
                })
            })
            .collect()
    }
}

fn map_row_to_kind(row: &PgRow) -> Result<DriveStorageProviderKindRegistry, DriveServiceError> {
    let provider_kind: String = row.get("provider_kind");
    if provider_kind.trim().is_empty() {
        return Err(DriveServiceError::Internal(
            "storage provider kind key is empty".to_string(),
        ));
    }
    Ok(DriveStorageProviderKindRegistry {
        provider_kind,
        display_name: row.get("display_name"),
        enabled: row.get("enabled"),
        sort_order: row.get("sort_order"),
        version: row.get("version"),
    })
}
