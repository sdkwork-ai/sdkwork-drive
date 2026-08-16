use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateSpaceRequest {
    #[serde(rename = "displayName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(rename = "presentationIcon")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation_icon: Option<String>,

    #[serde(rename = "presentationColor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation_color: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
