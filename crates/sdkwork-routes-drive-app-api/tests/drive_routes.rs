use axum::body::{to_bytes, Body};
use http::{Method, Request, StatusCode};
use sdkwork_routes_drive_app_api::build_router_with_pool_and_iam;
use tower::util::ServiceExt;

mod common;

fn build_router() -> axum::Router {
    build_router_with_pool_and_iam(sdkwork_drive_test_support::lazy_postgres_test_pool())
}

#[tokio::test]
async fn app_router_exposes_dr_drive_space_and_upload_routes() {
    let app = build_router();

    let spaces_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("spaces request should be handled");
    assert_ne!(spaces_response.status(), StatusCode::NOT_FOUND);

    let upload_response = app
        .clone()
        .oneshot(
            Request::builder()
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        common::auth_token("tenant-001", "user-001", "appbase")
                    ),
                )
                .header(
                    "access-token",
                    common::access_token("tenant-001", "user-001", "appbase"),
                )
                .method(Method::POST)
                .uri("/app/v3/api/drive/upload_sessions")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("upload request should be handled");
    assert_ne!(upload_response.status(), StatusCode::NOT_FOUND);

    for (method, uri) in [
        (Method::GET, "/app/v3/api/drive/spaces/space-001/nodes"),
        (Method::POST, "/app/v3/api/drive/nodes/folders"),
        (Method::POST, "/app/v3/api/drive/nodes/files"),
        (Method::PATCH, "/app/v3/api/drive/nodes/node-001"),
        (Method::GET, "/app/v3/api/drive/nodes/node-001"),
        (Method::DELETE, "/app/v3/api/drive/nodes/node-001"),
        (Method::POST, "/app/v3/api/drive/nodes/node-001/move"),
        (Method::POST, "/app/v3/api/drive/nodes/node-001/copy"),
        (Method::POST, "/app/v3/api/drive/nodes/node-001/trash"),
        (Method::GET, "/app/v3/api/drive/nodes/node-001/download_url"),
        (Method::POST, "/app/v3/api/drive/trash/node-001/restore"),
        (Method::GET, "/app/v3/api/drive/trash?spaceId=space-001"),
        (Method::POST, "/app/v3/api/drive/trash/empty"),
        (Method::GET, "/app/v3/api/drive/recent?spaceId=space-001"),
        (Method::GET, "/app/v3/api/drive/shared_with_me"),
        (Method::GET, "/app/v3/api/drive/favorites"),
        (Method::GET, "/app/v3/api/drive/quotas/summary"),
        (Method::PUT, "/app/v3/api/drive/nodes/node-001/favorite"),
        (Method::DELETE, "/app/v3/api/drive/nodes/node-001/favorite"),
        (Method::GET, "/app/v3/api/drive/nodes/node-001/versions"),
        (
            Method::POST,
            "/app/v3/api/drive/nodes/node-001/versions/version-001/restore",
        ),
        (
            Method::GET,
            "/app/v3/api/drive/nodes/node-001/versions/version-001",
        ),
        (
            Method::DELETE,
            "/app/v3/api/drive/nodes/node-001/versions/version-001",
        ),
        (Method::GET, "/app/v3/api/drive/nodes/node-001/permissions"),
        (Method::POST, "/app/v3/api/drive/nodes/node-001/permissions"),
        (
            Method::PATCH,
            "/app/v3/api/drive/nodes/node-001/permissions/permission-001",
        ),
        (
            Method::GET,
            "/app/v3/api/drive/nodes/node-001/permissions/permission-001",
        ),
        (
            Method::DELETE,
            "/app/v3/api/drive/nodes/node-001/permissions/permission-001",
        ),
        (Method::GET, "/app/v3/api/drive/nodes/node-001/share_links"),
        (Method::POST, "/app/v3/api/drive/nodes/node-001/share_links"),
        (
            Method::PATCH,
            "/app/v3/api/drive/share_links/share-link-001",
        ),
        (Method::GET, "/app/v3/api/drive/share_links/share-link-001"),
        (
            Method::DELETE,
            "/app/v3/api/drive/share_links/share-link-001",
        ),
        (
            Method::POST,
            "/app/v3/api/drive/share_links/share-link-token/claim",
        ),
        (Method::GET, "/app/v3/api/drive/search?q=report"),
        (Method::GET, "/app/v3/api/drive/changes"),
        (Method::GET, "/app/v3/api/drive/upload_sessions/session-001"),
        (Method::POST, "/app/v3/api/drive/uploader/uploads"),
        (
            Method::PUT,
            "/app/v3/api/drive/uploader/uploads/upload-item-001/parts/1",
        ),
        (
            Method::PUT,
            "/app/v3/api/drive/upload_sessions/session-001/parts/1",
        ),
        (
            Method::POST,
            "/app/v3/api/drive/upload_sessions/session-001/complete",
        ),
        (
            Method::POST,
            "/app/v3/api/drive/upload_sessions/session-001/abort",
        ),
        (Method::GET, "/app/v3/api/assets"),
        (Method::POST, "/app/v3/api/assets"),
        (Method::GET, "/app/v3/api/assets/asset-001"),
        (Method::PATCH, "/app/v3/api/assets/asset-001"),
        (Method::POST, "/app/v3/api/assets/asset-001/archive"),
        (Method::POST, "/app/v3/api/assets/asset-001/restore"),
        (Method::GET, "/app/v3/api/assets/collections"),
        (Method::POST, "/app/v3/api/assets/collections"),
        (
            Method::POST,
            "/app/v3/api/assets/collections/collection-001/items",
        ),
        (
            Method::DELETE,
            "/app/v3/api/assets/collections/collection-001/items/item-001",
        ),
        (Method::POST, "/app/v3/api/assets/asset-001/relations"),
        (
            Method::DELETE,
            "/app/v3/api/assets/asset-001/relations/relation-001",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-001", "user-001", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-001", "user-001", "appbase"),
                    )
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
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

    for (method, uri) in [
        (Method::GET, "/app/v3/api/drive/storage_providers"),
        (Method::POST, "/app/v3/api/drive/storage_providers"),
        (
            Method::GET,
            "/app/v3/api/drive/storage_providers/provider-001",
        ),
        (
            Method::PATCH,
            "/app/v3/api/drive/storage_providers/provider-001",
        ),
        (
            Method::DELETE,
            "/app/v3/api/drive/storage_providers/provider-001",
        ),
        (
            Method::POST,
            "/app/v3/api/drive/storage_providers/provider-001/test",
        ),
        (
            Method::GET,
            "/app/v3/api/drive/storage_providers/provider-001/capabilities",
        ),
        (
            Method::POST,
            "/app/v3/api/drive/storage_providers/provider-001/activate",
        ),
        (
            Method::POST,
            "/app/v3/api/drive/storage_providers/provider-001/deactivate",
        ),
        (
            Method::POST,
            "/app/v3/api/drive/storage_providers/provider-001/credentials/rotate",
        ),
        (
            Method::GET,
            "/app/v3/api/drive/storage_providers/provider-001/bucket",
        ),
        (
            Method::PUT,
            "/app/v3/api/drive/storage_providers/provider-001/bucket",
        ),
        (
            Method::DELETE,
            "/app/v3/api/drive/storage_providers/provider-001/bucket",
        ),
        (
            Method::GET,
            "/app/v3/api/drive/storage_providers/provider-001/objects",
        ),
        (
            Method::GET,
            "/app/v3/api/drive/storage_providers/provider-001/objects/object-key",
        ),
        (
            Method::DELETE,
            "/app/v3/api/drive/storage_providers/provider-001/objects/object-key",
        ),
        (
            Method::POST,
            "/app/v3/api/drive/storage_providers/provider-001/objects/copy",
        ),
        (
            Method::GET,
            "/app/v3/api/drive/storage_provider_bindings/default",
        ),
        (
            Method::PUT,
            "/app/v3/api/drive/storage_provider_bindings/default",
        ),
        (Method::GET, "/app/v3/api/generations/assets"),
        (Method::POST, "/app/v3/api/generations/assets"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-001", "user-001", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-001", "user-001", "appbase"),
                    )
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .expect("request should be built"),
            )
            .await
            .unwrap_or_else(|error| panic!("{uri} should be handled: {error}"));
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "{uri} must not be exposed by the app api"
        );
    }
}

