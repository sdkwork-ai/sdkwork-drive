use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxAdminProviderKind {
    LocalFilesystem,
}

impl SandboxAdminProviderKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "local_filesystem" => Some(Self::LocalFilesystem),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalFilesystem => "local_filesystem",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxAdminLifecycleStatus {
    Active,
    ReadOnly,
    Disabled,
}

impl SandboxAdminLifecycleStatus {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "read_only" => Some(Self::ReadOnly),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::ReadOnly => "read_only",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxAdminAccessLevel {
    Full,
    ReadOnly,
}

impl SandboxAdminAccessLevel {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "full" => Some(Self::Full),
            "read_only" => Some(Self::ReadOnly),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::ReadOnly => "read_only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxAdminGrantSubjectType {
    User,
    Organization,
}

impl SandboxAdminGrantSubjectType {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "user" => Some(Self::User),
            "organization" => Some(Self::Organization),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Organization => "organization",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SandboxAdminVolume {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub display_name: String,
    pub root_entry_id: String,
    pub provider_kind: String,
    pub provider_root_ref: String,
    pub lifecycle_status: String,
    pub default_access: String,
    pub version: i64,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl fmt::Debug for SandboxAdminVolume {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SandboxAdminVolume")
            .field("id", &self.id)
            .field("tenant_id", &self.tenant_id)
            .field("organization_id", &self.organization_id)
            .field("display_name", &self.display_name)
            .field("root_entry_id", &self.root_entry_id)
            .field("provider_kind", &self.provider_kind)
            .field("provider_root_ref", &"[REDACTED]")
            .field("lifecycle_status", &self.lifecycle_status)
            .field("default_access", &self.default_access)
            .field("version", &self.version)
            .field("created_by", &self.created_by)
            .field("updated_by", &self.updated_by)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxAdminGrant {
    pub id: String,
    pub sandbox_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub access_level: String,
    pub granted_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxAdminPage<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
}

#[cfg(test)]
mod tests {
    use super::SandboxAdminVolume;

    #[test]
    fn volume_debug_output_never_exposes_provider_root() {
        let volume = SandboxAdminVolume {
            id: "sandbox-1".to_string(),
            tenant_id: "tenant-1".to_string(),
            organization_id: "organization-1".to_string(),
            display_name: "Workspace".to_string(),
            root_entry_id: "root-1".to_string(),
            provider_kind: "local_filesystem".to_string(),
            provider_root_ref: "C:\\deployment\\private".to_string(),
            lifecycle_status: "active".to_string(),
            default_access: "full".to_string(),
            version: 1,
            created_by: "user-1".to_string(),
            updated_by: "user-1".to_string(),
            created_at: "2026-07-16T00:00:00Z".to_string(),
            updated_at: "2026-07-16T00:00:00Z".to_string(),
        };

        let debug = format!("{volume:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("deployment"));
    }
}
