use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateAssetCollectionItemRequest {
    #[serde(rename = "assetId")]
    pub asset_id: String,

    #[serde(rename = "sortOrder")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
}
