use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateSandboxGrantRequest {
    #[serde(rename = "accessLevel")]
    pub access_level: String,
}
