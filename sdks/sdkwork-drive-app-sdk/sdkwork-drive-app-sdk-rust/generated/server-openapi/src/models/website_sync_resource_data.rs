use serde::{Deserialize, Serialize};

use crate::models::{WebsiteSync};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteSyncResourceData {
    pub item: WebsiteSync,
}
