use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CheckFavoriteNodesRequest {
    #[serde(rename = "nodeIds")]
    pub node_ids: Vec<String>,
}
