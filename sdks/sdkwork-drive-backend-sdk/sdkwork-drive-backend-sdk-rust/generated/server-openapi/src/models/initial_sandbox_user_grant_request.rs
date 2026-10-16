use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InitialSandboxUserGrantRequest {
    /// When true, creates an explicit grant for the current verified user.
    pub enabled: bool,

    #[serde(rename = "accessLevel")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_level: Option<String>,
}
