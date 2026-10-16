use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SetStorageProviderKindEnabledRequest {
    /// Enable or disable the provider kind.
    pub enabled: bool,
}
