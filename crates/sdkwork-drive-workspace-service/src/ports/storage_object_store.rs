use async_trait::async_trait;
use std::collections::BTreeMap;

use crate::DriveServiceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveStorageObject {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub version_no: i64,
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub lifecycle_status: String,
}

#[derive(Debug, Clone)]
pub struct DownloadSignCommand {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub expires_at_epoch_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedDownloadPayload {
    pub method: String,
    pub raw_url: String,
    pub headers: BTreeMap<String, String>,
    pub expires_at_epoch_ms: i64,
}

#[async_trait]
pub trait DriveStorageObjectStore: Send + Sync {
    async fn find_latest_active_by_node(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<Option<DriveStorageObject>, DriveServiceError>;
}

#[async_trait]
pub trait DriveDownloadSigner: Send + Sync {
    async fn sign_download(
        &self,
        command: DownloadSignCommand,
    ) -> Result<SignedDownloadPayload, DriveServiceError>;
}
