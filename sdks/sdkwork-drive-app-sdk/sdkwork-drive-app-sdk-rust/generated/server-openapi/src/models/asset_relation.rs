use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AssetRelation {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "assetId")]
    pub asset_id: String,

    #[serde(rename = "relatedAssetId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_asset_id: Option<String>,

    #[serde(rename = "relationType")]
    pub relation_type: String,

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
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,
}
