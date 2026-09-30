//! End-to-end bootstrap coverage for the storage provider defaults.
//!
//! The failure these tests exist to prevent: `initialize_storage_provider_account_defaults`
//! gates every cloud vendor on `kind_is_enabled`, which reads
//! `dr_drive_storage_provider_kind`. On a plane whose catalog was never seeded
//! that table is empty, so every vendor took the `continue`, the response came
//! back 200 carrying only the local row, the console said "initialized", and not
//! one provider existed. Nothing errored, which is why it went unnoticed: the
//! fixture seeds the catalog, so the in-process suites always ran against a full
//! table.
//!
//! These tests delete the catalog first, so they reproduce the empty-catalog
//! plane the handler now has to repair on its own.

use axum::body::{to_bytes, Body};
use axum::Router;
use http::{Method, Request, StatusCode};
use sdkwork_drive_workspace_service::application::storage_provider_kind_service::{
    STORAGE_PROVIDER_KIND_TABLE, BUILTIN_STORAGE_PROVIDER_KIND_CATALOG,
};
use sdkwork_routes_storage_backend_api::build_router_with_pool_without_iam;
use sqlx::PgPool;
use tower::util::ServiceExt;

const BOOTSTRAP_PATH: &str = "/backend/v3/api/drive/storage/provider-account-defaults";

/// The 24 cloud vendors the bootstrap must create, i.e. the catalog minus the
/// credential-free `local_filesystem` entry.
fn expected_cloud_kind_count() -> usize {
    BUILTIN_STORAGE_PROVIDER_KIND_CATALOG
        .iter()
        .filter(|(kind, _, _)| *kind != "local_filesystem")
        .count()
}

async fn empty_the_kind_catalog(pool: &PgPool) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "DELETE FROM \"{STORAGE_PROVIDER_KIND_TABLE}\""
    )))
    .execute(pool)
    .await
    .expect("kind catalog should be emptied to reproduce a never-seeded plane");
}

async fn count_kinds(pool: &PgPool) -> i64 {
    sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(format!(
        "SELECT COUNT(1) FROM \"{STORAGE_PROVIDER_KIND_TABLE}\""
    )))
    .fetch_one(pool)
    .await
    .expect("count the kind catalog")
}

async fn count_providers(pool: &PgPool) -> i64 {
    // `dr_drive_storage_provider` has no soft-delete column beyond `status`; the
    // fixture truncates the table between runs, so a plain count is the live set.
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(1) FROM dr_drive_storage_provider WHERE status <> 'deleted'",
    )
    .fetch_one(pool)
    .await
    .expect("count storage providers")
}

fn bootstrap_app(pool: PgPool) -> Router {
    build_router_with_pool_without_iam(pool)
}

async fn post_bootstrap(app: Router) -> (StatusCode, serde_json::Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(BOOTSTRAP_PATH)
                .body(Body::empty())
                .expect("bootstrap request should be built"),
        )
        .await
        .expect("bootstrap request should be handled");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("bootstrap response body should be readable");
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

/// The list lives under the `data` envelope, i.e. `{"code":0,"data":{"items":[…]}}`.
/// Reading `items` off the root silently yields `None` and makes a passing
/// product look broken, so every assertion goes through this helper.
fn bootstrap_items(body: &serde_json::Value) -> &[serde_json::Value] {
    body.get("data")
        .and_then(|data| data.get("items"))
        .and_then(|items| items.as_array())
        .map(|items| items.as_slice())
        .unwrap_or_else(|| panic!("bootstrap response must carry data.items, got {body}"))
}

/// The whole point of the request: one call, from an empty catalog, must leave
/// every vendor initialized.
#[tokio::test]
async fn bootstrap_initializes_every_vendor_from_an_empty_catalog() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    empty_the_kind_catalog(&pool).await;
    assert_eq!(count_kinds(&pool).await, 0, "catalog must start empty");

    let (status, body) = post_bootstrap(bootstrap_app(pool.clone())).await;
    assert_eq!(status, StatusCode::OK, "bootstrap should succeed: {body}");

    // The catalog is repaired as part of the same call, so the plane is usable
    // without a second, separately-discovered endpoint.
    assert_eq!(
        count_kinds(&pool).await,
        BUILTIN_STORAGE_PROVIDER_KIND_CATALOG.len() as i64,
        "bootstrap must seed the full kind catalog"
    );

    // And every cloud vendor now has a provider row.
    let cloud_created = bootstrap_items(&body)
        .iter()
        .filter(|item| {
            item.get("providerKind").and_then(|kind| kind.as_str()) != Some("local_filesystem")
        })
        .count();
    assert_eq!(
        cloud_created,
        expected_cloud_kind_count(),
        "bootstrap must report every cloud vendor, got {body}"
    );

    let provider_rows = count_providers(&pool).await;
    assert!(
        provider_rows >= expected_cloud_kind_count() as i64,
        "bootstrap must persist a provider row per cloud vendor, found {provider_rows}"
    );
}

/// A vendor the operator disabled is a decision, not a gap: re-running the
/// bootstrap must not resurrect it. This is what makes the seed-on-bootstrap
/// change safe for existing planes.
#[tokio::test]
async fn bootstrap_does_not_reenable_a_vendor_the_operator_disabled() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    empty_the_kind_catalog(&pool).await;

    // Seed first (repairs the catalog), then disable one vendor as an operator would.
    let (first_status, _) = post_bootstrap(bootstrap_app(pool.clone())).await;
    assert_eq!(first_status, StatusCode::OK);

    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "UPDATE \"{STORAGE_PROVIDER_KIND_TABLE}\" SET enabled = FALSE WHERE provider_kind = 'wasabi'"
    )))
    .execute(&pool)
    .await
    .expect("disable wasabi");

    let (second_status, body) = post_bootstrap(bootstrap_app(pool.clone())).await;
    assert_eq!(second_status, StatusCode::OK, "re-run should succeed: {body}");

    let wasabi_enabled = sqlx::query_scalar::<_, bool>(sqlx::AssertSqlSafe(format!(
        "SELECT enabled FROM \"{STORAGE_PROVIDER_KIND_TABLE}\" WHERE provider_kind = 'wasabi'"
    )))
    .fetch_one(&pool)
    .await
    .expect("read wasabi enabled flag");
    assert!(
        !wasabi_enabled,
        "bootstrap must not flip an operator-disabled vendor back on"
    );

    let wasabi_reported = bootstrap_items(&body)
        .iter()
        .any(|item| item.get("providerKind").and_then(|k| k.as_str()) == Some("wasabi"));
    assert!(
        !wasabi_reported,
        "a disabled vendor must be absent from the bootstrap response"
    );
}

/// Re-running is safe: the second call must not duplicate rows or error.
#[tokio::test]
async fn bootstrap_is_idempotent_across_runs() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    empty_the_kind_catalog(&pool).await;

    let (first_status, _) = post_bootstrap(bootstrap_app(pool.clone())).await;
    assert_eq!(first_status, StatusCode::OK);
    let providers_after_first = count_providers(&pool).await;

    let (second_status, _) = post_bootstrap(bootstrap_app(pool.clone())).await;
    assert_eq!(second_status, StatusCode::OK);

    assert_eq!(
        count_providers(&pool).await,
        providers_after_first,
        "a second bootstrap must not create duplicate provider rows"
    );
}
