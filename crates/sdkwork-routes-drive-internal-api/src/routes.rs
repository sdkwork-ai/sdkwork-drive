use axum::middleware;
use axum::routing::{get, post, put};
use axum::Router;
use sqlx::PgPool;

use crate::content::retrieve_drive_resource_content;
use crate::handlers::{
    create_root_scope_subscription, ensure_root_scope_event_delivery,
    ensure_website_root_event_delivery, resolve_drive_resource, retrieve_root_scope_subscription,
    retrieve_website_root,
};
use crate::state::InternalApiState;

fn business_router(state: InternalApiState) -> Router {
    Router::new()
        .route(
            "/internal/v3/api/drive/root_scope_subscriptions",
            post(create_root_scope_subscription),
        )
        .route(
            "/internal/v3/api/drive/root_scope_subscriptions/{subscriptionUuid}",
            get(retrieve_root_scope_subscription),
        )
        .route(
            "/internal/v3/api/drive/root_scope_subscriptions/{subscriptionUuid}/event_delivery",
            put(ensure_root_scope_event_delivery),
        )
        .route(
            "/internal/v3/api/drive/website_roots/{websiteRootUuid}",
            get(retrieve_website_root),
        )
        .route(
            "/internal/v3/api/drive/website_roots/{websiteRootUuid}/event_deliveries/{channelId}",
            put(ensure_website_root_event_delivery),
        )
        .route(
            "/internal/v3/api/drive/resource_resolutions",
            post(resolve_drive_resource),
        )
        .route(
            "/internal/v3/api/drive/node_versions/{nodeVersionId}/content",
            get(retrieve_drive_resource_content),
        )
        .layer(middleware::from_fn(
            sdkwork_drive_http::problem_correlation::problem_correlation_middleware,
        ))
        .with_state(state)
}

pub fn build_router_with_pool(pool: PgPool) -> Router {
    let router = business_router(InternalApiState::new(pool));
    crate::web_bootstrap::wrap_with_default_resolver(router).layer(middleware::from_fn(
        sdkwork_drive_http::metrics::record_request_metrics,
    ))
}

pub async fn build_protected_router_with_pool(pool: PgPool) -> Router {
    let router = business_router(InternalApiState::new(pool));
    crate::web_bootstrap::wrap_from_env(router)
        .await
        .layer(middleware::from_fn(
            sdkwork_drive_http::metrics::record_request_metrics,
        ))
}

pub async fn gateway_mount_business(pool: PgPool) -> Router {
    build_internal_business_router(pool)
}

/// Raw Internal API router for a composing gateway that owns the Web Framework layer.
pub fn build_internal_business_router(pool: PgPool) -> Router {
    business_router(InternalApiState::new(pool)).layer(middleware::from_fn(
        sdkwork_drive_http::metrics::record_request_metrics,
    ))
}

pub async fn gateway_mount(pool: PgPool) -> Router {
    build_protected_router_with_pool(pool).await
}
