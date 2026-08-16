//! Infrastructure routes (`/healthz`, `/livez`, `/readyz`, `/metrics`) via sdkwork-web-bootstrap.

use std::sync::Arc;

use axum::extract::Request;
use axum::routing::get;
use axum::Router;
use sdkwork_web_bootstrap::{
    contract_fallback_handler, healthz_handler, livez_handler, mount_infra_routes, readyz_handler,
    ReadinessCheck, ReadinessFuture, ServiceRouterConfig,
};
use sqlx::PgPool;

/// Readiness probe for Drive PostgreSQL databases.
#[derive(Clone)]
pub struct PostgresReadinessCheck {
    pool: PgPool,
}

impl PostgresReadinessCheck {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ReadinessCheck for PostgresReadinessCheck {
    fn check(&self) -> ReadinessFuture<'_> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("SELECT 1")
                .execute(&pool)
                .await
                .map_err(|error| error.to_string())?;
            Ok(())
        })
    }
}

/// Builds a [`ServiceRouterConfig`] with database readiness for the given pool.
pub fn drive_service_router_config(pool: &PgPool) -> ServiceRouterConfig {
    ServiceRouterConfig::default()
        .with_readiness_check(Arc::new(PostgresReadinessCheck::new(pool.clone())))
}

/// Mounts standard infrastructure routes on `router`.
///
/// When `service_name` is provided, `/metrics` renders Drive observability counters and
/// histograms. Otherwise the web-bootstrap default registry is used.
pub fn mount_drive_infra_routes<S>(
    router: Router<S>,
    config: ServiceRouterConfig,
    service_name: Option<&'static str>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    match service_name {
        Some(service_name) => mount_drive_observability_infra_routes(router, config, service_name),
        None => mount_infra_routes(router, config),
    }
}

fn mount_drive_observability_infra_routes<S>(
    router: Router<S>,
    config: ServiceRouterConfig,
    service_name: &'static str,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let readiness = config.readiness;
    let mut router = router
        .route("/healthz", get(healthz_handler))
        .route("/livez", get(livez_handler))
        .route(
            "/readyz",
            get(move || {
                let readiness = readiness.clone();
                async move { readyz_handler(readiness).await }
            }),
        )
        .route(
            "/metrics",
            get(move || async move { crate::metrics::metrics_handler(service_name).await }),
        );
    if let Some(fallback_config) = config.contract_fallback {
        router = router.fallback(move |request: Request| {
            let config = fallback_config.clone();
            async move { contract_fallback_handler(request, config).await }
        });
    }
    router
}

/// Canonical public infrastructure path prefixes for Drive web framework profiles.
pub fn drive_infra_public_path_prefixes() -> Vec<String> {
    sdkwork_web_bootstrap::infra_public_path_prefixes()
}
