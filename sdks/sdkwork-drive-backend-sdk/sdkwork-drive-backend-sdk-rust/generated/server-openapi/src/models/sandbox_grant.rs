use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SandboxGrant {
    pub id: String,

    #[serde(rename = "sandboxId")]
    pub sandbox_id: String,

    /// Supported explicit grant subjects. Workspace and role grants remain unavailable until an authoritative membership resolver exists.
    #[serde(rename = "subjectType")]
    pub subject_type: String,

    #[serde(rename = "subjectId")]
    pub subject_id: String,

    #[serde(rename = "accessLevel")]
    pub access_level: String,

    #[serde(rename = "grantedBy")]
    pub granted_by: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,
}
