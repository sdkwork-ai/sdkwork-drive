use std::sync::Arc;

use axum::Router;
use sdkwork_drive_security::{can_invoke_drive_storage_operation, DriveAppContext};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    AuthorizationPolicy, DefaultWebRequestContextResolver, DomainContextInjector,
    EnforcePrincipalTenantIsolationPolicy, WebAuthLevel, WebDeploymentMode, WebEnvironment,
    WebFrameworkError, WebLoginScope, WebRequestContext, WebRequestContextProfile,
    WebRequestPrincipal, WebSubjectType,
};

use crate::app_context::DriveRequestContext;
use crate::http_route_manifest::storage_route_manifest;

pub fn drive_admin_storage_public_path_prefixes() -> Vec<String> {
    sdkwork_drive_http::infra::drive_infra_public_path_prefixes()
}

pub fn drive_admin_storage_gateway_api_prefixes() -> Vec<String> {
    vec!["/admin/v3/api".to_owned(), "/backend/v3/api".to_owned()]
}

#[derive(Clone, Default)]
struct DriveAdminStorageContextInjector;

impl DomainContextInjector for DriveAdminStorageContextInjector {
    fn inject(&self, request: &mut http::Request<axum::body::Body>, context: &WebRequestContext) {
        let Some(principal) = context.principal.as_ref() else {
            return;
        };

        let mut app_context = drive_app_context_from_web_principal(principal, context);
        sdkwork_drive_http::web_app_context::apply_trace_id_from_transport(
            request.headers(),
            &mut app_context,
        );
        let drive_context = DriveRequestContext::from_app_context(&app_context);
        request.extensions_mut().insert(app_context);
        request.extensions_mut().insert(drive_context);
    }
}

/// Derives the drive storage request context from the host web framework's
/// `WebRequestContext` extension for composing gateways that own the Web
/// Framework layer and do not register drive-specific domain injectors
/// (same-origin dependency composition, API_ASSEMBLY_SPEC §3/§6.1).
/// No-ops when the host already injected a drive context (drive gateway path).
pub async fn derive_drive_storage_context_from_web_context(
    mut request: axum::extract::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if request.extensions().get::<DriveRequestContext>().is_some() {
        return next.run(request).await;
    }
    let Some(context) = request.extensions().get::<WebRequestContext>().cloned() else {
        return next.run(request).await;
    };
    let Some(principal) = context.principal.as_ref() else {
        return next.run(request).await;
    };

    let mut app_context = drive_app_context_from_web_principal(principal, &context);
    sdkwork_drive_http::web_app_context::apply_trace_id_from_transport(
        request.headers(),
        &mut app_context,
    );
    let drive_context = DriveRequestContext::from_app_context(&app_context);
    request.extensions_mut().insert(app_context);
    request.extensions_mut().insert(drive_context);
    next.run(request).await
}

#[derive(Clone, Default)]
struct DriveAdminStorageAuthorizationPolicy;

impl AuthorizationPolicy for DriveAdminStorageAuthorizationPolicy {
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
            if can_invoke_drive_storage_operation(&app_context, operation_id) {
                return Ok(());
            }
            return Err(WebFrameworkError::forbidden(
                "Drive storage admin permission is required for this operation",
            ));
        }
        if can_invoke_drive_storage_operation(&app_context, "storageProviders.list") {
            return Ok(());
        }
        Err(WebFrameworkError::forbidden(
            "Drive storage admin permission is required",
        ))
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
    with_web_request_context(router, build_drive_admin_storage_framework_layer(resolver))
}

pub fn wrap_router_with_iam_web_framework(
    resolver: IamWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(router, build_drive_admin_storage_framework_layer(resolver))
}

