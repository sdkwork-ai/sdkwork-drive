use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateQuotaPolicyRequest {
    #[serde(rename = "quotaBytes")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_bytes: Option<i64>,

    #[serde(rename = "clearTenantPolicy")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clear_tenant_policy: Option<bool>,
}
