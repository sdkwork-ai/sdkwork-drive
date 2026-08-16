use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateSpaceRequest {
    pub id: String,

    #[serde(rename = "ownerSubjectType")]
    pub owner_subject_type: String,

    #[serde(rename = "ownerSubjectId")]
    pub owner_subject_id: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "spaceType")]
    pub space_type: String,

    #[serde(rename = "presentationIcon")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation_icon: Option<String>,

    #[serde(rename = "presentationColor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation_color: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