fn build_drive_admin_storage_framework_layer<R>(resolver: R) -> WebFrameworkLayer<R>
where
    R: sdkwork_web_core::WebRequestContextResolver + Clone,
{
    let route_manifest = storage_route_manifest();
    route_manifest
        .validate_public_path_prefixes(&drive_admin_storage_public_path_prefixes())
        .expect(
            "drive storage backend-api public prefixes must not cover protected manifest routes",
        );

    let environment =
        sdkwork_drive_http::web_framework::resolve_drive_web_environment_from_process_env();
    let security_policy =
        sdkwork_drive_http::web_framework::drive_service_security_policy(&environment);

    WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            gateway_api_prefixes: drive_admin_storage_gateway_api_prefixes(),
            public_path_prefixes: drive_admin_storage_public_path_prefixes(),
            environment,
            ..WebRequestContextProfile::default()
        })
        .with_security_policy(security_policy)
        .with_route_manifest(route_manifest)
        .with_tenant_isolation_policy(Arc::new(EnforcePrincipalTenantIsolationPolicy))
        .with_domain_injector(Arc::new(DriveAdminStorageContextInjector))
        .with_authorization_policy(Arc::new(DriveAdminStorageAuthorizationPolicy))
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    wrap_router_with_iam_web_framework(resolver, router)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::{Extension, Request};
    use axum::http::{Method, StatusCode};
    use axum::middleware;
    use axum::routing::get;
    use sdkwork_web_core::{
        ServerRequestId, WebApiSurface, WebAuthLevel, WebAuthMode, WebDeploymentMode,
        WebEnvironment, WebLoginScope, WebRequestPrincipal, WebTransportFacts,
    };
    use tower::util::ServiceExt;

    fn web_context_with_principal() -> WebRequestContext {
        let principal = WebRequestPrincipal::builder()
            .tenant_id("100001")
            .organization_id(Some("100002".to_owned()))
            .login_scope(WebLoginScope::Organization)
            .user_id("2")
            .session_id(Some("session-test".to_owned()))
            .app_id("sdkwork-cloudrouter")
            .environment(WebEnvironment::Test)
            .deployment_mode(WebDeploymentMode::Local)
            .auth_level(WebAuthLevel::Password)
            .permission_scope(vec!["cloudrouter.admin.access".to_owned()])
            .build();
        WebRequestContext {
            request_id: ServerRequestId("request-test".to_owned()),
            api_surface: WebApiSurface::BackendApi,
            auth_mode: WebAuthMode::DualToken,
            transport: WebTransportFacts {
                path: "/backend/v3/api/drive/storage/providers".to_owned(),
                method: "GET".to_owned(),
                auth_token_present: true,
                access_token_present: true,
                api_key_present: false,
                ingress_token_present: false,
                oauth_bearer_present: false,
                agent_token_present: false,
            },
            principal: Some(principal),
            locale: None,
            client_kind: None,
            operation: None,
            trace_id: None,
            idempotency_key: None,
        }
    }

    async fn echo_drive_tenant(Extension(ctx): Extension<DriveRequestContext>) -> String {
        ctx.tenant_id
    }

    #[tokio::test]
    async fn host_framework_middleware_derives_drive_request_context_from_web_context() {
        let router = Router::new()
            .route(
                "/backend/v3/api/drive/storage/providers",
                get(echo_drive_tenant),
            )
            .route_layer(middleware::from_fn(
                derive_drive_storage_context_from_web_context,
            ));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/backend/v3/api/drive/storage/providers")
            .body(Body::empty())
            .unwrap();
        let mut request_with_context = request;
        request_with_context
            .extensions_mut()
            .insert(web_context_with_principal());

        let response = router.clone().oneshot(request_with_context).await.unwrap();
        assert_eq!(
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
            "100001"
        );
    }

    #[tokio::test]
    async fn host_framework_middleware_is_noop_without_host_web_context() {
        let router = Router::new()
            .route(
                "/backend/v3/api/drive/storage/providers",
                get(echo_drive_tenant),
            )
            .route_layer(middleware::from_fn(
                derive_drive_storage_context_from_web_context,
            ));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/backend/v3/api/drive/storage/providers")
            .body(Body::empty())
            .unwrap();

        // Without a host `WebRequestContext` the middleware passes through and
        // the handler fails on the missing drive extension (no derivation).
        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
