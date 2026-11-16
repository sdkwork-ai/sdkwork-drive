use serde::{Deserialize, Serialize};

use crate::models::{RootScopeSubscription};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RootScopeSubscriptionResourceData {
    pub item: RootScopeSubscription,
}
