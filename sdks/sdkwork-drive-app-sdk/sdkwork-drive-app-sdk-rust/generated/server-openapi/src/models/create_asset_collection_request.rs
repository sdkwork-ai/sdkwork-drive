use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateAssetCollectionRequest {
    pub title: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "collectionType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
}
