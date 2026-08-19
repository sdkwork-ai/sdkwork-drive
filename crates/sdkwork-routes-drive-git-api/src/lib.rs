#![allow(clippy::result_large_err)]

mod http_route_manifest;
mod routes;
mod state;

pub use http_route_manifest::git_route_manifest;
pub use routes::{build_git_business_router, gateway_mount_business};
pub use state::GitState;

pub fn gateway_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    git_route_manifest()
}
