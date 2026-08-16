use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AssetCollection {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "organizationId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    #[serde(rename = "userId")]
    pub user_id: String,

    pub title: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "collectionType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
