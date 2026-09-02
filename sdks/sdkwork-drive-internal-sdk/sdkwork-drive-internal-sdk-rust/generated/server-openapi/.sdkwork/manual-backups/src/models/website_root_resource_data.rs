use serde::{Deserialize, Serialize};

use crate::models::WebsiteRoot;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteRootResourceData {
    pub item: WebsiteRoot,
}
