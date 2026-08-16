use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FavoriteNodeResponse {
    pub favorited: bool,
}
