use serde::{Deserialize, Serialize};

use crate::models::{DownloadPackageItem};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DownloadPackageResponse {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "packageName")]
    pub package_name: String,

    pub state: String,

    #[serde(rename = "storageProviderId")]
    pub storage_provider_id: String,

    pub bucket: String,

    #[serde(rename = "archiveObjectKey")]
    pub archive_object_key: String,

    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "fileCount")]
    pub file_count: i64,

    #[serde(rename = "totalBytes")]
    pub total_bytes: i64,

    #[serde(rename = "archiveSizeBytes")]
    pub archive_size_bytes: i64,

    #[serde(rename = "expiresAtEpochMs")]
    pub expires_at_epoch_ms: i64,

    #[serde(rename = "downloadUrl")]
    pub download_url: String,

    #[serde(rename = "signedSourceUrl")]
    pub signed_source_url: String,

    pub method: String,

    pub items: Vec<DownloadPackageItem>,
}
