use crate::error::{problem, ProblemDetail, SdkWorkResultCode};
use crate::validators::validate_subject_type;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_security::DriveAppContext;
use sdkwork_web_core::WebAuthLevel;

#[derive(Debug, Clone)]
pub(crate) struct DriveRequestContext {
    pub(crate) tenant_id: String,
    pub(crate) user_id: String,
    pub(crate) organization_id: Option<String>,
    pub(crate) app_id: Option<String>,
    pub(crate) actor_id: String,
    pub(crate) subject_type: String,
    pub(crate) subject_id: String,
    pub(crate) auth_level: WebAuthLevel,
    pub(crate) request_id: String,
    pub(crate) trace_id: String,
    pub(crate) from_token: bool,
}

impl DriveRequestContext {
    pub(crate) fn from_app_context(
        app_context: &DriveAppContext,
        auth_level: WebAuthLevel,
    ) -> Self {
        Self {
            tenant_id: app_context.tenant_id.clone(),
            user_id: app_context.user_id.clone(),
            organization_id: app_context.organization_id.clone(),
            app_id: app_context.app_id.clone(),
            actor_id: app_context.actor_id.clone(),
            subject_type: app_context.actor_kind.clone(),
            subject_id: app_context.user_id.clone(),
            auth_level,
            request_id: app_context.request_id.clone(),
            trace_id: app_context.trace_id.clone(),
            from_token: true,
        }
    }

    pub(crate) fn resolve_tenant_id(&self) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
        self.require_verified_context()?;
        Ok(self.tenant_id.clone())
    }

    pub(crate) fn resolve_operator_id(&self) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
        self.require_verified_context()?;
        Ok(self.actor_id.clone())
    }

    pub(crate) fn resolve_app_id(&self) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
        self.require_verified_context()?;
        self.app_id.clone().ok_or_else(|| {
            problem(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "verified WebRequestContext app identity is required",
                SdkWorkResultCode::AuthenticationRequired,
            )
        })
    }

    pub(crate) fn resolve_subject(
        &self,
    ) -> Result<(String, String), (StatusCode, Json<ProblemDetail>)> {
        self.require_verified_context()?;
        validate_subject_type(&self.subject_type)?;
        Ok((self.subject_type.clone(), self.subject_id.clone()))
    }

    pub(crate) fn require_verified_context(&self) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
        if self.from_token {
            return Ok(());
        }
        Err(problem(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "verified WebRequestContext is required",
            SdkWorkResultCode::AuthenticationRequired,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(from_token: bool) -> DriveRequestContext {
        DriveRequestContext {
            tenant_id: "tenant-001".to_string(),
            user_id: "user-001".to_string(),
            organization_id: None,
            app_id: Some("appbase".to_string()),
            actor_id: "user-001".to_string(),
            subject_type: "user".to_string(),
            subject_id: "user-001".to_string(),
            auth_level: WebAuthLevel::Password,
            request_id: "request-001".to_string(),
            trace_id: "trace-001".to_string(),
            from_token,
        }
    }

    #[test]
    fn resolve_tenant_id_from_token_when_query_is_omitted() {
        let ctx = context(true);

        assert_eq!(ctx.resolve_tenant_id().expect("tenant"), "tenant-001");
    }

    #[test]
    fn reject_tenant_resolution_without_token_context() {
        let ctx = context(false);

        let error = ctx.resolve_tenant_id().expect_err("missing token context");
        assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn resolve_subject_from_token_when_query_is_omitted() {
        let ctx = context(true);

        let (subject_type, subject_id) = ctx.resolve_subject().expect("subject");
        assert_eq!(subject_type, "user");
        assert_eq!(subject_id, "user-001");
    }

    #[test]
    fn resolve_app_id_from_token_when_body_omits_it() {
        let ctx = context(true);

        assert_eq!(ctx.resolve_app_id().expect("app"), "appbase");
    }

    #[test]
    fn operator_resolution_requires_verified_context_in_every_profile() {
        let ctx = context(false);

        let error = ctx
            .resolve_operator_id()
            .expect_err("verified context should be required");
        assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    }
}
