use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SetNodePropertyRequest {
    pub value: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
}
