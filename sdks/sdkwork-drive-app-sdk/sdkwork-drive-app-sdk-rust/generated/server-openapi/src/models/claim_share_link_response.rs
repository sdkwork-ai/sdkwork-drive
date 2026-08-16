use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClaimShareLinkResponse {
    #[serde(rename = "shareLinkId")]
    pub share_link_id: String,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    pub role: String,

    #[serde(rename = "permissionId")]
    pub permission_id: String,

    #[serde(rename = "alreadyClaimed")]
    pub already_claimed: bool,
}
