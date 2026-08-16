use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DownloadPackage {
    pub id: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

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

    #[serde(rename = "errorMessage")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "updatedBy")]
    pub updated_by: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
