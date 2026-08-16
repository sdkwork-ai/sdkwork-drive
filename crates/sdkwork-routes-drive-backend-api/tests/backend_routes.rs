use axum::body::Body;
use http::{Method, Request, StatusCode};
use sdkwork_routes_drive_backend_api::build_router_with_pool_and_iam;
use sdkwork_web_core::RateLimitTier;
use tower::util::ServiceExt;

fn build_router() -> axum::Router {
    build_router_with_pool_and_iam(sdkwork_drive_test_support::lazy_postgres_test_pool())
}

#[tokio::test]
async fn backend_router_exposes_operations_and_quota_routes() {
    let app = build_router();
    let pool_app = sdkwork_routes_drive_backend_api::build_router_with_pool(
        sdkwork_drive_test_support::lazy_postgres_test_pool(),
    );

    let quota_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/quotas")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("quota request should be handled");
    assert_ne!(quota_response.status(), StatusCode::NOT_FOUND);

    let audit_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/audit_events")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("audit events request should be handled");
    assert_ne!(audit_response.status(), StatusCode::NOT_FOUND);

    let storage_response = pool_app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/storage_providers")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("legacy storage provider request should be handled");
    assert_eq!(
        storage_response.status(),
        StatusCode::NOT_FOUND,
        "legacy flat storage provider routes must not remain on drive-backend-api"
    );

    let maintenance_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/maintenance/object_sweep")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"dryRun": true, "limit": 1}"#))
                .expect("request should be built"),
        )
        .await
        .expect("maintenance request should be handled");
    assert_ne!(maintenance_response.status(), StatusCode::NOT_FOUND);

    for uri in [
        "/backend/v3/api/drive/maintenance/expired_upload_content_sweep",
        "/backend/v3/api/drive/maintenance/abandoned_upload_task_sweep",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"nowEpochMs":1800000000000,"dryRun":true,"limit":1}"#,
                    ))
                    .expect("request should be built"),
            )
            .await
            .unwrap_or_else(|error| panic!("{uri} should be handled: {error}"));
        assert_ne!(
            response.status(),
            StatusCode::NOT_FOUND,
            "{uri} must be routed"
        );
    }
}

#[test]
fn sandbox_admin_manifest_rate_limits_high_risk_mutations() {
    let manifest = sdkwork_routes_drive_backend_api::backend_route_manifest();
    for operation_id in [
        "sandboxVolumes.create",
        "sandboxVolumes.update",
        "sandboxVolumes.delete",
        "sandboxGrants.create",
        "sandboxGrants.update",
        "sandboxGrants.delete",
    ] {
        let route = manifest
            .routes()
            .iter()
            .find(|route| route.operation_id == operation_id)
            .unwrap_or_else(|| panic!("sandbox admin route should exist: {operation_id}"));
        assert_eq!(route.rate_limit_tier, Some(RateLimitTier::AuthCritical));
    }
    for operation_id in [
        "sandboxVolumes.list",
        "sandboxVolumes.retrieve",
        "sandboxGrants.list",
        "sandboxGrants.retrieve",
    ] {
        let route = manifest
            .routes()
            .iter()
            .find(|route| route.operation_id == operation_id)
            .unwrap_or_else(|| panic!("sandbox admin route should exist: {operation_id}"));
        assert_eq!(route.rate_limit_tier, None);
    }
}
