use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EnsureWebsiteRootEventDeliveryRequest {
    pub address: String,

    #[serde(rename = "verificationToken")]
    pub verification_token: String,

    #[serde(rename = "expirationEpochMs")]
    pub expiration_epoch_ms: String,
}
