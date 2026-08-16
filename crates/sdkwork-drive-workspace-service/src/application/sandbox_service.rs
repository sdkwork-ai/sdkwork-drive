use crate::domain::sandbox::{AuthorizedSandboxMount, DriveSandboxVolume};
use crate::domain::sandbox_directory::SandboxDirectoryAccess;
use crate::ports::sandbox_principal_resolver::EffectiveSandboxPrincipal;
use crate::ports::sandbox_store::DriveSandboxStore;
use crate::DriveServiceError;

pub struct DriveSandboxService<S> {
    store: S,
}

impl<S: DriveSandboxStore> DriveSandboxService<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn list_accessible(
        &self,
        tenant_id: &str,
        subject_type: &str,
        subject_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveSandboxVolume>, DriveServiceError> {
        if tenant_id.trim().is_empty()
            || subject_type.trim().is_empty()
            || subject_id.trim().is_empty()
        {
            return Err(DriveServiceError::Validation(
                "tenant and subject are required".to_string(),
            ));
        }
        if !(1..=201).contains(&limit) || offset < 0 {
            return Err(DriveServiceError::Validation(
                "pagination is invalid".to_string(),
            ));
        }
        self.store
            .list_accessible(tenant_id, subject_type, subject_id, offset, limit)
            .await
    }

    pub async fn list_accessible_for_principals(
        &self,
        tenant_id: &str,
        principals: &[EffectiveSandboxPrincipal],
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<DriveSandboxVolume>, i64), DriveServiceError> {
        if offset < 0 || !(1..=201).contains(&limit) {
            return Err(DriveServiceError::Validation(
                "sandbox principal query is invalid".to_string(),
            ));
        }
        validate_principals(tenant_id, principals)?;
        self.store
            .list_accessible_for_principals(tenant_id, principals, offset, limit)
            .await
    }

    pub async fn require_full_access(
        &self,
        tenant_id: &str,
        sandbox_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<(), DriveServiceError> {
        if tenant_id.trim().is_empty()
            || sandbox_id.trim().is_empty()
            || subject_type.trim().is_empty()
            || subject_id.trim().is_empty()
        {
            return Err(DriveServiceError::Validation(
                "tenant, sandbox, and subject are required".to_string(),
            ));
        }
        let grant = self
            .store
            .get_grant(tenant_id, sandbox_id, subject_type, subject_id)
            .await?;
        match grant {
            Some(value) if value.access_level == "full" && value.lifecycle_status == "active" => {
                Ok(())
            }
            Some(_) => Err(DriveServiceError::PermissionDenied(
                "sandbox is read only".to_string(),
            )),
            None => Err(DriveServiceError::PermissionDenied(
                "sandbox access is denied".to_string(),
            )),
        }
    }

    pub async fn require_read_access(
        &self,
        tenant_id: &str,
        sandbox_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<(), DriveServiceError> {
        if tenant_id.trim().is_empty()
            || sandbox_id.trim().is_empty()
            || subject_type.trim().is_empty()
            || subject_id.trim().is_empty()
        {
            return Err(DriveServiceError::Validation(
                "tenant, sandbox, and subject are required".to_string(),
            ));
        }
        match self
            .store
            .get_grant(tenant_id, sandbox_id, subject_type, subject_id)
            .await?
        {
            Some(value) if matches!(value.lifecycle_status.as_str(), "active" | "read_only") => {
                Ok(())
            }
            Some(_) => Err(DriveServiceError::PermissionDenied(
                "sandbox access is denied".to_string(),
            )),
            None => Err(DriveServiceError::PermissionDenied(
                "sandbox access is denied".to_string(),
            )),
        }
    }

    pub async fn authorize_mount_for_principals(
        &self,
        tenant_id: &str,
        sandbox_id: &str,
        principals: &[EffectiveSandboxPrincipal],
        required_access: SandboxDirectoryAccess,
    ) -> Result<AuthorizedSandboxMount, DriveServiceError> {
        if sandbox_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "sandbox is required".to_string(),
            ));
        }
        validate_principals(tenant_id, principals)?;

        let mount = self
            .store
            .get_authorized_mount_for_principals(tenant_id, sandbox_id, principals)
            .await?
            .ok_or_else(|| {
                DriveServiceError::PermissionDenied("sandbox access is denied".to_string())
            })?;

        if mount.private_root_ref().trim().is_empty()
            || mount.provider_kind().trim().is_empty()
            || mount.root_entry_id().trim().is_empty()
        {
            return Err(DriveServiceError::Internal(
                "authorized sandbox mount configuration is incomplete".to_string(),
            ));
        }

        match required_access {
            SandboxDirectoryAccess::Read
                if matches!(mount.lifecycle_status(), "active" | "read_only") =>
            {
                Ok(mount)
            }
            SandboxDirectoryAccess::Full
                if mount.lifecycle_status() == "active" && mount.effective_access() == "full" =>
            {
                Ok(mount)
            }
            SandboxDirectoryAccess::Full
                if mount.lifecycle_status() == "read_only"
                    || mount.effective_access() == "read_only" =>
            {
                Err(DriveServiceError::PermissionDenied(
                    "sandbox is read only".to_string(),
                ))
            }
            _ => Err(DriveServiceError::PermissionDenied(
                "sandbox access is denied".to_string(),
            )),
        }
    }
}

fn validate_principals(
    tenant_id: &str,
    principals: &[EffectiveSandboxPrincipal],
) -> Result<(), DriveServiceError> {
    if tenant_id.trim().is_empty() || principals.is_empty() || principals.len() > 64 {
        return Err(DriveServiceError::Validation(
            "sandbox principal query is invalid".to_string(),
        ));
    }
    if principals.iter().any(|principal| {
        principal.subject_type.trim().is_empty() || principal.subject_id.trim().is_empty()
    }) {
        return Err(DriveServiceError::Validation(
            "sandbox principal is invalid".to_string(),
        ));
    }
    Ok(())
}
