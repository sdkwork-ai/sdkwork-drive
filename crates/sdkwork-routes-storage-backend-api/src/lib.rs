#![allow(clippy::result_large_err)]

mod app_context;
mod audit;
mod auth;
mod binding_handlers;
mod bucket_handlers;
mod config;
mod dto;
mod error;
mod handlers;
pub mod http_route_manifest;
mod object_handlers;
mod object_store;
mod provider_handlers;
mod provider_kind_handlers;
mod provider_lookup;
mod provider_mappers;
mod rate_limit;
mod response;
mod route_paths;
mod routes;
mod state;
mod validators;
mod web_bootstrap;

pub use config::{AdminStorageConfig, DriveAdminStorageObjectStoreAdapter};
pub use http_route_manifest::storage_route_manifest;
pub use routes::*;
pub use state::AdminStorageState;
pub use web_bootstrap::{
    wrap_router_with_iam_web_framework, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env,
};

use sdkwork_web_core::HttpRouteManifest;
use sqlx::PgPool;

pub fn gateway_route_manifest() -> HttpRouteManifest {
    storage_route_manifest()
}

pub fn gateway_mount(pool: PgPool) -> axum::Router {
    build_router_with_pool(pool)
}
