use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StopWatchChannelRequest {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, serde_json::Value>,
}
