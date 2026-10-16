use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSandboxMutationCommandData {
    pub accepted: bool,

    #[serde(rename = "resourceId")]
    pub resource_id: String,

    pub status: String,
}
