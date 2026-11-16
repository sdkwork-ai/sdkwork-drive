use serde::{Deserialize, Serialize};

use crate::models::{WebsiteSyncActivation};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteSyncActivationResourceData {
    pub item: WebsiteSyncActivation,
}
