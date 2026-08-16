#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveSandboxVolume {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub display_name: String,
    pub root_entry_id: String,
    pub provider_kind: String,
    pub lifecycle_status: String,
    pub default_access: String,
    pub effective_access: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveSandboxGrant {
    pub sandbox_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub access_level: String,
    pub lifecycle_status: String,
}

/// Server-only provider binding returned only after tenant and grant predicates succeed.
/// The private root reference must never be serialized into an HTTP or SDK DTO.
#[derive(Clone, PartialEq, Eq)]
pub struct AuthorizedSandboxMount {
    sandbox_id: String,
    root_entry_id: String,
    provider_kind: String,
    private_root_ref: String,
    lifecycle_status: String,
    effective_access: String,
    revision: i64,
}

impl std::fmt::Debug for AuthorizedSandboxMount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AuthorizedSandboxMount")
            .field("sandbox_id", &self.sandbox_id)
            .field("root_entry_id", &self.root_entry_id)
            .field("provider_kind", &self.provider_kind)
            .field("private_root_ref", &"[REDACTED]")
            .field("lifecycle_status", &self.lifecycle_status)
            .field("effective_access", &self.effective_access)
            .field("revision", &self.revision)
            .finish()
    }
}

impl AuthorizedSandboxMount {
    pub(crate) fn new(
        sandbox_id: String,
        root_entry_id: String,
        provider_kind: String,
        private_root_ref: String,
        lifecycle_status: String,
        effective_access: String,
        revision: i64,
    ) -> Self {
        Self {
            sandbox_id,
            root_entry_id,
            provider_kind,
            private_root_ref,
            lifecycle_status,
            effective_access,
            revision,
        }
    }

    pub fn sandbox_id(&self) -> &str {
        &self.sandbox_id
    }

    pub fn root_entry_id(&self) -> &str {
        &self.root_entry_id
    }

    pub fn provider_kind(&self) -> &str {
        &self.provider_kind
    }

    pub fn private_root_ref(&self) -> &str {
        &self.private_root_ref
    }

    pub fn lifecycle_status(&self) -> &str {
        &self.lifecycle_status
    }

    pub fn effective_access(&self) -> &str {
        &self.effective_access
    }

    pub fn revision(&self) -> i64 {
        self.revision
    }
}

#[cfg(test)]
mod tests {
    use super::AuthorizedSandboxMount;

    #[test]
    fn authorized_mount_debug_output_redacts_private_root() {
        let mount = AuthorizedSandboxMount::new(
            "sandbox-1".to_string(),
            "root-1".to_string(),
            "local_filesystem".to_string(),
            "C:\\sensitive\\deployment-root".to_string(),
            "active".to_string(),
            "full".to_string(),
            1,
        );

        let debug = format!("{mount:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("sensitive"));
    }
}
