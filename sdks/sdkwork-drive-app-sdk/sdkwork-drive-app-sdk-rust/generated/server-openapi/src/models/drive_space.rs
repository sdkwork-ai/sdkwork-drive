use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSpace {
    pub id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

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

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    pub version: i64,

    #[serde(rename = "createdBy")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}
