use crate::context::DriveAppContext;

/// Legacy umbrella scope for backend and admin-storage APIs.
pub const DRIVE_STORAGE_ADMIN_PERMISSION: &str = "drive.storage.admin";
pub const DRIVE_BACKEND_ADMIN_WILDCARD: &str = "drive.*";

/// Granular backend admin scopes enforced per route operation.
pub const DRIVE_AUDIT_ADMIN_PERMISSION: &str = "drive.audit.admin";
pub const DRIVE_MAINTENANCE_ADMIN_PERMISSION: &str = "drive.maintenance.admin";
pub const DRIVE_QUOTA_ADMIN_PERMISSION: &str = "drive.quota.admin";
pub const DRIVE_LABELS_ADMIN_PERMISSION: &str = "drive.labels.admin";
pub const DRIVE_SPACES_ADMIN_PERMISSION: &str = "drive.spaces.admin";
pub const DRIVE_DOWNLOAD_PACKAGES_ADMIN_PERMISSION: &str = "drive.download_packages.admin";
pub const DRIVE_SANDBOXES_ADMIN_PERMISSION: &str = "drive.sandboxes.admin";

const BACKEND_CAPABILITY_PERMISSIONS: &[&str] = &[
    DRIVE_AUDIT_ADMIN_PERMISSION,
    DRIVE_MAINTENANCE_ADMIN_PERMISSION,
    DRIVE_QUOTA_ADMIN_PERMISSION,
    DRIVE_LABELS_ADMIN_PERMISSION,
    DRIVE_SPACES_ADMIN_PERMISSION,
    DRIVE_DOWNLOAD_PACKAGES_ADMIN_PERMISSION,
    DRIVE_SANDBOXES_ADMIN_PERMISSION,
];

pub fn has_drive_admin_privilege(context: &DriveAppContext) -> bool {
    context.permission_scope.iter().any(|scope| {
        scope == DRIVE_BACKEND_ADMIN_WILDCARD || scope == DRIVE_STORAGE_ADMIN_PERMISSION
    })
}

pub fn has_drive_admin_capability(context: &DriveAppContext, capability_permission: &str) -> bool {
    if has_drive_admin_privilege(context) {
        return true;
    }
    context
        .permission_scope
        .iter()
        .any(|scope| scope == capability_permission)
}

pub fn can_access_drive_storage_admin(context: &DriveAppContext) -> bool {
    has_drive_admin_privilege(context)
}

pub fn drive_backend_operation_required_permission(operation_id: &str) -> Option<&'static str> {
    match operation_id {
        "auditEvents.list" => Some(DRIVE_AUDIT_ADMIN_PERMISSION),
        "downloadPackages.list" => Some(DRIVE_DOWNLOAD_PACKAGES_ADMIN_PERMISSION),
        "spaces.admin.list" => Some(DRIVE_SPACES_ADMIN_PERMISSION),
        operation_id if operation_id.starts_with("maintenance.") => {
            Some(DRIVE_MAINTENANCE_ADMIN_PERMISSION)
        }
        operation_id if operation_id.starts_with("quotas.") => Some(DRIVE_QUOTA_ADMIN_PERMISSION),
        operation_id if operation_id.starts_with("labels.") => Some(DRIVE_LABELS_ADMIN_PERMISSION),
        operation_id
            if operation_id.starts_with("sandboxVolumes.")
                || operation_id.starts_with("sandboxGrants.") =>
        {
            Some(DRIVE_SANDBOXES_ADMIN_PERMISSION)
        }
        _ => None,
    }
}

pub fn can_invoke_drive_backend_operation(context: &DriveAppContext, operation_id: &str) -> bool {
    let Some(required_permission) = drive_backend_operation_required_permission(operation_id)
    else {
        return false;
    };
    has_drive_admin_capability(context, required_permission)
}

pub fn can_invoke_drive_storage_operation(context: &DriveAppContext, operation_id: &str) -> bool {
    let _ = operation_id;
    can_access_drive_storage_admin(context)
}

