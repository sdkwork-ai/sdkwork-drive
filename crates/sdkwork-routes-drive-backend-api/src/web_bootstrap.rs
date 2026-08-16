use std::sync::Arc;

use axum::Router;
use sdkwork_drive_security::{
    can_access_any_drive_admin_surface, can_invoke_drive_backend_operation, DriveAppContext,
};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    AuthorizationPolicy, DefaultWebRequestContextResolver, DomainContextInjector,
    EnforcePrincipalTenantIsolationPolicy, WebAuthLevel, WebDeploymentMode, WebEnvironment,
    WebFrameworkError, WebLoginScope, WebRequestContext, WebRequestContextProfile,
    WebRequestPrincipal, WebSubjectType,
};

use crate::http_route_manifest::backend_route_manifest;

pub fn drive_backend_public_path_prefixes() -> Vec<String> {
    sdkwork_drive_http::infra::drive_infra_public_path_prefixes()
}

#[derive(Clone, Default)]
struct DriveBackendAuthorizationPolicy;

impl AuthorizationPolicy for DriveBackendAuthorizationPolicy {
    fn authorize(
        &self,
        ctx: &WebRequestContext,
        operation_id: Option<&str>,
    ) -> Result<(), WebFrameworkError> {
        let principal = ctx.principal.as_ref().ok_or_else(|| {
            WebFrameworkError::missing_credentials("authenticated principal is required")
        })?;
        if ctx.login_scope() == Some(WebLoginScope::Tenant) {
            return Err(WebFrameworkError::forbidden(
                "backend API rejects personal sessions (login_scope TENANT)",
            ));
        }
        let app_context = drive_app_context_from_web_principal(principal, ctx);
        if let Some(operation_id) = operation_id {
            if can_invoke_drive_backend_operation(&app_context, operation_id) {
                return Ok(());
            }
            return Err(WebFrameworkError::forbidden(
                "Drive backend admin permission is required for this operation",
            ));
        }
        if can_access_any_drive_admin_surface(&app_context) {
            return Ok(());
        }
        Err(WebFrameworkError::forbidden(
            "Drive backend admin permission is required",
        ))
    }
}

#[derive(Clone, Default)]
struct DriveBackendContextInjector;

impl DomainContextInjector for DriveBackendContextInjector {
    fn inject(&self, request: &mut http::Request<axum::body::Body>, context: &WebRequestContext) {
        if let Some(principal) = context.principal.as_ref() {
            let mut app_context = drive_app_context_from_web_principal(principal, context);
            sdkwork_drive_http::web_app_context::apply_trace_id_from_transport(
                request.headers(),
                &mut app_context,
            );
            request.extensions_mut().insert(app_context);
        }
    }
}

pub fn drive_app_context_from_web_principal(
    principal: &WebRequestPrincipal,
    context: &WebRequestContext,
) -> DriveAppContext {
    let environment = match principal.app.environment {
        WebEnvironment::Dev => Some("dev".to_owned()),
        WebEnvironment::Test => Some("test".to_owned()),
        WebEnvironment::Prod => Some("prod".to_owned()),
    };
    let deployment_mode = match principal.app.deployment_mode {
        WebDeploymentMode::Saas => Some("saas".to_owned()),
        WebDeploymentMode::Local => Some("local".to_owned()),
        WebDeploymentMode::Private => Some("private".to_owned()),
    };
    let auth_level = match principal.auth.auth_level {
        WebAuthLevel::Anonymous => Some("anonymous".to_owned()),
        WebAuthLevel::Password => Some("password".to_owned()),
        WebAuthLevel::Mfa => Some("mfa".to_owned()),
        WebAuthLevel::System | WebAuthLevel::ApiKey => Some("system".to_owned()),
    };
    let actor_kind = match principal.subject.subject_type {
        WebSubjectType::User => "user".to_owned(),
        WebSubjectType::Service => "service".to_owned(),
        WebSubjectType::System => "system".to_owned(),
        WebSubjectType::ApiKey => "api_key".to_owned(),
    };

    DriveAppContext {
        tenant_id: principal.tenant_id().to_owned(),
        user_id: principal.user_id().to_owned(),
        organization_id: principal.organization_id().map(str::to_owned),
        session_id: principal.session_id().map(str::to_owned),
        app_id: Some(principal.app_id().to_owned()),
        environment,
        deployment_mode,
        auth_level,
        data_scope: principal.scopes.data_scope.clone(),
        permission_scope: principal.scopes.permission_scope.clone(),
        actor_id: principal.user_id().to_owned(),
        actor_kind,
        device_id: None,
        request_id: context.request_id.0.clone(),
        trace_id: String::new(),
    }
}

pub fn wrap_router_with_web_framework(
    resolver: DefaultWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(router, build_drive_backend_framework_layer(resolver))
}

pub fn wrap_router_with_iam_web_framework(
    resolver: IamWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(router, build_drive_backend_framework_layer(resolver))
}

fn build_drive_backend_framework_layer<R>(resolver: R) -> WebFrameworkLayer<R>
where
    R: sdkwork_web_core::WebRequestContextResolver + Clone,
{
    let route_manifest = backend_route_manifest();
    route_manifest
        .validate_public_path_prefixes(&drive_backend_public_path_prefixes())
        .expect("drive backend-api public prefixes must not cover protected manifest routes");

    let environment =
        sdkwork_drive_http::web_framework::resolve_drive_web_environment_from_process_env();
    let security_policy =
        sdkwork_drive_http::web_framework::drive_service_security_policy(&environment);

    WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            public_path_prefixes: drive_backend_public_path_prefixes(),
            environment,
            ..WebRequestContextProfile::default()
        })
        .with_security_policy(security_policy)
        .with_route_manifest(route_manifest)
        .with_tenant_isolation_policy(Arc::new(EnforcePrincipalTenantIsolationPolicy))
        .with_authorization_policy(Arc::new(DriveBackendAuthorizationPolicy))
        .with_domain_injector(Arc::new(DriveBackendContextInjector))
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    wrap_router_with_iam_web_framework(resolver, router)
}
