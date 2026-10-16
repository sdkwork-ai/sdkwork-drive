use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateSandboxGrantRequest {
    /// Supported explicit grant subjects. Workspace and role grants remain unavailable until an authoritative membership resolver exists.
    #[serde(rename = "granteeType")]
    pub grantee_type: String,

    #[serde(rename = "granteeId")]
    pub grantee_id: String,

    #[serde(rename = "accessLevel")]
    pub access_level: String,
}
