use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateDownloadPackageRequest {
    #[serde(rename = "nodeIds")]
    pub node_ids: Vec<String>,

    #[serde(rename = "packageName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,

    #[serde(rename = "requestedTtlSeconds")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_ttl_seconds: Option<i64>,
}
