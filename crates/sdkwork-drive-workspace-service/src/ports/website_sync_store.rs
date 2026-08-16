use async_trait::async_trait;

use crate::domain::website_root::DriveWebsiteRoot;
use crate::domain::website_sync::{
    DriveWebsiteGeneration, DriveWebsiteManifestSummary, DriveWebsiteSync,
    DriveWebsiteSyncTreeEntry,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct CreateDriveWebsiteSync {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub idempotency_key: String,
    pub expected_root_version: i64,
    pub expected_generation: i64,
    pub manifest_sha256: String,
    pub manifest_file_count: i64,
    pub manifest_total_bytes: i64,
    pub expires_at: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct CreateDriveWebsiteSyncResult {
    pub sync: DriveWebsiteSync,
    pub created: bool,
}

#[derive(Debug, Clone)]
pub struct ValidateDriveWebsiteSync {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub sync_id: String,
    pub expected_sync_version: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DriveWebsiteSyncValidation {
    pub sync: DriveWebsiteSync,
    pub lease_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActivateValidatedWebsiteSync {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub sync_id: String,
    pub expected_sync_version: i64,
    pub lease_token: Option<String>,
    pub observed_manifest: DriveWebsiteManifestSummary,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct MarkDriveWebsiteSyncFailed {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub sync_id: String,
    pub expected_sync_version: i64,
    pub lease_token: String,
    pub error_code: String,
    pub error_summary: String,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DriveWebsiteSyncActivation {
    pub sync: DriveWebsiteSync,
    pub website_root: DriveWebsiteRoot,
}

#[derive(Debug, Clone)]
pub struct AbortDriveWebsiteSync {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub sync_id: String,
    pub expected_sync_version: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct ActivateDriveWebsiteGeneration {
    pub tenant_id: String,
    pub website_root_uuid: String,
    pub target_generation: i64,
    pub expected_root_version: i64,
    pub expected_generation: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct DriveWebsiteGenerationActivation {
    pub source_generation: DriveWebsiteGeneration,
    pub website_root: DriveWebsiteRoot,
}

#[async_trait]
pub trait DriveWebsiteSyncStore: Send + Sync {
    async fn create_or_get(
        &self,
        command: &CreateDriveWebsiteSync,
    ) -> Result<CreateDriveWebsiteSyncResult, DriveServiceError>;

    async fn get(
        &self,
        tenant_id: &str,
        website_root_uuid: &str,
        sync_id: &str,
    ) -> Result<DriveWebsiteSync, DriveServiceError>;

    async fn begin_validation(
        &self,
        command: &ValidateDriveWebsiteSync,
    ) -> Result<DriveWebsiteSyncValidation, DriveServiceError>;

    async fn list_staging_tree(
        &self,
        tenant_id: &str,
        website_root_uuid: &str,
        sync_id: &str,
    ) -> Result<Vec<DriveWebsiteSyncTreeEntry>, DriveServiceError>;

    async fn activate_validated(
        &self,
        command: &ActivateValidatedWebsiteSync,
    ) -> Result<DriveWebsiteSyncActivation, DriveServiceError>;

    async fn mark_failed(
        &self,
        command: &MarkDriveWebsiteSyncFailed,
    ) -> Result<(), DriveServiceError>;

    async fn abort(
        &self,
        command: &AbortDriveWebsiteSync,
    ) -> Result<DriveWebsiteSync, DriveServiceError>;

    async fn activate_generation(
        &self,
        command: &ActivateDriveWebsiteGeneration,
    ) -> Result<DriveWebsiteGenerationActivation, DriveServiceError>;
}
