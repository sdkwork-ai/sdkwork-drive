#![allow(clippy::result_large_err)]

mod acl;
mod acl_sql;
mod app_context;
mod archive;
mod archive_storage;
mod assets;
mod change_handlers;
mod collaboration_repository;
mod comment_handlers;
mod constants;
mod download_handlers;
mod download_packages;
mod dto;
mod error;
mod handlers;
mod hashing;
pub mod http_route_manifest;
mod ids;
mod library_handlers;
mod mappers;
mod metadata_handlers;
mod metadata_repository;
mod node_handlers;
mod node_lifecycle;
mod node_repository;
mod node_support;
mod object_store;
mod pagination_guard;
mod permission_handlers;
mod quota_handlers;
mod rate_limit;
mod response;
mod route_change;
mod routes;
mod runtime_sandbox_roots;
mod sandbox_handlers;
mod sandbox_principals;
mod search_handlers;
mod share_link_handlers;
mod space_handlers;
mod space_repository;
mod state;
mod storage_keys;
mod time;
mod trash_handlers;
mod upload_handlers;
mod upload_support;
mod uploader;
mod validators;
mod version_handlers;
mod watch_handlers;
mod watch_repository;
mod web_bootstrap;
mod webhook_url;
mod website_root_handlers;
mod website_sync_handlers;

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
