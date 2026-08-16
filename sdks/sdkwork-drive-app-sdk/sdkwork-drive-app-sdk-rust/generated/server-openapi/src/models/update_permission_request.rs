use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdatePermissionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}
