//! Authored API assembly bootstrap for SDKWork Drive.
//!
//! Drive has provider-specific startup and admin-storage configuration, so this file is
//! intentionally preserved by the assembly materializer. Business surfaces mount shared
//! infrastructure exactly once at the assembly boundary.

use std::sync::Arc;

use axum::Router;
use sdkwork_drive_config::DatabaseConfig;
use sdkwork_drive_http::infra::PostgresReadinessCheck;
use sdkwork_drive_workspace_service::application::download_service::ensure_production_download_token_signing_configured;
use sdkwork_drive_workspace_service::bootstrap::bootstrap_drive_database;
use sdkwork_drive_workspace_service::infrastructure::outbox_dispatch::ensure_domain_outbox_dispatcher;
use sdkwork_drive_workspace_service::infrastructure::sql::{
    connect_postgres_database_and_install_schema, postgres_pool_from_database_pool,
};
pub use sdkwork_web_bootstrap::ApiAssemblyContribution;
use sdkwork_web_core::HttpRouteManifest;

pub type ApiAssembly = ApiAssemblyContribution;

/// Drive App API route manifest for host gateway composition.
///
/// Host gateways that merge the unwrapped App surface contribution compose
/// this manifest into their own surface route inventory so the Web Framework
/// honors the App routes' declared authentication and permissions
/// (API_ASSEMBLY_SPEC §3).
pub fn app_api_route_manifest() -> HttpRouteManifest {
    sdkwork_routes_drive_app_api::app_route_manifest()
}

/// Drive Admin Storage backend route manifest for host gateway composition.
///
/// Same-origin dependency hosts mount the storage backend surface through
/// `assemble_backend_admin_storage_contribution_with_pool`; this manifest
/// must be composed into the host backend route inventory so auth resolution
/// matches the mounted router (API_ASSEMBLY_SPEC §3/§6.1).
pub fn backend_admin_storage_route_manifest() -> HttpRouteManifest {
    sdkwork_routes_storage_backend_api::storage_route_manifest()
}

pub struct BusinessRouterAssembly {
    pub router: Router,
}

async fn assemble_application_business_routes(pool: sqlx::PgPool) -> BusinessRouterAssembly {
    let mut router = Router::new();
    router = router.merge(sdkwork_routes_drive_app_api::gateway_mount_business(pool.clone()).await);
    router =
        router.merge(sdkwork_routes_drive_backend_api::gateway_mount_business(pool.clone()).await);
    router =
        router.merge(sdkwork_routes_drive_internal_api::gateway_mount_business(pool.clone()).await);
    router =
        router.merge(sdkwork_routes_drive_open_api::gateway_mount_business(pool.clone()).await);
    BusinessRouterAssembly { router }
}

pub async fn assemble_business_routes(pool: sqlx::PgPool) -> BusinessRouterAssembly {
    let application = assemble_application_business_routes(pool.clone()).await;
    let admin_storage = sdkwork_routes_storage_backend_api::gateway_mount_business(pool).await;
    BusinessRouterAssembly {
        router: application.router.merge(admin_storage),
    }
}

pub async fn assemble_business_routes_with_config(
    pool: sqlx::PgPool,
    admin_storage_config: sdkwork_routes_storage_backend_api::AdminStorageConfig,
) -> BusinessRouterAssembly {
    let application = assemble_application_business_routes(pool.clone()).await;
    let admin_storage = sdkwork_routes_storage_backend_api::gateway_mount_business_with_config(
        pool,
        admin_storage_config,
    )
    .await;
    BusinessRouterAssembly {
        router: application.router.merge(admin_storage),
    }
}

