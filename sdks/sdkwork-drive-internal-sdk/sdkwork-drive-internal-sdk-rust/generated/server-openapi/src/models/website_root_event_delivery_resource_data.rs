use serde::{Deserialize, Serialize};

use crate::models::WebsiteRootEventDelivery;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteRootEventDeliveryResourceData {
    pub item: WebsiteRootEventDelivery,
}
