use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateOpenDownloadUrlRequest {
    pub(crate) requested_ttl_seconds: Option<u32>,
    #[serde(default)]
    pub(crate) access_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenShareLinkAccessQuery {
    #[serde(default)]
    pub(crate) access_code: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenShareLinkResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) role: String,
    pub(crate) expires_at_epoch_ms: Option<i64>,
    pub(crate) download_limit: Option<i64>,
    pub(crate) download_count: i64,
    pub(crate) access_code_required: bool,
    pub(crate) node: OpenNodeResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenNodeResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) space_id: String,
    pub(crate) node_type: String,
    pub(crate) node_name: String,
    pub(crate) content_type: Option<String>,
    pub(crate) content_length: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenDownloadUrlResponse {
    pub(crate) download_url: String,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) method: String,
}