pub fn can_access_any_drive_admin_surface(context: &DriveAppContext) -> bool {
    if can_access_drive_storage_admin(context) {
        return true;
    }
    BACKEND_CAPABILITY_PERMISSIONS
        .iter()
        .any(|permission| has_drive_admin_capability(context, permission))
}

pub fn can_access_drive_backend_admin(context: &DriveAppContext) -> bool {
    can_access_any_drive_admin_surface(context)
}

pub fn can_access_drive_admin_storage(context: &DriveAppContext) -> bool {
    can_access_drive_storage_admin(context)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context_with_scopes(scopes: &[&str]) -> DriveAppContext {
        DriveAppContext {
            tenant_id: "tenant-001".to_string(),
            user_id: "user-001".to_string(),
            organization_id: None,
            session_id: None,
            app_id: Some("appbase".to_string()),
            environment: Some("prod".to_string()),
            deployment_mode: Some("saas".to_string()),
            auth_level: Some("password".to_string()),
            data_scope: Vec::new(),
            permission_scope: scopes.iter().map(|scope| (*scope).to_string()).collect(),
            actor_id: "user-001".to_string(),
            actor_kind: "user".to_string(),
            device_id: None,
            request_id: "request-001".to_string(),
            trace_id: "trace-001".to_string(),
        }
    }

    #[test]
    fn allows_drive_storage_admin_permission() {
        let context = context_with_scopes(&[DRIVE_STORAGE_ADMIN_PERMISSION]);
        assert!(can_access_drive_storage_admin(&context));
        assert!(can_invoke_drive_backend_operation(
            &context,
            "auditEvents.list"
        ));
        assert!(can_invoke_drive_storage_operation(
            &context,
            "storageProviders.list"
        ));
    }

    #[test]
    fn allows_drive_wildcard_permission() {
        let context = context_with_scopes(&[DRIVE_BACKEND_ADMIN_WILDCARD]);
        assert!(can_access_drive_storage_admin(&context));
    }

    #[test]
    fn rejects_missing_admin_permission() {
        let context = context_with_scopes(&["drive.read"]);
        assert!(!can_access_any_drive_admin_surface(&context));
        assert!(!can_invoke_drive_backend_operation(
            &context,
            "auditEvents.list"
        ));
    }

    #[test]
    fn audit_scope_allows_audit_but_not_quota_or_storage() {
        let context = context_with_scopes(&[DRIVE_AUDIT_ADMIN_PERMISSION]);
        assert!(can_invoke_drive_backend_operation(
            &context,
            "auditEvents.list"
        ));
        assert!(!can_invoke_drive_backend_operation(
            &context,
            "quotas.retrieve"
        ));
        assert!(!can_invoke_drive_storage_operation(
            &context,
            "storageProviders.list"
        ));
        assert!(can_access_any_drive_admin_surface(&context));
    }

    #[test]
    fn quota_scope_allows_quota_but_not_audit() {
        let context = context_with_scopes(&[DRIVE_QUOTA_ADMIN_PERMISSION]);
        assert!(can_invoke_drive_backend_operation(
            &context,
            "quotas.update"
        ));
        assert!(!can_invoke_drive_backend_operation(
            &context,
            "auditEvents.list"
        ));
    }

    #[test]
    fn storage_admin_alias_matches_storage_gate() {
        let context = context_with_scopes(&[DRIVE_STORAGE_ADMIN_PERMISSION]);
        assert_eq!(
            can_access_drive_admin_storage(&context),
            can_access_drive_storage_admin(&context),
        );
    }

    #[test]
    fn sandbox_scope_is_limited_to_sandbox_admin_operations() {
        let context = context_with_scopes(&[DRIVE_SANDBOXES_ADMIN_PERMISSION]);
        assert!(can_invoke_drive_backend_operation(
            &context,
            "sandboxVolumes.create"
        ));
        assert!(can_invoke_drive_backend_operation(
            &context,
            "sandboxGrants.delete"
        ));
        assert!(!can_invoke_drive_backend_operation(
            &context,
            "quotas.update"
        ));
        assert!(!can_invoke_drive_storage_operation(
            &context,
            "storageProviders.list"
        ));
    }
}
