use crate::handlers::{create_share_link_download_url, resolve_share_link};
use crate::rate_limit::share_link_rate_limit;
use crate::state::OpenState;
use crate::web_bootstrap::{
    wrap_router_with_dev_open_api_web_framework, wrap_router_with_web_framework_from_env,
};
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use sdkwork_drive_config::DatabaseConfig;
use sdkwork_drive_http::infra::{drive_service_router_config, mount_drive_infra_routes};
use sdkwork_drive_workspace_service::infrastructure::sql::connect_postgres_database_and_install_schema;
use sqlx::PgPool;

pub fn build_router_with_pool(pool: PgPool) -> Router {
    let router = build_router_with_state(OpenState::new(pool));
    wrap_router_with_dev_open_api_web_framework(router)
}

pub async fn build_protected_router_with_pool(pool: PgPool) -> Router {
    let router = build_router_with_state(OpenState::new(pool));
    wrap_router_with_web_framework_from_env(router).await
}

pub async fn build_router_with_database_url(database_url: &str) -> Result<Router, sqlx::Error> {
    let config = DatabaseConfig::from_url(database_url)
        .map_err(|error| sqlx::Error::Configuration(Box::new(error)))?;
    let pool = connect_postgres_database_and_install_schema(&config).await?;
    Ok(build_protected_router_with_pool(pool).await)
}

pub async fn build_router_with_database_config(
    config: &DatabaseConfig,
) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    let pool = connect_postgres_database_and_install_schema(config)
        .await
        .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync>)?;
    Ok(build_protected_router_with_pool(pool).await)
}

fn build_business_router_layers(state: OpenState) -> Router {
    let share_routes = Router::new()
        .route(
            "/open/v3/api/drive/share_links/{token}",
            get(resolve_share_link),
        )
        .route(
            "/open/v3/api/drive/share_links/{token}/download_url",
            post(create_share_link_download_url),
        )
        .route_layer(middleware::from_fn(share_link_rate_limit));

    Router::new()
        .merge(share_routes)
        .layer(middleware::from_fn(
            sdkwork_drive_http::problem_correlation::problem_correlation_middleware,
        ))
        .with_state(state)
}

fn build_router_with_state(state: OpenState) -> Router {
    let pool = state.pool.clone();
    mount_drive_infra_routes(
        build_business_router_layers(state),
        drive_service_router_config(&pool),
        Some("sdkwork-drive-open-api"),
    )
    .layer(middleware::from_fn(
        sdkwork_drive_http::metrics::record_request_metrics,
    ))
}

/// Business router for multi-surface gateway assembly (infra mounted once by assembly).
pub async fn gateway_mount_business(pool: PgPool) -> Router {
    build_open_business_router(pool)
}

/// Raw Open API router for a composing gateway that owns the Web Framework layer.
pub fn build_open_business_router(pool: PgPool) -> Router {
    let state = OpenState::new(pool);
    build_business_router_layers(state).layer(middleware::from_fn(
        sdkwork_drive_http::metrics::record_request_metrics,
    ))
}

/// Raw Open API router retained as an explicit assembly-oriented name.
pub async fn build_gateway_business_router_with_pool(pool: PgPool) -> Router {
    build_open_business_router(pool)
}

pub async fn gateway_mount(pool: sqlx::PgPool) -> axum::Router {
    build_protected_router_with_pool(pool).await
}