#[tokio::test]
async fn app_forbidden_asset_upload_routes_return_gone_problem_detail() {
    let app = build_router();

    for (method, uri) in [
        (Method::POST, "/app/v3/api/assets/upload"),
        (Method::POST, "/app/v3/api/assets/presign"),
        (Method::POST, "/app/v3/api/assets/upload_sessions"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-001", "user-001", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-001", "user-001", "appbase"),
                    )
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .expect("request should be built"),
            )
            .await
            .unwrap_or_else(|error| panic!("{uri} should be handled: {error}"));
        assert_eq!(
            response.status(),
            StatusCode::GONE,
            "{uri} must remain fail-closed and direct clients to Drive uploader APIs"
        );
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("assets response should be read"),
        )
        .expect("assets response should be valid json");
        assert_eq!(payload["code"].as_i64(), Some(41001));
        assert_eq!(payload["status"].as_i64(), Some(410));
        assert_ne!(payload["title"].as_str(), Some("not implemented"));
        assert_ne!(payload["detail"].as_str(), Some("not implemented"));
        assert!(payload["traceId"].as_str().is_some());
    }
}

#[tokio::test]
async fn app_asset_delete_resource_post_routes_return_method_not_allowed_problem_detail() {
    let app = build_router();

    for uri in [
        "/app/v3/api/assets/collections/collection-001/items/item-001",
        "/app/v3/api/assets/asset-001/relations/relation-001",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(
                        "authorization",
                        format!(
                            "Bearer {}",
                            common::auth_token("tenant-001", "user-001", "appbase")
                        ),
                    )
                    .header(
                        "access-token",
                        common::access_token("tenant-001", "user-001", "appbase"),
                    )
                    .method(Method::POST)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .expect("request should be built"),
            )
            .await
            .unwrap_or_else(|error| panic!("{uri} should be handled: {error}"));
        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "{uri} must reject POST with a standard method-not-allowed problem"
        );
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("assets response should be read"),
        )
        .expect("assets response should be valid json");
        assert_eq!(payload["code"].as_i64(), Some(40501));
        assert_eq!(payload["status"].as_i64(), Some(405));
        assert_eq!(payload["title"].as_str(), Some("method not allowed"));
        assert!(payload["traceId"].as_str().is_some());
    }
}
