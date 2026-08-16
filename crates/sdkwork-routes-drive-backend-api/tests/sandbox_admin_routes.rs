use axum::body::{to_bytes, Body};
use axum::response::Response;
use http::{header, Method, Request, StatusCode};
use sdkwork_routes_drive_backend_api::build_router_with_pool;
use serde_json::{json, Value};
use tower::util::ServiceExt;

#[tokio::test]
async fn sandbox_admin_routes_complete_volume_and_grant_lifecycle_without_path_leakage() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let app = build_router_with_pool(pool.clone());
    let root = tempfile::tempdir().expect("sandbox root should be created");
    let canonical_root = std::fs::canonicalize(root.path())
        .expect("sandbox root should canonicalize")
        .to_string_lossy()
        .to_string();

    let create = send_json(
        &app,
        Method::POST,
        "/backend/v3/api/drive/sandbox_volumes",
        json!({
            "displayName": "Deployment workspace",
            "providerKind": "local_filesystem",
            "providerRootRef": root.path().to_string_lossy(),
            "defaultAccess": "full"
        }),
    )
    .await;
    assert_eq!(create.status(), StatusCode::CREATED);
    let create_body = response_json(create).await;
    assert_eq!(create_body["code"], 0);
    assert_eq!(create_body["traceId"], "trace-test");
    let created = &create_body["data"]["item"];
    assert_eq!(created["displayName"], "Deployment workspace");
    assert_eq!(created["providerKind"], "local_filesystem");
    assert_eq!(created["providerRootRef"], canonical_root);
    assert_eq!(created["defaultAccess"], "full");
    assert_eq!(created["lifecycleStatus"], "active");
    assert_eq!(created["version"], 1);
    let sandbox_id = created["id"]
        .as_str()
        .expect("sandbox id should be present")
        .to_string();

    sqlx::query(
        "INSERT INTO dr_drive_sandbox_volume (
            id, tenant_id, organization_id, display_name, root_entry_id,
            provider_kind, provider_root_ref, lifecycle_status, default_access,
            version, created_by, updated_by
         ) VALUES ('sandbox-other-organization', 'tenant-001', 'organization-002',
                   'Other organization', 'root-other-organization', 'local_filesystem',
                   'private-other-root', 'active', 'full', 1, 'user-002', 'user-002')",
    )
    .execute(&pool)
    .await
    .expect("cross-organization fixture should be inserted");

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/sandbox_volumes?page=1&page_size=1")
                .body(Body::empty())
                .expect("list request should build"),
        )
        .await
        .expect("list request should be handled");
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = response_json(list).await;
    assert_eq!(list_body["code"], 0);
    assert_eq!(list_body["data"]["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(list_body["data"]["pageInfo"]["mode"], "offset");
    assert_eq!(list_body["data"]["pageInfo"]["page"], 1);
    assert_eq!(list_body["data"]["pageInfo"]["pageSize"], 1);
    assert_eq!(list_body["data"]["pageInfo"]["totalItems"], "1");

    let cross_organization = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/sandbox_volumes/sandbox-other-organization")
                .body(Body::empty())
                .expect("cross-organization request should build"),
        )
        .await
        .expect("cross-organization request should be handled");
    assert_eq!(cross_organization.status(), StatusCode::NOT_FOUND);
    assert_eq!(response_json(cross_organization).await["code"], 40401);
    let cross_organization_delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/sandbox_volumes/sandbox-other-organization")
                .body(Body::empty())
                .expect("cross-organization delete should build"),
        )
        .await
        .expect("cross-organization delete should be handled");
    assert_eq!(cross_organization_delete.status(), StatusCode::NOT_FOUND);
    let cross_organization_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_sandbox_volume
         WHERE id='sandbox-other-organization'",
    )
    .fetch_one(&pool)
    .await
    .expect("cross-organization fixture count should be readable");
    assert_eq!(cross_organization_count, 1);

    let retrieve = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/backend/v3/api/drive/sandbox_volumes/{sandbox_id}"
                ))
                .body(Body::empty())
                .expect("retrieve request should build"),
        )
        .await
        .expect("retrieve request should be handled");
    assert_eq!(retrieve.status(), StatusCode::OK);
    assert_eq!(
        response_json(retrieve).await["data"]["item"]["providerRootRef"],
        canonical_root
    );

    let disabled = send_json(
        &app,
        Method::PATCH,
        &format!("/backend/v3/api/drive/sandbox_volumes/{sandbox_id}"),
        json!({"lifecycleStatus": "disabled", "expectedVersion": 1}),
    )
    .await;
    assert_eq!(disabled.status(), StatusCode::OK);
    let disabled_body = response_json(disabled).await;
    assert_eq!(disabled_body["data"]["item"]["lifecycleStatus"], "disabled");
    assert_eq!(disabled_body["data"]["item"]["version"], 2);

    let stale = send_json(
        &app,
        Method::PATCH,
        &format!("/backend/v3/api/drive/sandbox_volumes/{sandbox_id}"),
        json!({"displayName": "Stale update", "expectedVersion": 1}),
    )
    .await;
    assert_eq!(stale.status(), StatusCode::CONFLICT);
    assert_eq!(response_json(stale).await["code"], 40901);

    let active = send_json(
        &app,
        Method::PATCH,
        &format!("/backend/v3/api/drive/sandbox_volumes/{sandbox_id}"),
        json!({"lifecycleStatus": "active", "expectedVersion": 2}),
    )
    .await;
    assert_eq!(active.status(), StatusCode::OK);
    assert_eq!(
        response_json(active).await["data"]["item"]["lifecycleStatus"],
        "active"
    );

    let grants = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/backend/v3/api/drive/sandbox_volumes/{sandbox_id}/grants?page=1&page_size=20"
                ))
                .body(Body::empty())
                .expect("grant list request should build"),
        )
        .await
        .expect("grant list request should be handled");
    assert_eq!(grants.status(), StatusCode::OK);
    let grants_body = response_json(grants).await;
    assert_eq!(grants_body["data"]["items"][0]["subjectType"], "user");
    assert_eq!(grants_body["data"]["items"][0]["subjectId"], "user-001");
    assert_eq!(grants_body["data"]["items"][0]["accessLevel"], "full");

    let grant_create = send_json(
        &app,
        Method::POST,
        &format!("/backend/v3/api/drive/sandbox_volumes/{sandbox_id}/grants"),
        json!({
            "granteeType": "organization",
            "granteeId": "organization-002",
            "accessLevel": "full"
        }),
    )
    .await;
    assert_eq!(grant_create.status(), StatusCode::CREATED);
    let grant_create_body = response_json(grant_create).await;
    let grant_id = grant_create_body["data"]["item"]["id"]
        .as_str()
        .expect("grant id should be present")
        .to_string();
    assert_eq!(
        grant_create_body["data"]["item"]["subjectType"],
        "organization"
    );

    let duplicate_grant = send_json(
        &app,
        Method::POST,
        &format!("/backend/v3/api/drive/sandbox_volumes/{sandbox_id}/grants"),
        json!({
            "granteeType": "organization",
            "granteeId": "organization-002",
            "accessLevel": "full"
        }),
    )
    .await;
    assert_eq!(duplicate_grant.status(), StatusCode::CONFLICT);
    assert_eq!(response_json(duplicate_grant).await["code"], 40901);

    let grant_retrieve = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/backend/v3/api/drive/sandbox_volumes/{sandbox_id}/grants/{grant_id}"
                ))
                .body(Body::empty())
                .expect("grant retrieve request should build"),
        )
        .await
        .expect("grant retrieve request should be handled");
    assert_eq!(grant_retrieve.status(), StatusCode::OK);
    assert_eq!(
        response_json(grant_retrieve).await["data"]["item"]["id"],
        grant_id
    );

    let grant_update = send_json(
        &app,
        Method::PATCH,
        &format!("/backend/v3/api/drive/sandbox_volumes/{sandbox_id}/grants/{grant_id}"),
        json!({"accessLevel": "read_only"}),
    )
    .await;
    assert_eq!(grant_update.status(), StatusCode::OK);
    assert_eq!(
        response_json(grant_update).await["data"]["item"]["accessLevel"],
        "read_only"
    );

    let grant_delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!(
                    "/backend/v3/api/drive/sandbox_volumes/{sandbox_id}/grants/{grant_id}"
                ))
                .body(Body::empty())
                .expect("grant delete request should build"),
        )
        .await
        .expect("grant delete request should be handled");
    assert_eq!(grant_delete.status(), StatusCode::NO_CONTENT);
    assert!(
        to_bytes(grant_delete.into_body(), usize::MAX)
            .await
            .expect("delete body should be readable")
            .is_empty(),
        "204 must not contain a JSON body"
    );

    let unresolved = send_json(
        &app,
        Method::POST,
        &format!("/backend/v3/api/drive/sandbox_volumes/{sandbox_id}/grants"),
        json!({
            "granteeType": "workspace",
            "granteeId": "workspace-001",
            "accessLevel": "full"
        }),
    )
    .await;
    assert_eq!(unresolved.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        unresolved
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );
    let unresolved_body = response_json(unresolved).await;
    assert_eq!(unresolved_body["code"], 40001);
    assert!(unresolved_body["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("authoritative membership resolver")));

    let missing_root = root.path().join("secret-missing-root");
    let missing_root_text = missing_root.to_string_lossy().to_string();
    let invalid_root = send_json(
        &app,
        Method::POST,
        "/backend/v3/api/drive/sandbox_volumes",
        json!({
            "displayName": "Invalid root",
            "providerRootRef": missing_root_text
        }),
    )
    .await;
    assert_eq!(invalid_root.status(), StatusCode::BAD_REQUEST);
    let invalid_root_body = response_json(invalid_root).await;
    assert!(
        !invalid_root_body.to_string().contains(&missing_root_text),
        "validation problems must not echo physical paths"
    );

    for unavailable_provider in ["s3", "opendal"] {
        let invalid_provider = send_json(
            &app,
            Method::POST,
            "/backend/v3/api/drive/sandbox_volumes",
            json!({
                "displayName": format!("Unavailable {unavailable_provider}"),
                "providerKind": unavailable_provider,
                "providerRootRef": root.path().to_string_lossy()
            }),
        )
        .await;
        assert_eq!(invalid_provider.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            invalid_provider
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/problem+json")
        );
        let problem = response_json(invalid_provider).await;
        assert_eq!(problem["code"], 40003);
        assert!(problem["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("only local_filesystem")));
    }

    let invalid_query = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/sandbox_volumes?page=not-a-number")
                .body(Body::empty())
                .expect("invalid query request should build"),
        )
        .await
        .expect("invalid query request should be handled");
    assert_eq!(invalid_query.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        invalid_query
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );
    assert_eq!(response_json(invalid_query).await["code"], 40003);

    let oversized_page = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/sandbox_volumes?page=1&page_size=201")
                .body(Body::empty())
                .expect("oversized page request should build"),
        )
        .await
        .expect("oversized page request should be handled");
    assert_eq!(oversized_page.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response_json(oversized_page).await["code"], 40001);

    let overflowing_page = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/sandbox_volumes?page=9223372036854775807&page_size=200")
                .body(Body::empty())
                .expect("overflowing page request should build"),
        )
        .await
        .expect("overflowing page request should be handled");
    assert_eq!(overflowing_page.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response_json(overflowing_page).await["code"], 40001);

    for legacy_query in [
        "pageSize=20",
        "limit=20",
        "lifecycleStatus=active",
        "providerKind=local_filesystem",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!(
                        "/backend/v3/api/drive/sandbox_volumes?{legacy_query}"
                    ))
                    .body(Body::empty())
                    .expect("legacy query request should build"),
            )
            .await
            .expect("legacy query request should be handled");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response_json(response).await["code"], 40003);
    }

    let unknown_field = send_json(
        &app,
        Method::POST,
        "/backend/v3/api/drive/sandbox_volumes",
        json!({
            "displayName": "Unknown field",
            "providerRootRef": root.path().to_string_lossy(),
            "implicitTenantAccess": true
        }),
    )
    .await;
    assert_eq!(unknown_field.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response_json(unknown_field).await["code"], 40002);

    let volume_delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!(
                    "/backend/v3/api/drive/sandbox_volumes/{sandbox_id}"
                ))
                .body(Body::empty())
                .expect("volume delete request should build"),
        )
        .await
        .expect("volume delete request should be handled");
    assert_eq!(volume_delete.status(), StatusCode::NO_CONTENT);
    assert!(
        to_bytes(volume_delete.into_body(), usize::MAX)
            .await
            .expect("volume delete body should be readable")
            .is_empty(),
        "204 must not contain a JSON body"
    );
    assert!(
        root.path().exists(),
        "volume deletion must not delete provider data"
    );

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_audit_event
         WHERE tenant_id='tenant-001'
           AND action LIKE 'drive.sandbox_%'",
    )
    .fetch_one(&pool)
    .await
    .expect("sandbox audit count should be readable");
    assert_eq!(audit_count, 8);
    let wrong_tenant_count: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_audit_event WHERE tenant_id='0'")
            .fetch_one(&pool)
            .await
            .expect("legacy tenant audit count should be readable");
    assert_eq!(wrong_tenant_count, 0);
    let audit_values: Vec<(
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT tenant_id, action, resource_type, resource_id, request_id, trace_id
             FROM dr_drive_audit_event
             WHERE action LIKE 'drive.sandbox_%'",
    )
    .fetch_all(&pool)
    .await
    .expect("sandbox audit facts should be readable");
    assert!(
        !serde_json::to_string(&audit_values)
            .expect("audit values should serialize")
            .contains(&canonical_root),
        "physical roots must never enter audit facts"
    );
}

async fn send_json(app: &axum::Router, method: Method, uri: &str, payload: Value) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .expect("JSON request should build"),
        )
        .await
        .expect("JSON request should be handled")
}

async fn response_json(response: Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response body should be valid JSON")
}
