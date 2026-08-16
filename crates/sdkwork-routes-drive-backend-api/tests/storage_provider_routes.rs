use axum::body::Body;
use http::{Method, Request, StatusCode};
use sdkwork_routes_drive_backend_api::build_router_with_pool_and_iam;
use tower::util::ServiceExt;

fn build_router() -> axum::Router {
    build_router_with_pool_and_iam(sdkwork_drive_test_support::lazy_postgres_test_pool())
}

#[tokio::test]
async fn backend_router_exposes_health_route() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("health request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
}
