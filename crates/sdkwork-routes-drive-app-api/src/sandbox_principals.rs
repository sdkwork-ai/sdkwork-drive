use sdkwork_drive_workspace_service::ports::sandbox_principal_resolver::EffectiveSandboxPrincipal;

use crate::app_context::DriveRequestContext;

/// Produces only principals cryptographically bound to the verified app session.
/// Workspace and role principals require their respective authoritative resolvers.
pub(crate) fn token_bound_sandbox_principals(
    context: &DriveRequestContext,
) -> Vec<EffectiveSandboxPrincipal> {
    let mut principals = vec![EffectiveSandboxPrincipal {
        subject_type: "user".to_string(),
        subject_id: context.user_id.clone(),
    }];
    if let Some(organization_id) = context
        .organization_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        principals.push(EffectiveSandboxPrincipal {
            subject_type: "organization".to_string(),
            subject_id: organization_id.to_string(),
        });
    }
    principals
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context(organization_id: Option<&str>) -> DriveRequestContext {
        DriveRequestContext {
            tenant_id: "tenant-1".to_string(),
            user_id: "user-1".to_string(),
            organization_id: organization_id.map(str::to_string),
            app_id: None,
            actor_id: "user-1".to_string(),
            subject_type: "user".to_string(),
            subject_id: "user-1".to_string(),
            auth_level: sdkwork_web_core::WebAuthLevel::Password,
            request_id: "request-1".to_string(),
            trace_id: "trace-1".to_string(),
            from_token: true,
        }
    }
    #[test]
    fn includes_only_verified_user_and_current_organization() {
        let principals = token_bound_sandbox_principals(&context(Some("organization-1")));
        assert_eq!(principals.len(), 2);
        assert_eq!(principals[0].subject_type, "user");
        assert_eq!(principals[1].subject_type, "organization");
    }
}
