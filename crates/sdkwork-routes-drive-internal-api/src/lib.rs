#![allow(clippy::result_large_err)]

mod content;
mod dto;
mod error;
mod handlers;
pub mod http_route_manifest;
mod response;
mod routes;
mod state;
mod web_bootstrap;

pub use http_route_manifest::internal_route_manifest;
pub use routes::{
    build_internal_business_router, build_protected_router_with_pool, build_router_with_pool,
    gateway_mount, gateway_mount_business,
};

pub fn gateway_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    internal_route_manifest()
}
