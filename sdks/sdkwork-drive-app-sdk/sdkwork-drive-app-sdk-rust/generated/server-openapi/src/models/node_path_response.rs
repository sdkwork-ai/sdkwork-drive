use serde::{Deserialize, Serialize};

use crate::models::DriveNode;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NodePathResponse {
    pub items: Vec<DriveNode>,

    #[serde(rename = "pathSegments")]
    pub path_segments: Vec<String>,
}
