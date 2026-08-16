#![allow(clippy::result_large_err)]

mod audit;
mod audit_event_handlers;
mod auth;
mod download_package_handlers;
mod dto;
mod error;
mod handlers;
pub mod http_route_manifest;
mod label_handlers;
mod maintenance_handlers;
mod mappers;
mod rate_limit;
mod response;
mod routes;
mod sandbox_admin_dto;
mod sandbox_admin_handlers;
mod space_quota_handlers;
mod state;
mod tenant_context;
mod validators;
mod web_bootstrap;

pub use http_route_manifest::backend_route_manifest;
pub use routes::*;
pub use web_bootstrap::{
    wrap_router_with_iam_web_framework, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env,
};

pub fn gateway_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    backend_route_manifest()
}
