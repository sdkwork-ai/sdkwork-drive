use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct InitialSandboxUserGrantRequest {
    pub(crate) enabled: bool,
    pub(crate) access_level: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateSandboxVolumeRequest {
    pub(crate) display_name: String,
    pub(crate) provider_kind: Option<String>,
    pub(crate) provider_root_ref: String,
    pub(crate) default_access: Option<String>,
    pub(crate) initial_user_grant: Option<InitialSandboxUserGrantRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateSandboxVolumeRequest {
    pub(crate) display_name: Option<String>,
    pub(crate) provider_root_ref: Option<String>,
    pub(crate) lifecycle_status: Option<String>,
    pub(crate) default_access: Option<String>,
    pub(crate) expected_version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListSandboxVolumesQuery {
    pub(crate) lifecycle_status: Option<String>,
    pub(crate) provider_kind: Option<String>,
    pub(crate) page: Option<i64>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SandboxVolumeResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) organization_id: String,
    pub(crate) display_name: String,
    pub(crate) root_entry_id: String,
    pub(crate) provider_kind: String,
    pub(crate) provider_root_ref: String,
    pub(crate) lifecycle_status: String,
    pub(crate) default_access: String,
    pub(crate) version: i64,
    pub(crate) created_by: String,
    pub(crate) updated_by: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListSandboxGrantsQuery {
    pub(crate) page: Option<i64>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateSandboxGrantRequest {
    pub(crate) grantee_type: String,
    pub(crate) grantee_id: String,
    pub(crate) access_level: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateSandboxGrantRequest {
    pub(crate) access_level: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SandboxGrantResponse {
    pub(crate) id: String,
    pub(crate) sandbox_id: String,
    pub(crate) subject_type: String,
    pub(crate) subject_id: String,
    pub(crate) access_level: String,
    pub(crate) granted_by: String,
    pub(crate) created_at: String,
}
