use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SandboxVolume {
    pub id: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    /// Owning organization resolved from the verified backend session.
    #[serde(rename = "organizationId")]
    pub organization_id: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "rootEntryId")]
    pub root_entry_id: String,

    /// Runtime-backed server sandbox provider. Only local_filesystem is available in this release.
    #[serde(rename = "providerKind")]
    pub provider_kind: String,

    /// Canonical provider root visible only on the protected Drive backend-admin API. It must never appear in app/open API responses, logs, or audit payloads.
    #[serde(rename = "providerRootRef")]
    pub provider_root_ref: String,

    #[serde(rename = "lifecycleStatus")]
    pub lifecycle_status: String,

    #[serde(rename = "defaultAccess")]
    pub default_access: String,

    pub version: String,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "updatedBy")]
    pub updated_by: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
