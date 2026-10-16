use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateSandboxVolumeRequest {
    #[serde(rename = "displayName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Admin-only provider root. local_filesystem values are canonicalized server-side.
    #[serde(rename = "providerRootRef")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_root_ref: Option<String>,

    #[serde(rename = "lifecycleStatus")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle_status: Option<String>,

    #[serde(rename = "defaultAccess")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_access: Option<String>,

    #[serde(rename = "expectedVersion")]
    pub expected_version: String,
}
