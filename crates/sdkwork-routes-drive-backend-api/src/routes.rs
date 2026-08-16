use crate::auth::drive_context_projection_guard;
use crate::handlers::*;
use crate::state::BackendState;
use crate::web_bootstrap::wrap_router_with_web_framework;
use axum::middleware;
use axum::routing::{get, post};
use axum::{Extension, Router};
use sdkwork_drive_config::DatabaseConfig;
use sdkwork_drive_http::infra::{drive_service_router_config, mount_drive_infra_routes};
use sdkwork_drive_security::DriveAppContext;
use sdkwork_drive_workspace_service::infrastructure::sql::connect_postgres_database_and_install_schema;
use sqlx::PgPool;

pub fn build_router_with_pool(pool: PgPool) -> Router {
    build_router_with_state(BackendState::new(pool), false)
        .layer(Extension(default_test_app_context()))
}

/// Constructs a `DriveAppContext` with well-known test claims.
///
/// `build_router_with_pool` is a test-only entrypoint that bypasses the IAM web
/// framework. Handlers still require `Extension<DriveAppContext>` to resolve the
/// authenticated tenant, so we inject a default context with `tenant-001` to keep
/// tests deterministic without requiring dual-token headers.
fn default_test_app_context() -> DriveAppContext {
    DriveAppContext {
        tenant_id: "tenant-001".to_string(),
        user_id: "user-001".to_string(),
        organization_id: Some("organization-001".to_string()),
        session_id: None,
        app_id: Some("appbase".to_string()),
        environment: Some("dev".to_string()),
        deployment_mode: Some("saas".to_string()),
        auth_level: Some("password".to_string()),
        data_scope: Vec::new(),
        permission_scope: vec![sdkwork_drive_security::DRIVE_STORAGE_ADMIN_PERMISSION.to_string()],
        actor_id: "user-001".to_string(),
        actor_kind: "user".to_string(),
        device_id: None,
        request_id: "request-test".to_string(),
        trace_id: "trace-test".to_string(),
    }
}

pub fn build_router_with_pool_and_iam(pool: PgPool) -> Router {
    let router = build_router_with_state(BackendState::new(pool), true);
    wrap_router_with_web_framework(
        sdkwork_web_core::DefaultWebRequestContextResolver::default(),
        router,
    )
}

pub async fn build_protected_router_with_pool(pool: PgPool) -> Router {
    let router = build_router_with_state(BackendState::new(pool), true);
    crate::web_bootstrap::wrap_router_with_web_framework_from_env(router).await
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

fn build_business_router_layers(state: BackendState, require_iam: bool) -> Router {
    let mut drive_routes = Router::new()
        .route(
            "/backend/v3/api/drive/labels",
            get(list_labels).post(create_label),
        )
        .route(
            "/backend/v3/api/drive/labels/{label_id}",
            get(get_label).patch(update_label).delete(delete_label),
        )
        .route("/backend/v3/api/drive/audit_events", get(list_audit_events))
        .route(
            "/backend/v3/api/drive/maintenance/object_sweep",
            post(sweep_object_store),
        )
        .route(
            "/backend/v3/api/drive/maintenance/upload_session_sweep",
            post(sweep_upload_sessions),
        )
        .route(
            "/backend/v3/api/drive/maintenance/expired_upload_content_sweep",
            post(sweep_expired_upload_content),
        )
        .route(
            "/backend/v3/api/drive/maintenance/abandoned_upload_task_sweep",
            post(sweep_abandoned_upload_tasks),
        )
        .route(
            "/backend/v3/api/drive/maintenance/jobs",
            get(list_maintenance_jobs),
        )
        .route(
            "/backend/v3/api/drive/download_packages",
            get(list_download_packages),
        )
        .route("/backend/v3/api/drive/spaces", get(list_spaces))
        .route(
            "/backend/v3/api/drive/sandbox_volumes",
            get(list_sandbox_volumes).post(create_sandbox_volume),
        )
        .route(
            "/backend/v3/api/drive/sandbox_volumes/{sandbox_id}",
            get(get_sandbox_volume)
                .patch(update_sandbox_volume)
                .delete(delete_sandbox_volume),
        )
        .route(
            "/backend/v3/api/drive/sandbox_volumes/{sandbox_id}/grants",
            get(list_sandbox_grants).post(create_sandbox_grant),
        )
        .route(
            "/backend/v3/api/drive/sandbox_volumes/{sandbox_id}/grants/{grant_id}",
            get(get_sandbox_grant)
                .patch(update_sandbox_grant)
                .delete(delete_sandbox_grant),
        )
        .route(
            "/backend/v3/api/drive/quotas",
            get(list_quotas).put(update_quota_policy),
        )
        .route_layer(middleware::from_fn(
            crate::rate_limit::backend_api_rate_limit,
        ));

    if require_iam {
        drive_routes =
            drive_routes.route_layer(middleware::from_fn(drive_context_projection_guard));
    }

    drive_routes = drive_routes.route_layer(middleware::from_fn(
        sdkwork_drive_http::problem_correlation::problem_correlation_middleware,
    ));

    Router::new().merge(drive_routes).with_state(state)
}

fn build_router_with_state(state: BackendState, require_iam: bool) -> Router {
    let pool = state.pool.clone();
    mount_drive_infra_routes(
        build_business_router_layers(state, require_iam),
        drive_service_router_config(&pool),
        Some("sdkwork-drive-backend-api"),
    )
    .layer(middleware::from_fn(
        sdkwork_drive_http::metrics::record_request_metrics,
    ))
}

/// Business router for multi-surface gateway assembly (infra mounted once by assembly).
pub async fn gateway_mount_business(pool: PgPool) -> Router {
    build_backend_business_router(pool)
}

/// Raw Backend API router for a composing gateway that owns the Web Framework layer.
pub fn build_backend_business_router(pool: PgPool) -> Router {
    let state = BackendState::new(pool);
    build_business_router_layers(state, true).layer(middleware::from_fn(
        sdkwork_drive_http::metrics::record_request_metrics,
    ))
}

/// Raw Backend API router retained as an explicit assembly-oriented name.
pub async fn build_gateway_business_router_with_pool(pool: PgPool) -> Router {
    build_backend_business_router(pool)
}

pub async fn gateway_mount(pool: sqlx::PgPool) -> axum::Router {
    build_protected_router_with_pool(pool).await
}
