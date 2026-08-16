#![allow(clippy::result_large_err)]

mod dto;
mod error;
mod handlers;
mod http_route_manifest;
mod rate_limit;
mod repository;
mod response;
mod routes;
mod state;
mod storage;
mod time;
mod web_bootstrap;

pub use http_route_manifest::open_route_manifest;
pub use routes::{
    build_gateway_business_router_with_pool, build_open_business_router,
    build_protected_router_with_pool, build_router_with_database_config,
    build_router_with_database_url, build_router_with_pool, gateway_mount, gateway_mount_business,
};
pub use state::OpenState;
pub use web_bootstrap::wrap_router_with_web_framework_from_env;

pub fn gateway_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    open_route_manifest()
}
