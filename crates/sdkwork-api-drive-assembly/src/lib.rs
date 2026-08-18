//! Gateway assembly for sdkwork-drive.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM

mod bootstrap;
mod generated;

pub use bootstrap::{
    assemble_api_router, assemble_api_router_from_env, assemble_app_api_contribution,
    assemble_backend_admin_storage_contribution_with_pool, assemble_backend_business_router_from_env,
    assemble_business_routes, assemble_business_routes_from_env, assemble_business_routes_with_config,
    assemble_business_routes_with_process_pool, ApiAssembly, ApiAssemblyContribution,
    BusinessRouterAssembly,
};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}

/// App-api surface route manifest owned by the Drive dependency assembly.
pub fn app_api_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    sdkwork_routes_drive_app_api::app_route_manifest()
}

/// Admin storage backend-api surface route manifest owned by the Drive dependency assembly.
pub fn backend_admin_storage_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    sdkwork_routes_storage_backend_api::storage_route_manifest()
}
