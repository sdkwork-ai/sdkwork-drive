use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;

use crate::ports::storage_object_store::{DriveStorageObject, DriveStorageObjectStore};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlStorageObjectStore {
    pool: PgPool,
}

impl SqlStorageObjectStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriveStorageObjectStore for SqlStorageObjectStore {
    async fn find_latest_active_by_node(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<Option<DriveStorageObject>, DriveServiceError> {
        let row = sqlx::query(
            "SELECT o.id, o.tenant_id, o.node_id, o.version_no, o.storage_provider_id, o.bucket, o.object_key,
                    o.content_type, o.content_length, o.checksum_sha256_hex, o.lifecycle_status
             FROM dr_drive_storage_object o
             INNER JOIN dr_drive_node n
                ON n.tenant_id=o.tenant_id
               AND n.id=o.node_id
               AND n.lifecycle_status='active'
               AND n.node_type='file'
             INNER JOIN dr_drive_space s
                ON s.tenant_id=o.tenant_id
               AND s.id=n.space_id
               AND s.lifecycle_status='active'
             WHERE o.tenant_id=$1
               AND o.node_id=$2
               AND o.lifecycle_status='active'
             ORDER BY o.version_no DESC
             LIMIT 1",
        )
        .bind(tenant_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("query dr_drive_storage_object failed: {error}"))
        })?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(DriveStorageObject {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            node_id: row.get("node_id"),
            version_no: row.get("version_no"),
            storage_provider_id: row.get("storage_provider_id"),
            bucket: row.get("bucket"),
            object_key: row.get("object_key"),
            content_type: row.get("content_type"),
            content_length: row.get("content_length"),
            checksum_sha256_hex: row.get("checksum_sha256_hex"),
            lifecycle_status: row.get("lifecycle_status"),
        }))
    }
}
