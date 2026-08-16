use std::sync::Arc;

use axum::Router;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultOpenApiWebRequestContextResolver, DomainContextInjector, WebRequestContext,
    WebRequestContextProfile, WebRequestContextResolver,
};

use crate::http_route_manifest::open_route_manifest;

pub fn drive_open_api_public_path_prefixes() -> Vec<String> {
    let mut prefixes = sdkwork_drive_http::infra::drive_infra_public_path_prefixes();
    prefixes.push("/open/v3/api/drive/share_links".to_owned());
    prefixes
}

pub fn drive_open_api_prefixes() -> Vec<String> {
    vec!["/open/v3/api".to_owned()]
}

#[derive(Clone, Default)]
struct DriveOpenApiContextInjector;

impl DomainContextInjector for DriveOpenApiContextInjector {
    fn inject(&self, _request: &mut http::Request<axum::body::Body>, _context: &WebRequestContext) {
    }
}

pub fn wrap_router_with_web_framework<R>(resolver: R, router: Router) -> Router
where
    R: WebRequestContextResolver + Clone + 'static,
{
    with_web_request_context(router, build_open_api_framework_layer(resolver))
}

pub fn wrap_router_with_iam_web_framework(
    resolver: IamWebRequestContextResolver,
    router: Router,
) -> Router {
    with_web_request_context(router, build_open_api_framework_layer(resolver))
}

fn build_open_api_framework_layer<R>(resolver: R) -> WebFrameworkLayer<R>
where
    R: WebRequestContextResolver + Clone,
{
    let route_manifest = open_route_manifest();
    route_manifest
        .validate_public_path_prefixes(&drive_open_api_public_path_prefixes())
        .expect("drive open-api public prefixes must not cover protected manifest routes");

    let environment =
        sdkwork_drive_http::web_framework::resolve_drive_web_environment_from_process_env();
    let security_policy =
        sdkwork_drive_http::web_framework::drive_service_security_policy(&environment);

    WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            open_api_prefixes: drive_open_api_prefixes(),
            public_path_prefixes: drive_open_api_public_path_prefixes(),
            environment,
            ..WebRequestContextProfile::default()
        })
        .with_security_policy(security_policy)
        .with_route_manifest(route_manifest)
        .with_domain_injector(Arc::new(DriveOpenApiContextInjector))
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    wrap_router_with_iam_web_framework(resolver, router)
}

/// Dev/test helper when callers need a synchronous open-api web framework layer.
pub fn wrap_router_with_dev_open_api_web_framework(router: Router) -> Router {
    wrap_router_with_web_framework(DefaultOpenApiWebRequestContextResolver::default(), router)
}
