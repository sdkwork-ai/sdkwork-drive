use serde::{Deserialize, Serialize};

/// List response with pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
}

/// Storage provider response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProviderResponse {
    pub id: String,
    pub provider_kind: String,
    pub name: String,
    pub endpoint_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    pub bucket: String,
    pub path_style: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<String>,
    pub credential_configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_side_encryption_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_storage_class: Option<String>,
    pub status: String,
    pub version: i64,
    pub strict_tls: bool,
}

/// Storage provider binding response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProviderBindingResponse {
    pub id: String,
    pub tenant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    pub provider_id: String,
    pub binding_scope: String,
    pub lifecycle_status: String,
    pub version: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_root_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_provider: Option<StorageProviderResponse>,
}

/// Storage provider capabilities response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProviderCapabilitiesResponse {
    pub provider_id: String,
    pub provider_kind: String,
    pub supports_multipart_upload: bool,
    pub supports_presigned_upload_part: bool,
    pub supports_presigned_download: bool,
    pub supports_server_side_encryption: bool,
    pub supports_storage_class: bool,
    pub supports_credential_rotation: bool,
    pub supported_server_side_encryption_modes: Vec<String>,
    pub supported_storage_classes: Vec<String>,
}

/// Test provider response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProviderResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<i64>,
}

/// Bucket info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketInfoResponse {
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// Object info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfoResponse {
    pub key: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_ms: Option<i64>,
}

/// Upload session response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSessionResponse {
    pub id: String,
    pub space_id: String,
    pub node_id: String,
    pub state: String,
    pub expires_at_ms: i64,
    pub created_at_ms: i64,
}

/// Download URL response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadUrlResponse {
    pub download_url: String,
    pub expires_at_epoch_ms: i64,
    pub method: String,
}
