use async_trait::async_trait;

use crate::domain::upload::DriveUploadSession;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct NewDriveUploadSession {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub bucket: String,
    pub object_key: String,
    pub idempotency_key: String,
    pub storage_provider_id: String,
    pub storage_upload_id: String,
    pub state: String,
    pub expires_at_epoch_ms: i64,
    pub created_by: String,
    pub updated_by: String,
}

#[async_trait]
pub trait DriveUploadSessionStore: Send + Sync {
    async fn find_by_idempotency(
        &self,
        tenant_id: &str,
        space_id: &str,
        node_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<DriveUploadSession>, DriveServiceError>;

    async fn insert_upload_session(
        &self,
        new_session: &NewDriveUploadSession,
    ) -> Result<DriveUploadSession, DriveServiceError>;
}
