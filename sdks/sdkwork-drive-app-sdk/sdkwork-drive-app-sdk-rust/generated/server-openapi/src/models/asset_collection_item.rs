use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AssetCollectionItem {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "collectionId")]
    pub collection_id: String,

    #[serde(rename = "assetId")]
    pub asset_id: String,

    #[serde(rename = "sortOrder")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
}
