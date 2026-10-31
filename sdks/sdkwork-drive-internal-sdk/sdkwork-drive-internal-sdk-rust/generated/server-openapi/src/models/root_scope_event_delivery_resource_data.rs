use serde::{Deserialize, Serialize};

use crate::models::RootScopeEventDelivery;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RootScopeEventDeliveryResourceData {
    pub item: RootScopeEventDelivery,
}