pub async fn assemble_business_routes_from_env() -> Result<BusinessRouterAssembly, String> {
    sdkwork_drive_security::ensure_drive_auth_policy_refresh_task();
    ensure_production_download_token_signing_configured()
        .map_err(|error| format!("download token signing config invalid: {error}"))?;
    let database_config = DatabaseConfig::from_env()
        .map_err(|error| format!("resolve drive database config failed: {error}"))?;
    let pool = connect_postgres_database_and_install_schema(&database_config)
        .await
        .map_err(|error| format!("create drive database pool failed: {error}"))?;
    ensure_domain_outbox_dispatcher(pool.clone());
    let admin_storage_config =
        sdkwork_routes_storage_backend_api::AdminStorageConfig::from_env()
            .map_err(|error| format!("resolve admin storage config failed: {error}"))?;
    Ok(assemble_business_routes_with_config(pool, admin_storage_config).await)
}

/// Builds the raw Drive App API for a gateway-owned Web Framework layer.
pub async fn assemble_app_api_contribution() -> Result<ApiAssemblyContribution, String> {
    sdkwork_drive_security::ensure_drive_auth_policy_refresh_task();
    ensure_production_download_token_signing_configured()
        .map_err(|error| format!("download token signing config invalid: {error}"))?;
    let database_config = DatabaseConfig::from_env()
        .map_err(|error| format!("resolve drive database config failed: {error}"))?;
    let pool = connect_postgres_database_and_install_schema(&database_config)
        .await
        .map_err(|error| format!("create drive database pool failed: {error}"))?;
    ensure_domain_outbox_dispatcher(pool.clone());

    let route_manifest = sdkwork_routes_drive_app_api::app_route_manifest();
    let router = sdkwork_routes_drive_app_api::build_app_business_router(pool.clone());
    ApiAssemblyContribution::from_manifest(
        "sdkwork-drive",
        "SDKWork Drive App API",
        router,
        route_manifest,
        vec![sdkwork_routes_drive_app_api::drive_app_context_injector()],
        Arc::new(sdkwork_drive_http::infra::PostgresReadinessCheck::new(pool)),
    )
}

pub async fn assemble_business_routes_with_process_pool(
    process_pool: &sdkwork_database_sqlx::DatabasePool,
) -> Result<BusinessRouterAssembly, String> {
    sdkwork_drive_security::ensure_drive_auth_policy_refresh_task();
    ensure_production_download_token_signing_configured()
        .map_err(|error| format!("download token signing config invalid: {error}"))?;
    bootstrap_drive_database(process_pool.clone()).await?;
    let pool = postgres_pool_from_database_pool(process_pool)?;
    ensure_domain_outbox_dispatcher(pool.clone());
    let admin_storage_config =
        sdkwork_routes_storage_backend_api::AdminStorageConfig::from_env()
            .map_err(|error| format!("resolve admin storage config failed: {error}"))?;
    Ok(assemble_business_routes_with_config(pool, admin_storage_config).await)
}

pub async fn assemble_backend_business_router_from_env() -> Result<BusinessRouterAssembly, String> {
    sdkwork_drive_security::ensure_drive_auth_policy_refresh_task();
    ensure_production_download_token_signing_configured()
        .map_err(|error| format!("download token signing config invalid: {error}"))?;
    let database_config = DatabaseConfig::from_env()
        .map_err(|error| format!("resolve drive database config failed: {error}"))?;
    let pool = connect_postgres_database_and_install_schema(&database_config)
        .await
        .map_err(|error| format!("create drive database pool failed: {error}"))?;
    ensure_domain_outbox_dispatcher(pool.clone());
    let admin_storage_config =
        sdkwork_routes_storage_backend_api::AdminStorageConfig::from_env()
            .map_err(|error| format!("resolve admin storage config failed: {error}"))?;
    let drive_backend =
        sdkwork_routes_drive_backend_api::gateway_mount_business(pool.clone()).await;
    let admin_storage = sdkwork_routes_storage_backend_api::gateway_mount_business_with_config(
        pool,
        admin_storage_config,
    )
    .await;
    Ok(BusinessRouterAssembly {
        router: drive_backend.merge(admin_storage),
    })
}

