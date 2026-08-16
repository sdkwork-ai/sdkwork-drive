use serde::{Deserialize, Serialize};

/// Create storage provider request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStorageProviderRequest {
    pub id: String,
    pub provider_kind: String,
    pub name: String,
    pub endpoint_url: String,
    pub bucket: String,
    pub operator_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_style: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_side_encryption_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_storage_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_tls: Option<bool>,
}

/// Update storage provider request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStorageProviderRequest {
    pub operator_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_style: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_side_encryption_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_storage_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_tls: Option<bool>,
}

/// Set default storage provider binding request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDefaultStorageProviderBindingRequest {
    pub tenant_id: String,
    pub provider_id: String,
    pub operator_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_root_prefix: Option<String>,
}

/// Rotate credential request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateCredentialRequest {
    pub operator_id: String,
    pub new_credential_ref: String,
}

/// Copy object request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyObjectRequest {
    pub source_object_key: String,
    pub target_object_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_bucket: Option<String>,
    pub operator_id: String,
}
