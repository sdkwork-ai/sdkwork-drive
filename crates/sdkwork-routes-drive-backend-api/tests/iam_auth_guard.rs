use axum::body::{to_bytes, Body};
use http::{Method, Request, StatusCode};
use sdkwork_routes_drive_backend_api::build_router_with_pool_and_iam;
use sdkwork_web_core::encode_unsigned_test_jwt;
use serde_json::{json, Value};
use tower::util::ServiceExt;

const TEST_SESSION: &str = "session-1";
const TEST_APP: &str = "appbase";

fn build_router() -> axum::Router {
    build_router_with_pool_and_iam(sdkwork_drive_test_support::lazy_postgres_test_pool())
}

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
        "permission_scope": "drive.storage.admin",
    }))
}

fn access_token(tenant: &str, user: &str) -> String {
    access_token_with_scope(tenant, user, "drive.storage.admin")
}

fn access_token_with_scope(tenant: &str, user: &str, permission_scope: &str) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "access",
        "tenant_id": tenant,
        "user_id": user,
        "session_id": TEST_SESSION,
        "app_id": TEST_APP,
        "organization_id": "100002",
        "environment": "prod",
        "deployment_mode": "saas",
        "login_scope": "ORGANIZATION",
        "permission_scope": permission_scope,
    }))
}

fn auth_token_with_scope(tenant: &str, user: &str, permission_scope: &str) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "auth",
        "tenant_id": tenant,
        "user_id": user,
        "session_id": TEST_SESSION,
        "app_id": TEST_APP,
        "organization_id": "100002",
        "auth_level": "password",
        "login_scope": "ORGANIZATION",
        "permission_scope": permission_scope,
    }))
}

fn personal_access_token(tenant: &str, user: &str) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "access",
        "tenant_id": tenant,
        "user_id": user,
        "session_id": TEST_SESSION,
        "app_id": TEST_APP,
        "environment": "prod",
        "deployment_mode": "saas",
        "login_scope": "TENANT",
        "permission_scope": "drive.storage.admin",
    }))
}

fn personal_auth_token(tenant: &str, user: &str) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "auth",
        "tenant_id": tenant,
        "user_id": user,
        "session_id": TEST_SESSION,
        "app_id": TEST_APP,
        "auth_level": "password",
        "login_scope": "TENANT",
        "permission_scope": "drive.storage.admin",
    }))
}

#[tokio::test]
async fn backend_production_routes_require_valid_dual_tokens() {
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
                .uri("/backend/v3/api/drive/quotas")
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
                .uri("/backend/v3/api/drive/quotas")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "admin-001")),
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
                .uri("/backend/v3/api/drive/quotas")
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
async fn backend_routes_validate_token_derived_app_context() {
    let Some((app, _database_guard)) = backend_router_allowing_unsigned_context().await else {
        return;
    };

    let tenant_conflict = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/quotas")
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

    let operator_conflict = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/maintenance/object_sweep")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "admin-001")),
                )
                .header("access-token", access_token("tenant-a", "admin-001"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"dryRun":true,"limit":1,"operatorId":"admin-002"}"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_problem(operator_conflict, StatusCode::FORBIDDEN, 40303).await;

    let allowed = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/quotas")
                .header(
                    "authorization",
                    format!("Bearer {}", auth_token("tenant-a", "admin-001")),
                )
                .header("access-token", access_token("tenant-a", "admin-001"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_ne!(allowed.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(allowed.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn backend_routes_reject_personal_login_scope_session() {
    let Some((app, _database_guard)) = backend_router_allowing_unsigned_context().await else {
        return;
    };

    let personal_session = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/quotas")
                .header(
                    "authorization",
                    format!("Bearer {}", personal_auth_token("tenant-a", "admin-001")),
                )
                .header(
                    "access-token",
                    personal_access_token("tenant-a", "admin-001"),
                )
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("protected request should be handled");
    assert_problem(personal_session, StatusCode::FORBIDDEN, 40301).await;
}

#[tokio::test]
async fn backend_routes_enforce_per_operation_admin_scopes() {
    let Some((app, _database_guard)) = backend_router_allowing_unsigned_context().await else {
        return;
    };
    let audit_scope = "drive.audit.admin";

    let audit_allowed = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/audit_events")
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        auth_token_with_scope("tenant-a", "admin-001", audit_scope)
                    ),
                )
                .header(
                    "access-token",
                    access_token_with_scope("tenant-a", "admin-001", audit_scope),
                )
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("audit request should be handled");
    assert_ne!(audit_allowed.status(), StatusCode::FORBIDDEN);

    let quota_denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/quotas")
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        auth_token_with_scope("tenant-a", "admin-001", audit_scope)
                    ),
                )
                .header(
                    "access-token",
                    access_token_with_scope("tenant-a", "admin-001", audit_scope),
                )
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("quota request should be handled");
    assert_problem(quota_denied, StatusCode::FORBIDDEN, 40301).await;

    let sandbox_denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/sandbox_volumes")
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        auth_token_with_scope("tenant-a", "admin-001", audit_scope)
                    ),
                )
                .header(
                    "access-token",
                    access_token_with_scope("tenant-a", "admin-001", audit_scope),
                )
                .body(Body::empty())
                .expect("sandbox request should be built"),
        )
        .await
        .expect("sandbox request should be handled");
    assert_problem(sandbox_denied, StatusCode::FORBIDDEN, 40301).await;

    let sandbox_scope = "drive.sandboxes.admin";
    let sandbox_allowed = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/sandbox_volumes")
                .header(
                    "authorization",
                    format!(
                        "Bearer {}",
                        auth_token_with_scope("tenant-a", "admin-001", sandbox_scope)
                    ),
                )
                .header(
                    "access-token",
                    access_token_with_scope("tenant-a", "admin-001", sandbox_scope),
                )
                .body(Body::empty())
                .expect("sandbox request should be built"),
        )
        .await
        .expect("sandbox request should be handled");
    assert_ne!(sandbox_allowed.status(), StatusCode::FORBIDDEN);
}

async fn backend_router_allowing_unsigned_context() -> Option<(
    axum::Router,
    sdkwork_drive_test_support::PostgresTestDatabaseGuard,
)> {
    let (pool, database_guard) = sdkwork_drive_test_support::postgres_test_database().await?;
    Some((build_router_with_pool_and_iam(pool), database_guard))
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
