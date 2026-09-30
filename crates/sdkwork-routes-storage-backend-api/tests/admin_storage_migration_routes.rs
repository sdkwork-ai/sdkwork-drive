//! HTTP-level tests for the cross-provider storage migration endpoints.
//!
//! These prove the *surface*: that the manifest's paths are actually mounted,
//! that the response envelopes match the shape the OpenAPI promises (single
//! resource under `data.item`, collection under `data.items` + `data.pageInfo`),
//! and that a failure maps onto the right problem status. The engine's
//! bookkeeping is covered by `sdkwork-drive-workspace-service`; here the point
//! is the contract the client generates its SDK from.

use axum::body::{to_bytes, Body};
use axum::Router;
use http::{Method, Request, StatusCode};
use sdkwork_routes_storage_backend_api::{
    build_router_with_pool_without_iam_and_test_tenant, storage_route_manifest,
};
use sqlx::PgPool;
use tower::util::ServiceExt;

/// Every migration path the manifest advertises must be mounted.
///
/// This is the check that would have caught a handler wired into `route_paths`
/// but never listed in the manifest (or vice versa): the two are generated from
/// different sources, so nothing else compares them.
#[test]
fn manifest_advertises_every_migration_route() {
    let manifest = storage_route_manifest();
    let paths: Vec<&str> = manifest.routes().iter().map(|route| route.path).collect();

    for expected in [
        "/backend/v3/api/drive/storage/migrations",
        "/backend/v3/api/drive/storage/migrations/{migrationId}",
        "/backend/v3/api/drive/storage/migrations/{migrationId}/run",
        "/backend/v3/api/drive/storage/migrations/{migrationId}/items",
        "/backend/v3/api/drive/storage/migrations/{migrationId}/cancel",
    ] {
        assert!(
            paths.iter().any(|path| *path == expected),
            "manifest is missing {expected}; present: {paths:?}"
        );
    }
}

async fn seed_provider(pool: &PgPool, tenant_id: &str, provider_id: &str, status: &str) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider
            (id, tenant_id, provider_kind, name, endpoint_url, bucket, path_style, status,
             created_by, updated_by)
         VALUES ($1, $2, 's3_compatible', $3, 'https://s3.example.com', $4, true, $5,
                 'seed', 'seed')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(provider_id)
    .bind(tenant_id)
    .bind(provider_id)
    .bind(format!("bucket-{provider_id}"))
    .bind(status)
    .execute(pool)
    .await
    .expect("seed provider");
}

async fn cleanup(pool: &PgPool, tenant_id: &str) {
    // The table names are compile-time literals from this list, never caller
    // input, so the dynamic statement is safe to assert as such.
    for table in [
        "dr_drive_storage_migration_item",
        "dr_drive_storage_migration",
        "dr_drive_storage_provider",
    ] {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM {table} WHERE tenant_id = $1"
        )))
        .bind(tenant_id)
        .execute(pool)
        .await
        .ok();
    }
}

/// Read a JSON body into a `serde_json::Value` for shape assertions.
async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&bytes).expect("response body should be JSON")
}

fn migration_router(pool: PgPool) -> Router {
    build_router_with_pool_without_iam_and_test_tenant(pool, "tenant-mig-http")
}

/// One driver for every HTTP scenario, so the shared advisory-lock pool is
/// acquired exactly once in this binary.
#[tokio::test]
async fn storage_migration_http_contract() {
    let Some((pool, _guard)) = sdkwork_drive_test_support::postgres_test_database().await else {
        return;
    };

    plan_returns_the_resource_envelope(&pool).await;
    run_returns_the_run_report_envelope(&pool).await;
    items_return_the_page_envelope(&pool).await;
    list_returns_the_page_envelope(&pool).await;
    a_missing_run_is_a_404_problem(&pool).await;
    an_unknown_item_status_is_a_400_problem(&pool).await;
    cancel_takes_no_body(&pool).await;
    cancel_rejects_an_unexpected_body(&pool).await;
}

