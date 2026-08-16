use serde::{Deserialize, Serialize};

/// Storage provider status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveStorageProviderStatus {
    Active,
    Inactive,
    Error,
}

impl DriveStorageProviderStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Error => "error",
        }
    }
}

/// Storage provider entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveStorageProvider {
    pub id: String,
    pub provider_kind: String,
    pub name: String,
    pub endpoint_url: String,
    pub region: Option<String>,
    pub bucket: String,
    pub path_style: bool,
    pub credential_ref: Option<String>,
    pub status: DriveStorageProviderStatus,
    pub version: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Storage provider binding scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveBindingScope {
    Tenant,
    Space,
}

/// Storage provider binding entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveStorageProviderBinding {
    pub id: String,
    pub tenant_id: String,
    pub space_id: Option<String>,
    pub provider_id: String,
    pub binding_scope: DriveBindingScope,
    pub lifecycle_status: String,
    pub storage_root_prefix: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Storage provider capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveProviderCapabilities {
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
