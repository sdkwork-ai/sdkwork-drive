use serde::{Deserialize, Serialize};

use crate::models::{InitialSandboxUserGrantRequest};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateSandboxVolumeRequest {
    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "providerKind")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_kind: Option<String>,

    /// Admin-only provider root. local_filesystem values must already exist, must be directories, and are canonicalized by the server before persistence.
    #[serde(rename = "providerRootRef")]
    pub provider_root_ref: String,

    /// Volume configuration default only. It never grants implicit tenant, organization, or user access.
    #[serde(rename = "defaultAccess")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_access: Option<String>,

    /// Omitted means enabled=true and accessLevel=full. Set enabled=false to create no initial grant.
    #[serde(rename = "initialUserGrant")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_user_grant: Option<InitialSandboxUserGrantRequest>,
}
