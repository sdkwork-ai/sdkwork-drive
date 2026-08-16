use axum::body::{to_bytes, Body};
use http::{Method, Request, StatusCode};
use sdkwork_routes_drive_app_api::build_router_with_pool_and_iam;
use sdkwork_web_core::{access_token_jwt, auth_token_jwt};
use serde_json::Value;
use tower::util::ServiceExt;

const DEFAULT_SESSION_ID: &str = "session-1";
const DEFAULT_APP_ID: &str = "appbase";

fn build_router() -> axum::Router {
    build_router_with_pool_and_iam(sdkwork_drive_test_support::lazy_postgres_test_pool())
}

fn auth_token(tenant: &str, user: &str) -> String {
    auth_token_jwt(tenant, user, DEFAULT_SESSION_ID, DEFAULT_APP_ID)
}

fn access_token(tenant: &str, user: &str) -> String {
    access_token_jwt(tenant, user, DEFAULT_SESSION_ID, DEFAULT_APP_ID)
}

#[tokio::test]
async fn app_production_routes_require_valid_dual_tokens() {
    let app = build_router();

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("health request should be handled");
    assert_eq!(health.status(), StatusCode::OK);

    let missing_credentials = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_problem(missing_credentials, StatusCode::UNAUTHORIZED, 40101).await;

    let missing_access = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "user-001")),
                )
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_problem(missing_access, StatusCode::UNAUTHORIZED, 40101).await;

    let invalid_credentials = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .header("authorization", "Bearer opaque-auth-token")
                .header("access-token", "opaque-access-token")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_problem(invalid_credentials, StatusCode::UNAUTHORIZED, 40103).await;
}

#[tokio::test]
async fn app_routes_validate_token_derived_app_context() {
    let app = app_router_allowing_unsigned_context();

    let tenant_conflict = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "user-001")),
                )
                .header("access-token", access_token("tenant-b", "user-001"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_problem(tenant_conflict, StatusCode::FORBIDDEN, 40301).await;

    let unknown_operator_id = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/drive/nodes/node-001/trash")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "user-001")),
                )
                .header("access-token", access_token("tenant-a", "user-001"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"operatorId":"user-002"}"#))
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_eq!(
        unknown_operator_id.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "operatorId is not a writable command field and must be rejected as unknown input"
    );

    let unknown_subject_selector = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/drive/shared_with_me?subjectType=user&subjectId=user-002")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "user-001")),
                )
                .header("access-token", access_token("tenant-a", "user-001"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_eq!(
        unknown_subject_selector.status(),
        StatusCode::BAD_REQUEST,
        "subject selectors are token-derived and must be rejected as unknown query input"
    );

    let prepare_without_app_id = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/drive/uploader/uploads")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "user-001")),
                )
                .header("access-token", access_token("tenant-a", "user-001"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"upload-item-token-app",
                        "taskId":"task-token-app",
                        "appResourceType":"desktop-file-browser",
                        "appResourceId":"root",
                        "fileFingerprint":"fp-token-app",
                        "originalFileName":"a.pdf",
                        "contentType":"application/pdf",
                        "contentLength":5,
                        "chunkSizeBytes":5242880
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("uploader prepare request should be handled");
    assert_ne!(
        prepare_without_app_id.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "authenticated uploader prepare should not fail JSON deserialization when appId is omitted"
    );

    let unknown_app_id = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/drive/uploader/uploads")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "user-001")),
                )
                .header("access-token", access_token("tenant-a", "user-001"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"upload-item-conflict",
                        "taskId":"task-conflict",
                        "appId":"other-app",
                        "appResourceType":"desktop-file-browser",
                        "appResourceId":"root",
                        "fileFingerprint":"fp-conflict",
                        "originalFileName":"a.pdf",
                        "contentType":"application/pdf",
                        "contentLength":5,
                        "chunkSizeBytes":5242880
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("uploader prepare request should be handled");
    assert_eq!(
        unknown_app_id.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "appId is not a writable prepare field and must be rejected as unknown input"
    );

    let allowed = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/drive/spaces")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "user-001")),
                )
                .header("access-token", access_token("tenant-a", "user-001"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_ne!(allowed.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(allowed.status(), StatusCode::FORBIDDEN);

    let token_scoped_shared = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/drive/shared_with_me")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "user-001")),
                )
                .header("access-token", access_token("tenant-a", "user-001"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("shared request should be handled");
    assert_ne!(token_scoped_shared.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(token_scoped_shared.status(), StatusCode::FORBIDDEN);
}

fn app_router_allowing_unsigned_context() -> axum::Router {
    let pool = sdkwork_drive_test_support::lazy_postgres_test_pool();
    build_router_with_pool_and_iam(pool)
}

async fn assert_problem(response: axum::response::Response, status: StatusCode, code: i64) {
    assert_eq!(response.status(), status);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("problem body should be readable");
    let problem: Value = serde_json::from_slice(&body).expect("problem body should be json");
    assert_eq!(problem["status"], status.as_u16());
    assert_eq!(problem["code"].as_i64(), Some(code));
    assert!(problem["traceId"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
}
