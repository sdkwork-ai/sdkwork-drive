//! Regression: the federated Drive module must give the admin-storage routes a
//! `DriveRequestContext` even though the composing host owns the Web Framework
//! layer and registers only the Drive **app** domain injector.
//!
//! `assemble_drive_module` is the assembly every composing host uses
//! (`sdkwork-api-cloud-gateway` via `web_module_with_postgres_pool`, the Manager
//! standalone gateway and the Drive standalone gateway via `assemble_api_router`).
//! It used to mount the admin-storage business router through
//! `build_admin_storage_business_router`, which inserts no request context, while
//! the module's only registered injector is `drive_app_context_injector`
//! (a different `DriveRequestContext` type). Every
//! `/backend/v3/api/drive/storage/*` request therefore died at the handler's
//! `Extension<DriveRequestContext>` extractor. The host's Web Framework layer
//! then rewrote that bare 500 into the client-safe
//! `50001 / "An internal error occurred"` problem the Storage Center console
//! showed in production.
//!
//! The test composes the real module exactly like the host does and proves the
//! storage list resolves the tenant-scoped context and answers `200`.

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_bootstrap::ApiModuleRegistry;
use sdkwork_web_core::{
    encode_unsigned_test_jwt, DefaultWebRequestContextResolver, WebEnvironment,
    WebRequestContextProfile,
};
use serde_json::{json, Value};
use tower::util::ServiceExt;

const TEST_SESSION: &str = "session-context-regression";
const TEST_APP: &str = "appbase";

fn auth_token(tenant: &str, user: &str) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "auth",
        "tenant_id": tenant,
        "user_id": user,
        "session_id": TEST_SESSION,
        "app_id": TEST_APP,
        "organization_id": "100002",
        "auth_level": "password",
        "login_scope": "ORGANIZATION",
    }))
}

fn access_token(tenant: &str, user: &str) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "access",
        "tenant_id": tenant,
        "user_id": user,
        "session_id": TEST_SESSION,
        "app_id": TEST_APP,
        "organization_id": "100002",
        "environment": "test",
        "deployment_mode": "saas",
        "login_scope": "ORGANIZATION",
    }))
}

async fn seed_provider(pool: &sqlx::PgPool, tenant_id: &str, provider_id: &str) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, tenant_id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, provider_account_id, server_side_encryption_mode,
            default_storage_class, status, version, created_by, updated_by
        ) VALUES ($1, $2, 's3_compatible', 'context regression', 'https://s3.example.com',
                  'us-east-1', 'regression-bucket', true, true, 'plain:test:test', NULL, NULL,
                  NULL, 'active', 1, 'test', 'test')",
    )
    .bind(provider_id)
    .bind(tenant_id)
    .execute(pool)
    .await
    .expect("seed dr_drive_storage_provider");
}

#[tokio::test]
async fn federated_host_resolves_admin_storage_request_context() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_provider(&pool, "tenant-a", "provider-context-regression").await;

    // Exactly what `web_module_with_postgres_pool` installs into a composing
    // host: the federated Drive module and its single registered injector.
    let module = sdkwork_api_drive_assembly::web_module_with_postgres_pool(pool.clone())
        .await
        .expect("assemble federated Drive module");
    let mut registry = ApiModuleRegistry::new();
    registry.add_modules([module]);
    let composed = registry
        .try_compose("federated host")
        .expect("compose federated Drive module");

    let mut layer = WebFrameworkLayer::new(DefaultWebRequestContextResolver::default())
        .with_profile(WebRequestContextProfile {
            environment: WebEnvironment::Test,
            ..WebRequestContextProfile::default()
        })
        .with_route_manifest(composed.route_manifest.clone());
    for injector in &composed.domain_context_injectors {
        layer = layer.with_domain_injector(injector.clone());
    }
    let app = with_web_request_context(composed.router, layer);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/providers")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "admin-001")),
                )
                .header("access-token", access_token("tenant-a", "admin-001"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("storage provider list should be handled");

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read storage provider list body");
    assert_eq!(
        status,
        StatusCode::OK,
        "federated storage list must not fail the DriveRequestContext extractor: {}",
        String::from_utf8_lossy(&body)
    );

    let payload: Value = serde_json::from_slice(&body).expect("storage list body is JSON");
    assert_eq!(payload["code"], json!(0));
    let items = payload["data"]["items"]
        .as_array()
        .expect("storage list data.items is an array");
    let ids = items
        .iter()
        .map(|item| item["id"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec!["provider-context-regression"],
        "the derived context must scope the list to the caller's tenant"
    );
}
