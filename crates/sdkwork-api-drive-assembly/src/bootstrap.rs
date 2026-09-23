//! Authored API assembly bootstrap for SDKWork Drive.
//!
//! Drive has provider-specific startup and admin-storage configuration, so this file is
//! intentionally preserved by the assembly materializer. Business surfaces mount shared
//! infrastructure exactly once at the assembly boundary.

use sdkwork_web_bootstrap::WebModule;
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

    if let Some(config) = sdkwork_routes_drive_app_api::deploy_sandbox_config_from_env() {
        sdkwork_routes_drive_app_api::ensure_deploy_sandbox_volume(&pool, &config)
            .await
            .map_err(|error| format!("ensure deploy sandbox volume failed: {error}"))?;
    }

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

/// Same-origin embedding surface for a composing host gateway
/// (API_ASSEMBLY_SPEC §4.1.1).
///
/// An embedding host serves the surfaces drive declares as same-origin — the
/// App API (`/app/v3/api/drive/*`) and the Admin Storage backend surface
/// (`/backend/v3/api/drive/storage/*`) — and must install them as **one**
/// contribution. `ComposedApiAssembly::try_compose` rejects an owner selected
/// more than once, so a host that fetched the two surfaces from two entrypoints
/// cannot compose them at all, while a host that installed them as two modules
/// has the second silently dropped by `ApiModuleRegistry` and serves a partial
/// route surface (an opaque 404 on a surface its component contract declares as
/// served).
///
/// The contribution is built on the caller's process-shared PostgreSQL pool:
/// the drive database module is bootstrapped on that pool, and the storage
/// router derives its request context from the host Web Framework layer
/// (API_ASSEMBLY_SPEC §3/§6.1) instead of a drive-owned storage injector. The
/// App surfaces keep their own context injector, which only enriches the
/// request extensions and never rejects a request.
pub async fn assemble_same_origin_contribution_with_pool(
    pool: &sdkwork_database_sqlx::DatabasePool,
) -> Result<ApiAssemblyContribution, String> {
    sdkwork_drive_security::ensure_drive_auth_policy_refresh_task();
    ensure_production_download_token_signing_configured()
        .map_err(|error| format!("download token signing config invalid: {error}"))?;
    bootstrap_drive_database(pool.clone()).await?;
    let pg_pool = postgres_pool_from_database_pool(pool)?;
    ensure_domain_outbox_dispatcher(pg_pool.clone());

    if let Some(config) = sdkwork_routes_drive_app_api::deploy_sandbox_config_from_env() {
        sdkwork_routes_drive_app_api::ensure_deploy_sandbox_volume(&pg_pool, &config)
            .await
            .map_err(|error| format!("ensure deploy sandbox volume failed: {error}"))?;
    }

    let admin_storage_config =
        sdkwork_routes_storage_backend_api::AdminStorageConfig::from_env()
            .map_err(|error| format!("resolve admin storage config failed: {error}"))?;
    let router = sdkwork_routes_drive_app_api::build_app_business_router(pg_pool.clone()).merge(
        sdkwork_routes_storage_backend_api::build_admin_storage_business_router_for_host_framework(
            pg_pool.clone(),
            admin_storage_config,
        ),
    );
    let routes = sdkwork_routes_drive_app_api::app_route_manifest()
        .routes()
        .iter()
        .chain(
            sdkwork_routes_storage_backend_api::storage_route_manifest()
                .routes()
                .iter(),
        )
        .cloned()
        .collect();

    ApiAssemblyContribution::from_manifest(
        "sdkwork-drive",
        "SDKWork Drive Same-Origin API",
        router,
        HttpRouteManifest::from_owned_routes(routes),
        vec![sdkwork_routes_drive_app_api::drive_app_context_injector()],
        Arc::new(PostgresReadinessCheck::new(pg_pool)),
    )
}

/// Runs the Drive database lifecycle on the caller's process-shared pool
/// **without** mounting routes or resolving the admin-storage configuration.
///
/// This is the Drive half of the **explicit migration** path
/// (DATABASE_FRAMEWORK_SPEC §4.4.1). The serve path converges Drive inside
/// [`assemble_same_origin_contribution_with_pool`], which also builds routers and
/// reads `AdminStorageConfig` — a standalone gateway's `db-migrate` subcommand
/// must not pay for either, so it needs this route-free entrypoint.
///
/// Closing this entrypoint is what makes the drift repair instruction honest:
/// every module the serve path converges must also be converged by the explicit
/// migration command, otherwise a drifted Drive schema fails boot while naming a
/// command that cannot repair it — a permanent outage with a wrong fix
/// (DATABASE_FRAMEWORK_SPEC §4.4.1).
///
/// Whether forward migrations are applied stays governed by
/// `SDKWORK_DATABASE_AUTO_MIGRATE`, falling back to the Drive module manifest;
/// the migration command is what turns it on for its own process.
pub async fn ensure_database_lifecycle_with_pool(
    pool: sdkwork_database_sqlx::DatabasePool,
) -> Result<(), String> {
    bootstrap_drive_database(pool).await.map(|_| ())
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

/// Canonical Web Module definition for this application
/// (API_ASSEMBLY_SPEC §4.1.1): the complete HTTP surface — every route,
/// manifest, and OpenAPI document of this owner — as one installable module.
pub async fn web_module() -> Result<WebModule, String> {
    Ok(WebModule::from_contribution(
        assemble_api_router_from_env().await?,
    ))
}

/// Same as [`web_module`] but composed on a caller-owned PostgreSQL pool
/// (platform gateways, API_ASSEMBLY_SPEC §4.1.1).
///
/// Drive's authoritative storage server is PostgreSQL-only, so the platform
/// cloud gateway hands it the process-shared PostgreSQL pool rather than
/// letting the module open a second connection pool from the environment.
pub async fn web_module_with_postgres_pool(pool: sqlx::PgPool) -> Result<WebModule, String> {
    Ok(WebModule::from_contribution(
        assemble_api_router(pool).await?,
    ))
}
