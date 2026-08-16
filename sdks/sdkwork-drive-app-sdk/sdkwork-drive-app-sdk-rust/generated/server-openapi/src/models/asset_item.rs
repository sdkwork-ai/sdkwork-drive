use serde::{Deserialize, Serialize};

use crate::models::{MediaResource};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AssetItem {
    /// Alias of driveNodeId.
    #[serde(rename = "assetId")]
    pub asset_id: String,

    /// Deprecated alias of assetId.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "organizationId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    #[serde(rename = "userId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    #[serde(rename = "driveSpaceId")]
    pub drive_space_id: String,

    #[serde(rename = "driveNodeId")]
    pub drive_node_id: String,

    #[serde(rename = "driveUri")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drive_uri: Option<String>,

    #[serde(rename = "nodeType")]
    pub node_type: String,

    #[serde(rename = "assetKind")]
    pub asset_kind: String,

    #[serde(rename = "assetType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,

    pub title: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(rename = "sourceType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,

    #[serde(rename = "sourceDomain")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_domain: Option<String>,

    #[serde(rename = "sourceResourceType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_resource_type: Option<String>,

    #[serde(rename = "sourceResourceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_resource_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    #[serde(rename = "resourceSnapshot")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_snapshot: Option<MediaResource>,

    #[serde(rename = "thumbnailDriveNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_drive_node_id: Option<String>,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