/// Drive Admin Storage backend surface for a composing gateway that owns the
/// Web Framework layer (same-origin dependency composition, API_ASSEMBLY_SPEC
/// §3/§6.1). Mirrors `sdkwork-api-rtc-assembly::assemble_backend_api_contribution_with_pool`:
/// the drive database module is bootstrapped on the shared pool, and the
/// storage business router derives its request context from the host web
/// framework instead of drive-owned domain injectors.
pub async fn assemble_backend_admin_storage_contribution_with_pool(
    pool: &sdkwork_database_sqlx::DatabasePool,
) -> Result<ApiAssemblyContribution, String> {
    bootstrap_drive_database(pool.clone()).await?;
    let pg_pool = postgres_pool_from_database_pool(pool)?;
    let admin_storage_config =
        sdkwork_routes_storage_backend_api::AdminStorageConfig::from_env()
            .map_err(|error| format!("resolve admin storage config failed: {error}"))?;
    let router =
        sdkwork_routes_storage_backend_api::build_admin_storage_business_router_for_host_framework(
            pg_pool.clone(),
            admin_storage_config,
        );
    ApiAssemblyContribution::from_manifest(
        "sdkwork-drive",
        "SDKWork Drive Admin Storage API",
        router,
        sdkwork_routes_storage_backend_api::storage_route_manifest(),
        Vec::new(),
        Arc::new(PostgresReadinessCheck::new(pg_pool)),
    )
}

pub async fn assemble_api_router(pool: sqlx::PgPool) -> Result<ApiAssembly, String> {
    sdkwork_drive_security::ensure_drive_auth_policy_refresh_task();
    ensure_production_download_token_signing_configured()
        .map_err(|error| format!("download token signing config invalid: {error}"))?;
    ensure_domain_outbox_dispatcher(pool.clone());

    let admin_storage_config =
        sdkwork_routes_storage_backend_api::AdminStorageConfig::from_env()
            .map_err(|error| format!("resolve admin storage config failed: {error}"))?;
    let router = Router::new()
        .merge(sdkwork_routes_drive_app_api::build_app_business_router(
            pool.clone(),
        ))
        .merge(sdkwork_routes_drive_backend_api::build_backend_business_router(pool.clone()))
        .merge(sdkwork_routes_drive_internal_api::build_internal_business_router(pool.clone()))
        .merge(sdkwork_routes_drive_open_api::build_open_business_router(
            pool.clone(),
        ))
        .merge(
            sdkwork_routes_storage_backend_api::build_admin_storage_business_router(
                pool.clone(),
                admin_storage_config,
            ),
        );
    let routes = sdkwork_routes_drive_app_api::gateway_route_manifest()
        .routes()
        .iter()
        .chain(
            sdkwork_routes_drive_backend_api::gateway_route_manifest()
                .routes()
                .iter(),
        )
        .chain(
            sdkwork_routes_drive_internal_api::gateway_route_manifest()
                .routes()
                .iter(),
        )
        .chain(
            sdkwork_routes_drive_open_api::gateway_route_manifest()
                .routes()
                .iter(),
        )
        .chain(
            sdkwork_routes_storage_backend_api::gateway_route_manifest()
                .routes()
                .iter(),
        )
        .cloned()
        .collect();

    ApiAssemblyContribution::from_manifest(
        "sdkwork-drive",
        "SDKWork Drive API",
        router,
        HttpRouteManifest::from_owned_routes(routes),
        vec![sdkwork_routes_drive_app_api::drive_app_context_injector()],
        Arc::new(PostgresReadinessCheck::new(pool)),
    )
}

pub async fn assemble_api_router_from_env() -> Result<ApiAssembly, String> {
    let database_config = DatabaseConfig::from_env()
        .map_err(|error| format!("resolve drive database config failed: {error}"))?;
    let pool = connect_postgres_database_and_install_schema(&database_config)
        .await
        .map_err(|error| format!("create drive database pool failed: {error}"))?;
    assemble_api_router(pool).await
}