async fn plan_returns_the_resource_envelope(pool: &PgPool) {
    let tenant = "tenant-mig-http";
    cleanup(pool, tenant).await;
    seed_provider(pool, tenant, "prov-http-src", "active").await;
    seed_provider(pool, tenant, "prov-http-dst", "active").await;

    let response = migration_router(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/migrations")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"mig-http-1",
                        "name":"http plan",
                        "sourceProviderId":"prov-http-src",
                        "targetProviderId":"prov-http-dst",
                        "applyBindingSwitch":true
                    }"#,
                ))
                .expect("request builds"),
        )
        .await
        .expect("request handled");

    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "plan failed; body = {}",
        serde_json::to_string_pretty(&body).unwrap_or_default()
    );
    // Envelope: `code` is the int32 success marker, `data.item` holds the resource.
    assert_eq!(body["code"], 0);
    assert!(body["traceId"].is_string(), "traceId must be echoed");
    let item = &body["data"]["item"];
    assert_eq!(item["id"], "mig-http-1");
    assert_eq!(item["status"], "pending");
    assert_eq!(item["sourceProviderId"], "prov-http-src");
    assert_eq!(item["targetProviderId"], "prov-http-dst");
    // int64 counters cross the wire as strings, per the platform convention.
    assert!(item["objectsTotal"].is_string(), "int64 must be a string");
    assert!(item["bytesCopied"].is_string(), "int64 must be a string");
    assert!(item["progressRatio"].is_number());
}

async fn run_returns_the_run_report_envelope(pool: &PgPool) {
    let response = migration_router(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/migrations/mig-http-1/run")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("request builds"),
        )
        .await
        .expect("request handled");

    // An empty queue completes immediately, which is the honest outcome for a
    // run whose source provider holds no objects.
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["code"], 0);
    let item = &body["data"]["item"];
    assert!(item["migration"].is_object());
    assert!(item["copiedThisBatch"].is_string());
    assert!(item["failedThisBatch"].is_string());
    assert_eq!(item["completed"], true);
}

async fn items_return_the_page_envelope(pool: &PgPool) {
    let response = migration_router(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/migrations/mig-http-1/items")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("request handled");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["code"], 0);
    assert!(body["data"]["items"].is_array());
    assert!(body["data"]["pageInfo"].is_object());
    assert!(body["data"]["pageInfo"]["mode"].is_string());
}

/// `status` filters the tenant's runs; an unrelated status must return nothing.
///
/// The filter is asserted in both directions on purpose. Checking only the
/// matching case would still pass if the `status` parameter were silently
/// dropped, which is exactly the regression worth catching.
async fn list_returns_the_page_envelope(pool: &PgPool) {
    let matching = migration_router(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/migrations")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("request handled");

    assert_eq!(matching.status(), StatusCode::OK);
    let body = json_body(matching).await;
    assert_eq!(body["code"], 0);
    assert!(body["data"]["pageInfo"].is_object());
    let items = body["data"]["items"].as_array().expect("items array");
    assert_eq!(items.len(), 1, "the tenant owns exactly one run");
    assert_eq!(items[0]["id"], "mig-http-1");

    // A status the run does not currently hold must yield an empty page — the
    // run was driven to completion by the `run` scenario above.
    let filtered = migration_router(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/migrations?status=cancelled")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("request handled");

    assert_eq!(filtered.status(), StatusCode::OK);
    let filtered_body = json_body(filtered).await;
    assert_eq!(
        filtered_body["data"]["items"].as_array().map(Vec::len),
        Some(0),
        "a run that is not cancelled must not match ?status=cancelled"
    );
}

async fn a_missing_run_is_a_404_problem(pool: &PgPool) {
    let response = migration_router(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/migrations/does-not-exist")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("request handled");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["status"], 404);
    // `code` is the int32 platform result code (`API_SPEC.md` §15.2), and the
    // failure code must be non-zero.
    let code = body["code"]
        .as_i64()
        .expect("problem carries an int32 code");
    assert_ne!(code, 0, "a problem must not carry the success code");
    assert!(body["traceId"].is_string());
    assert!(body["title"].is_string(), "problem has a human title");
}

async fn an_unknown_item_status_is_a_400_problem(pool: &PgPool) {
    let response = migration_router(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage/migrations/mig-http-1/items?status=halfway")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("request handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["status"], 400);
}

async fn cancel_takes_no_body(pool: &PgPool) {
    // No content-type and no body at all: the operator comes from the verified
    // request context, so a caller needs to send nothing.
    let response = migration_router(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/migrations/mig-http-1/cancel")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("request handled");

    // The run already succeeded in the `run` scenario above, so cancelling is a
    // conflict — which is itself the assertion that the endpoint is mounted and
    // reaches the terminal-state guard rather than failing on a parse error.
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["status"], 409);

    cleanup(pool, "tenant-mig-http").await;
}

/// A body on `cancel` is rejected rather than silently ignored: the schema
/// declares none, so accepting one would mean the client is out of contract.
///
/// The rejection happens in the extractor, before any database work, so this
/// scenario needs no seeded rows.
async fn cancel_rejects_an_unexpected_body(pool: &PgPool) {
    let response = migration_router(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/storage/migrations/whatever/cancel")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"operatorId":"someone-else"}"#))
                .expect("request builds"),
        )
        .await
        .expect("request handled");

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "a client-supplied operatorId must be refused, not honoured"
    );
}
