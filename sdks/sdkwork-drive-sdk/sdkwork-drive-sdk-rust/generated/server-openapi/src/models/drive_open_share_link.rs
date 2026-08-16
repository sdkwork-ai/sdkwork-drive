use serde::{Deserialize, Serialize};

use crate::models::{OpenNode};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveOpenShareLink {
    pub id: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    pub role: String,

    #[serde(rename = "expiresAtEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_epoch_ms: Option<i64>,

    #[serde(rename = "downloadLimit")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_limit: Option<i64>,

    #[serde(rename = "downloadCount")]
    pub download_count: i64,

    #[serde(rename = "accessCodeRequired")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_code_required: Option<bool>,

    pub node: OpenNode,
}
