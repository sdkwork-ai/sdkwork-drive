use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateRootScopeSubscriptionRequest {
    pub space_id: String,
    pub knowledge_base_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnsureRootScopeEventDeliveryRequest {
    pub address: String,
    pub verification_token: String,
    pub expiration_epoch_ms: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnsureWebsiteRootEventDeliveryRequest {
    pub address: String,
    pub verification_token: String,
    pub expiration_epoch_ms: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveDriveResourceRequest {
    pub scope_type: String,
    pub scope_uuid: String,
    pub relative_path: String,
    pub pinned_generation: Option<String>,
    pub pinned_node_version_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetrieveDriveResourceContentQuery {
    pub scope_type: String,
    pub scope_uuid: String,
    pub relative_path: String,
    pub pinned_generation: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootScopeSubscriptionResponse {
    pub uuid: String,
    pub space_id: String,
    pub consumer_kind: String,
    pub consumer_resource_id: String,
    pub root_node_id: String,
    pub scope_status: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsiteRootResponse {
    pub uuid: String,
    pub space_id: String,
    pub source_root_mode: String,
    pub content_mode: String,
    pub active_generation: String,
    pub root_status: String,
    pub capabilities: Vec<String>,
    pub version: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootScopeEventDeliveryResponse {
    pub channel_id: String,
    pub subscription_uuid: String,
    pub address: String,
    pub expiration_epoch_ms: String,
    pub lifecycle_status: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsiteRootEventDeliveryResponse {
    pub channel_id: String,
    pub website_root_uuid: String,
    pub address: String,
    pub expiration_epoch_ms: String,
    pub lifecycle_status: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveResourceResolutionResponse {
    pub scope_type: String,
    pub scope_uuid: String,
    pub scope_generation: String,
    pub normalized_relative_path: String,
    pub resource_type: String,
    pub node_id: String,
    pub logical_node_version_id: String,
    pub version_no: String,
    pub checksum_sha256_hex: String,
    pub etag: String,
    pub content_type: String,
    pub content_length: String,
    pub last_modified: String,
    pub scope_status: String,
    pub node_status: String,
    pub eligibility: String,
}
