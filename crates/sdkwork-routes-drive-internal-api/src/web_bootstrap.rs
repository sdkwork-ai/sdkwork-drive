use axum::Router;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, WebRequestContextProfile, WebRequestContextResolver,
};

use crate::http_route_manifest::internal_route_manifest;

pub fn wrap_with_default_resolver(router: Router) -> Router {
    wrap_with_resolver(DefaultWebRequestContextResolver::default(), router)
}

pub fn wrap_with_iam_resolver(resolver: IamWebRequestContextResolver, router: Router) -> Router {
    wrap_with_resolver(resolver, router)
}

pub async fn wrap_from_env(router: Router) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    wrap_with_iam_resolver(resolver, router)
}

fn wrap_with_resolver<R>(resolver: R, router: Router) -> Router
where
    R: WebRequestContextResolver + Clone + Send + Sync + 'static,
{
    let route_manifest = internal_route_manifest();
    let profile = WebRequestContextProfile {
        public_path_prefixes: Vec::new(),
        ..WebRequestContextProfile::default()
    };
    route_manifest
        .validate_route_auth_for_surfaces(&profile)
        .expect("Drive internal-api routes must use ingress-token auth");
    let environment =
        sdkwork_drive_http::web_framework::resolve_drive_web_environment_from_process_env();
    let security_policy =
        sdkwork_drive_http::web_framework::drive_service_security_policy(&environment);
    with_web_request_context(
        router,
        WebFrameworkLayer::new(resolver)
            .with_profile(WebRequestContextProfile {
                environment,
                ..profile
            })
            .with_security_policy(security_policy)
            .with_route_manifest(route_manifest),
    )
}
