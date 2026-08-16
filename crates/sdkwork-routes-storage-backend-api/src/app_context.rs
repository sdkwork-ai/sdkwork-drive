use crate::error::{map_service_error, ProblemDetail};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_security::DriveAppContext;
use sdkwork_drive_workspace_service::DriveServiceError;

#[derive(Debug, Clone)]
pub(crate) struct DriveRequestContext {
    pub(crate) tenant_id: String,
    pub(crate) actor_id: String,
    from_token: bool,
    test_mode: bool,
}

impl DriveRequestContext {
    pub(crate) fn from_app_context(app_context: &DriveAppContext) -> Self {
        Self {
            tenant_id: app_context.tenant_id.clone(),
            actor_id: app_context.actor_id.clone(),
            from_token: true,
            test_mode: false,
        }
    }

    pub(crate) fn resolve_tenant_id(&self) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
        if self.test_mode || self.from_token {
            Ok(self.tenant_id.clone())
        } else {
            Err(map_service_error(DriveServiceError::Validation(
                "verified WebRequestContext is required".to_string(),
            )))
        }
    }

    pub(crate) fn resolve_operator_id(&self) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
        if self.test_mode || self.from_token {
            Ok(self.actor_id.clone())
        } else {
            Err(map_service_error(DriveServiceError::Validation(
                "verified WebRequestContext is required".to_string(),
            )))
        }
    }
}

/// Test-only default tenant context for routers built with `require_iam = false`.
pub(crate) fn default_test_drive_request_context() -> DriveRequestContext {
    DriveRequestContext {
        tenant_id: "tenant-storage".to_string(),
        actor_id: "admin-storage".to_string(),
        from_token: false,
        test_mode: true,
    }
}

/// Test-only tenant context with a custom tenant id for routers built with `require_iam = false`.
pub(crate) fn test_drive_request_context_with_tenant(tenant_id: String) -> DriveRequestContext {
    DriveRequestContext {
        tenant_id,
        actor_id: "admin-storage".to_string(),
        from_token: false,
        test_mode: true,
    }
}
