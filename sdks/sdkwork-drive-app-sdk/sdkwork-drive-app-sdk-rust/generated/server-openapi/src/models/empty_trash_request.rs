use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EmptyTrashRequest {
    #[serde(rename = "spaceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
}
