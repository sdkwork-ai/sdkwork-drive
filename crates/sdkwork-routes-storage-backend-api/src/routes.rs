use crate::auth::drive_context_projection_guard;
use crate::config::AdminStorageConfig;
use crate::state::AdminStorageState;
use crate::web_bootstrap::wrap_router_with_web_framework;
use axum::middleware;
use axum::{Extension, Router};
use sdkwork_drive_config::DatabaseConfig;
use sdkwork_drive_http::infra::{drive_service_router_config, mount_drive_infra_routes};
use sdkwork_drive_workspace_service::infrastructure::sql::connect_postgres_database_and_install_schema;
use sqlx::PgPool;

pub fn build_router_with_pool(pool: PgPool) -> Router {
    build_router_with_pool_and_config(pool, AdminStorageConfig::default())
}

pub fn build_router_with_pool_and_config(pool: PgPool, config: AdminStorageConfig) -> Router {
    let router = build_router_with_state(AdminStorageState::new(pool, config), true);
    wrap_router_with_web_framework(
        sdkwork_web_core::DefaultWebRequestContextResolver::default(),
        router,
    )
}

pub async fn build_protected_router_with_pool_and_config(
    pool: PgPool,
    config: AdminStorageConfig,
) -> Router {
    let router = build_router_inner_with_infra(AdminStorageState::new(pool, config), true, None);
    crate::web_bootstrap::wrap_router_with_web_framework_from_env(router).await
}

/// Business-only admin storage router for multi-surface assembly.
pub async fn gateway_mount_business(pool: PgPool) -> Router {
    gateway_mount_business_with_config(pool, AdminStorageConfig::default()).await
}

/// Business-only admin storage router with explicit provider configuration.
pub async fn gateway_mount_business_with_config(
    pool: PgPool,
    config: AdminStorageConfig,
) -> Router {
    build_admin_storage_business_router(pool, config)
}

/// Raw admin-storage router for a composing gateway that owns the Web Framework layer.
pub fn build_admin_storage_business_router(pool: PgPool, config: AdminStorageConfig) -> Router {
    build_business_router_inner(AdminStorageState::new(pool, config), true, None)
}

/// Raw admin-storage router for a composing gateway that owns the Web
/// Framework layer but does not register drive-specific domain injectors
/// (same-origin dependency composition, API_ASSEMBLY_SPEC §3/§6.1): the drive
/// request context is derived from the host-injected `WebRequestContext` by a
/// middleware inside this router instead.
pub fn build_admin_storage_business_router_for_host_framework(
    pool: PgPool,
    config: AdminStorageConfig,
) -> Router {
    build_business_router_inner(AdminStorageState::new(pool, config), true, None).route_layer(
        middleware::from_fn(crate::web_bootstrap::derive_drive_storage_context_from_web_context),
    )
}

pub fn build_router_with_pool_and_iam(pool: PgPool) -> Router {
    build_router_with_pool_and_config(pool, AdminStorageConfig::default())
}

pub fn build_router_with_pool_without_iam(pool: PgPool) -> Router {
    build_router_with_state(
        AdminStorageState::new(pool, AdminStorageConfig::default()),
        false,
    )
}

pub fn build_router_with_pool_without_iam_and_test_tenant(
    pool: PgPool,
    tenant_id: impl Into<String>,
) -> Router {
    build_router_with_state_and_test_tenant(
        AdminStorageState::new(pool, AdminStorageConfig::default()),
        tenant_id.into(),
    )
}

pub fn build_router_with_pool_config_without_iam(
    pool: PgPool,
    config: AdminStorageConfig,
) -> Router {
    build_router_with_state(AdminStorageState::new(pool, config), false)
}

pub async fn build_router_with_database_url(database_url: &str) -> Result<Router, sqlx::Error> {
    let config = DatabaseConfig::from_url(database_url)
        .map_err(|error| sqlx::Error::Configuration(Box::new(error)))?;
    let admin_storage_config =
        AdminStorageConfig::from_env().map_err(|error| sqlx::Error::Configuration(error.into()))?;
    build_router_with_database_url_config_parts(&config, admin_storage_config).await
}

pub async fn build_router_with_database_url_and_admin_storage_config(
    database_url: &str,
    admin_storage_config: AdminStorageConfig,
) -> Result<Router, sqlx::Error> {
    let config = DatabaseConfig::from_url(database_url)
        .map_err(|error| sqlx::Error::Configuration(Box::new(error)))?;
    build_router_with_database_url_config_parts(&config, admin_storage_config).await
}

pub async fn build_router_with_database_config(
    config: &DatabaseConfig,
) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    let admin_storage_config =
        AdminStorageConfig::from_env().map_err(admin_storage_config_error)?;
    build_router_with_database_config_and_admin_storage_config(config, admin_storage_config).await
}

pub async fn build_router_with_database_config_and_admin_storage_config(
    config: &DatabaseConfig,
    admin_storage_config: AdminStorageConfig,
) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    let pool = connect_postgres_database_and_install_schema(config).await?;
    Ok(build_protected_router_with_pool_and_config(pool, admin_storage_config).await)
}

async fn build_router_with_database_url_config_parts(
    config: &DatabaseConfig,
    admin_storage_config: AdminStorageConfig,
) -> Result<Router, sqlx::Error> {
    let pool = connect_postgres_database_and_install_schema(config).await?;
    Ok(build_protected_router_with_pool_and_config(pool, admin_storage_config).await)
}

fn admin_storage_config_error(error: String) -> Box<dyn std::error::Error + Send + Sync> {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, error).into()
}

fn build_router_with_state(state: AdminStorageState, require_iam: bool) -> Router {
    if require_iam {
        build_router_inner_with_infra(state, true, None)
    } else {
        build_router_inner_with_infra(
            state,
            false,
            Some(crate::app_context::default_test_drive_request_context()),
        )
    }
}

fn build_router_with_state_and_test_tenant(state: AdminStorageState, tenant_id: String) -> Router {
    build_router_inner_with_infra(
        state,
        false,
        Some(crate::app_context::test_drive_request_context_with_tenant(
            tenant_id,
        )),
    )
}

fn build_business_router_inner(
    state: AdminStorageState,
    require_iam: bool,
    test_context: Option<crate::app_context::DriveRequestContext>,
) -> Router {
    let mut drive_routes = crate::route_paths::storage_drive_routes("/admin/v3/api")
        .merge(crate::route_paths::storage_drive_routes("/backend/v3/api"))
        .route_layer(middleware::from_fn(
            crate::rate_limit::admin_storage_api_rate_limit,
        ));

    if require_iam {
        drive_routes = drive_routes
            .route_layer(middleware::from_fn(
                sdkwork_drive_http::problem_correlation::problem_correlation_middleware,
            ))
            .route_layer(middleware::from_fn(drive_context_projection_guard));
    }

    let router = Router::new()
        .merge(drive_routes)
        .layer(middleware::from_fn(
            sdkwork_drive_http::metrics::record_request_metrics,
        ))
        .with_state(state);

    if !require_iam {
        if let Some(ctx) = test_context {
            return router.layer(Extension(ctx));
        }
    }

    router
}

fn build_router_inner_with_infra(
    state: AdminStorageState,
    require_iam: bool,
    test_context: Option<crate::app_context::DriveRequestContext>,
) -> Router {
    let pool = state.pool.clone();
    mount_drive_infra_routes(
        build_business_router_inner(state, require_iam, test_context),
        drive_service_router_config(&pool),
        Some("sdkwork-drive-admin-storage-api"),
    )
}
