use async_trait::async_trait;

use crate::domain::uploader::{DriveUploadItem, DriveUploadPart};
use crate::DriveServiceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveUploaderSpaceRecord {
    pub id: String,
    pub tenant_id: String,
    pub owner_subject_type: String,
    pub owner_subject_id: String,
    pub space_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveUploaderNodeRecord {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub parent_node_id: Option<String>,
    pub node_type: String,
}

#[derive(Debug, Clone)]
pub struct NewDriveUploadItem {
    pub id: String,
    pub task_id: String,
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub user_id: Option<String>,
    pub actor_type: String,
    pub actor_id: String,
    pub app_id: String,
    pub app_resource_type: String,
    pub app_resource_id: String,
    pub scene: Option<String>,
    pub source: Option<String>,
    pub upload_profile_code: String,
    pub file_fingerprint: String,
    pub space_id: String,
    pub node_id: String,
    pub upload_session_id: Option<String>,
    pub storage_provider_id: Option<String>,
    pub storage_upload_id: Option<String>,
    pub original_file_name: String,
    pub file_extension: Option<String>,
    pub content_type: String,
    pub content_type_group: String,
    pub detected_content_type: Option<String>,
    pub content_length: i64,
    pub checksum_sha256_hex: Option<String>,
    pub chunk_size_bytes: i64,
    pub total_parts: i64,
    pub status: String,
    pub retention_mode: String,
    pub retention_expires_at_epoch_ms: Option<i64>,
    pub cleanup_action: Option<String>,
    pub hard_delete_after_epoch_ms: Option<i64>,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Debug, Clone)]
pub struct NewDriveUploadPart {
    pub id: String,
    pub tenant_id: String,
    pub upload_item_id: String,
    pub upload_session_id: String,
    pub part_no: i64,
    pub offset_bytes: i64,
    pub size_bytes: i64,
    pub etag: String,
    pub checksum_sha256_hex: Option<String>,
    pub uploaded_at_epoch_ms: i64,
}

#[derive(Debug, Clone)]
pub struct CompleteDriveStoredUpload {
    pub tenant_id: String,
    pub upload_item_id: String,
    pub upload_session_id: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub uploaded_parts_count: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct NewDriveUploaderSpace {
    pub id: String,
    pub tenant_id: String,
    pub owner_subject_type: String,
    pub owner_subject_id: String,
    pub space_type: String,
    pub display_name: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct NewDriveUploaderNode {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub parent_node_id: Option<String>,
    pub node_name: String,
    pub scene: Option<String>,
    pub source: Option<String>,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct NewDriveUploaderSession {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub operator_id: String,
    pub expires_at_epoch_ms: i64,
}

#[async_trait]
pub trait DriveUploaderStore: Send + Sync {
    async fn find_upload_space(
        &self,
        tenant_id: &str,
        owner_subject_type: &str,
        owner_subject_id: &str,
        space_type: &str,
    ) -> Result<Option<String>, DriveServiceError>;

    async fn find_active_space(
        &self,
        tenant_id: &str,
        space_id: &str,
    ) -> Result<Option<DriveUploaderSpaceRecord>, DriveServiceError>;

    async fn find_active_node(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<Option<DriveUploaderNodeRecord>, DriveServiceError>;

    async fn has_writer_permission(
        &self,
        tenant_id: &str,
        node_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<bool, DriveServiceError>;

    async fn has_writer_share_token(
        &self,
        tenant_id: &str,
        node_id: &str,
        token_hash: &str,
        now_epoch_ms: i64,
    ) -> Result<bool, DriveServiceError>;

    async fn insert_upload_space(
        &self,
        space: &NewDriveUploaderSpace,
    ) -> Result<String, DriveServiceError>;

    async fn live_node_name_exists_in_parent(
        &self,
        tenant_id: &str,
        space_id: &str,
        parent_node_id: Option<&str>,
        node_name: &str,
    ) -> Result<bool, DriveServiceError>;

    async fn insert_upload_node(
        &self,
        node: &NewDriveUploaderNode,
    ) -> Result<String, DriveServiceError>;

    async fn insert_upload_session(
        &self,
        session: &NewDriveUploaderSession,
    ) -> Result<String, DriveServiceError>;

    async fn find_default_storage_provider(
        &self,
        tenant_id: &str,
    ) -> Result<Option<(String, String)>, DriveServiceError>;

    async fn insert_upload_item(
        &self,
        item: &NewDriveUploadItem,
    ) -> Result<DriveUploadItem, DriveServiceError>;

    async fn find_upload_item_by_task(
        &self,
        tenant_id: &str,
        task_id: &str,
    ) -> Result<Option<DriveUploadItem>, DriveServiceError>;

    async fn record_uploaded_part(
        &self,
        part: &NewDriveUploadPart,
    ) -> Result<DriveUploadPart, DriveServiceError>;

    async fn complete_stored_upload(
        &self,
        completion: &CompleteDriveStoredUpload,
    ) -> Result<DriveUploadItem, DriveServiceError>;

    async fn quarantine_blocked_upload_content(
        &self,
        tenant_id: &str,
        upload_item_id: &str,
        operator_id: &str,
    ) -> Result<(), DriveServiceError>;
}
