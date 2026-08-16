use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateLabelRequest {
    pub id: String,

    #[serde(rename = "labelKey")]
    pub label_key: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
