#![allow(clippy::result_large_err)]

pub mod acl;
pub mod acl_sql;
pub mod app_context;
mod archive;
mod archive_storage;
mod change_handlers;
mod collaboration_repository;
mod comment_handlers;
pub mod constants;
mod download_handlers;
mod download_packages;
pub mod dto;
pub mod error;
mod handlers;
pub mod hashing;
pub mod http_route_manifest;
pub mod ids;
mod library_handlers;
pub mod mappers;
mod metadata_handlers;
mod metadata_repository;
mod node_handlers;
mod node_lifecycle;
pub mod node_repository;
mod node_support;
mod object_store;
pub mod pagination_guard;
mod permission_handlers;
mod quota_handlers;
pub mod rate_limit;
pub mod response;
mod route_change;
mod routes;
mod runtime_sandbox_roots;
mod sandbox_handlers;
mod sandbox_principals;
mod search_handlers;
mod share_link_handlers;
mod space_handlers;
mod space_repository;
pub mod state;
mod storage_keys;
mod time;
mod trash_handlers;
mod upload_handlers;
mod upload_support;
mod uploader;
pub mod validators;
mod version_handlers;
mod watch_handlers;
mod watch_repository;
mod web_bootstrap;
mod webhook_url;
mod website_root_handlers;
mod website_sync_handlers;

pub mod composition_host {
    pub use super::acl;
    pub use super::acl_sql;
    pub use super::app_context;
    pub use super::constants;
    pub use super::dto;
    pub use super::error;
    pub use super::hashing;
    pub use super::ids;
    pub use super::mappers;
    pub use super::node_repository;
    pub use super::response;
    pub use super::state;
    pub use super::validators;
    pub use super::pagination_guard;
    pub use super::rate_limit;
}

pub use http_route_manifest::app_route_manifest;
pub use routes::*;
pub use state::AppState;
pub use web_bootstrap::{
    drive_app_context_injector, wrap_router_with_iam_web_framework, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env,
};

pub fn gateway_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    app_route_manifest()
}
