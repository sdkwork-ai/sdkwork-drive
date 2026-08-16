use axum::body::{to_bytes, Body};
use http::{Method, Request, StatusCode};
use serde_json::Value;
use sqlx::PgPool;
use tower::util::ServiceExt;

mod common;

#[test]
fn sandbox_file_openapi_inlines_problem_json_for_every_error_response() {
    let document: Value = serde_json::from_str(include_str!(
        "../../../apis/app-api/drive/drive-app-api.openapi.json"
    ))
    .expect("drive app OpenAPI should be valid JSON");
    let operations = [
        ("/app/v3/api/drive/sandboxes/{sandboxId}/files", "post"),
        (
            "/app/v3/api/drive/sandboxes/{sandboxId}/files/{entryId}/content",
            "get",
        ),
        (
            "/app/v3/api/drive/sandboxes/{sandboxId}/files/{entryId}/content",
            "put",
        ),
        (
            "/app/v3/api/drive/sandboxes/{sandboxId}/entries/{entryId}",
            "patch",
        ),
        (
            "/app/v3/api/drive/sandboxes/{sandboxId}/entries/{entryId}/purge",
            "post",
        ),
    ];
    for (path, method) in operations {
        let responses = document["paths"][path][method]["responses"]
            .as_object()
            .unwrap_or_else(|| panic!("responses missing for {method} {path}"));
        for (status, response) in responses {
            if status.parse::<u16>().is_ok_and(|status| status >= 400) {
                assert!(
                    response.get("$ref").is_none(),
                    "sdkgen requires an inline error response for {method} {path} {status}"
                );
                assert_eq!(
                    response["content"]["application/problem+json"]["schema"]["$ref"],
                    "#/components/schemas/ProblemDetail",
                    "ProblemDetail media type missing for {method} {path} {status}"
                );
            }
        }
    }
}

#[tokio::test]
async fn sandbox_list_collapses_effective_grants_and_never_exposes_provider_details() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_sandbox(
        &pool,
        "sandbox-alpha",
        "Alpha",
        "active",
        "C:\\private\\alpha",
    )
    .await;
    seed_grant(
        &pool,
        "grant-alpha-user",
        "sandbox-alpha",
        "user",
        "user-001",
        "read_only",
    )
    .await;
    seed_grant(
        &pool,
        "grant-alpha-org",
        "sandbox-alpha",
        "organization",
        "organization-001",
        "full",
    )
    .await;

    seed_sandbox(
        &pool,
        "sandbox-beta",
        "Beta",
        "read_only",
        "/srv/private/beta",
    )
    .await;
    seed_grant(
        &pool,
        "grant-beta-org",
        "sandbox-beta",
        "organization",
        "organization-001",
        "full",
    )
    .await;

    seed_sandbox(
        &pool,
        "sandbox-disabled",
        "Disabled",
        "disabled",
        "/srv/disabled",
    )
    .await;
    seed_grant(
        &pool,
        "grant-disabled-user",
        "sandbox-disabled",
        "user",
        "user-001",
        "full",
    )
    .await;

    seed_sandbox(
        &pool,
        "sandbox-ungranted",
        "Ungranted",
        "active",
        "/srv/ungranted",
    )
    .await;

    let app = common::test_router_with_pool(pool);
    let response = app
        .oneshot(organization_request(
            "/app/v3/api/drive/sandboxes?page=1&page_size=200",
        ))
        .await
        .expect("sandbox list request should be handled");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("sandbox list response should be readable");
    let payload: Value = serde_json::from_slice(&body).expect("response should be json");
    assert_eq!(payload["code"], 0);
    assert!(payload["traceId"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert_eq!(payload["data"]["pageInfo"]["mode"], "offset");
    assert_eq!(payload["data"]["pageInfo"]["page"], 1);
    assert_eq!(payload["data"]["pageInfo"]["pageSize"], 200);
    assert_eq!(payload["data"]["pageInfo"]["totalItems"], "2");

    let items = payload["data"]["items"]
        .as_array()
        .expect("items should be an array");
    assert_eq!(items.len(), 2, "duplicate grants must collapse by sandbox");
    assert_eq!(items[0]["id"], "sandbox-alpha");
    assert_eq!(items[0]["effectiveAccess"], "full");
    assert_eq!(items[0]["capabilities"]["browse"], true);
    assert_eq!(items[0]["capabilities"]["createDirectory"], true);
    assert_eq!(items[0]["capabilities"]["selectDirectory"], true);
    assert_eq!(items[1]["id"], "sandbox-beta");
    assert_eq!(items[1]["effectiveAccess"], "full");
    assert_eq!(items[1]["lifecycleStatus"], "read_only");
    assert_eq!(items[1]["capabilities"]["createDirectory"], false);

    let serialized = String::from_utf8(body.to_vec()).expect("response should be utf-8");
    for forbidden in [
        "providerRootRef",
        "provider_root_ref",
        "providerKind",
        "tenantId",
        "organizationId",
        "C:\\private\\alpha",
        "/srv/private/beta",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "public response leaked private field or value: {forbidden}"
        );
    }
}

#[tokio::test]
async fn sandbox_list_enforces_auth_pagination_and_explicit_grants() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_sandbox(
        &pool,
        "sandbox-private",
        "Private",
        "active",
        "/srv/private",
    )
    .await;
    let app = common::test_router_with_pool(pool);

    let empty_response = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/sandboxes",
            "tenant-001",
            "user-without-grant",
            "appbase",
        ))
        .await
        .expect("empty sandbox list request should be handled");
    assert_eq!(empty_response.status(), StatusCode::OK);
    let empty_payload: Value = serde_json::from_slice(
        &to_bytes(empty_response.into_body(), usize::MAX)
            .await
            .expect("empty response should be readable"),
    )
    .expect("empty response should be json");
    assert_eq!(empty_payload["data"]["items"], serde_json::json!([]));
    assert_eq!(empty_payload["data"]["pageInfo"]["totalItems"], "0");

    for uri in [
        "/app/v3/api/drive/sandboxes?page=0",
        "/app/v3/api/drive/sandboxes?page_size=0",
        "/app/v3/api/drive/sandboxes?page_size=201",
        "/app/v3/api/drive/sandboxes?pageSize=20",
        "/app/v3/api/drive/sandboxes?limit=20",
    ] {
        let response = app
            .clone()
            .oneshot(common::authed_get(
                uri,
                "tenant-001",
                "user-without-grant",
                "appbase",
            ))
            .await
            .expect("invalid pagination request should be handled");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "uri: {uri}");
        assert_eq!(
            response.headers()["content-type"],
            "application/problem+json"
        );
    }

    let missing_tokens = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/drive/sandboxes")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("unauthenticated request should be handled");
    assert_eq!(missing_tokens.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn sandbox_directory_routes_list_create_paginate_and_audit_without_path_leakage() {
    let temp = tempfile::tempdir().expect("sandbox root should be created");
    std::fs::create_dir(temp.path().join("alpha")).expect("alpha should be created");
    std::fs::create_dir(temp.path().join("beta")).expect("beta should be created");
    std::fs::write(temp.path().join("README.md"), b"sandbox").expect("readme should be created");

    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_sandbox(
        &pool,
        "sandbox-directory",
        "Directory",
        "active",
        temp.path().to_string_lossy().as_ref(),
    )
    .await;
    seed_grant(
        &pool,
        "grant-directory-user",
        "sandbox-directory",
        "user",
        "user-001",
        "full",
    )
    .await;
    let app = common::test_router_with_pool(pool.clone());

    let first_response = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/sandboxes/sandbox-directory/entries?page_size=2",
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("first directory page should be handled");
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_body = to_bytes(first_response.into_body(), usize::MAX)
        .await
        .expect("first directory page should be readable");
    let first_payload: Value =
        serde_json::from_slice(&first_body).expect("first page should be json");
    assert_eq!(
        first_payload["data"]["items"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(first_payload["data"]["pageInfo"]["mode"], "cursor");
    assert_eq!(first_payload["data"]["pageInfo"]["pageSize"], 2);
    assert_eq!(first_payload["data"]["pageInfo"]["hasMore"], true);
    let cursor = first_payload["data"]["pageInfo"]["nextCursor"]
        .as_str()
        .expect("next cursor should be returned");
    assert!(!String::from_utf8_lossy(&first_body).contains(temp.path().to_string_lossy().as_ref()));

    let second_response = app
        .clone()
        .oneshot(common::authed_get(
            format!(
                "/app/v3/api/drive/sandboxes/sandbox-directory/entries?page_size=2&cursor={cursor}"
            ),
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("second directory page should be handled");
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_payload: Value = serde_json::from_slice(
        &to_bytes(second_response.into_body(), usize::MAX)
            .await
            .expect("second directory page should be readable"),
    )
    .expect("second page should be json");
    assert_eq!(
        second_payload["data"]["items"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(second_payload["data"]["pageInfo"]["hasMore"], false);

    let maximum_page_response = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/sandboxes/sandbox-directory/entries?page_size=1000",
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("maximum directory page should be handled");
    assert_eq!(maximum_page_response.status(), StatusCode::OK);

    let oversized_page_response = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/sandboxes/sandbox-directory/entries?page_size=1001",
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("oversized directory page should be handled");
    assert_eq!(oversized_page_response.status(), StatusCode::BAD_REQUEST);

    let create_response = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-directory/directories",
            "tenant-001",
            "user-001",
            "appbase",
            "create-components-001",
            Body::from(r#"{"parentPath":"alpha","name":"components"}"#),
        ))
        .await
        .expect("create directory request should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_payload: Value = serde_json::from_slice(
        &to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create response should be readable"),
    )
    .expect("create response should be json");
    assert_eq!(create_payload["code"], 0);
    assert_eq!(
        create_payload["data"]["item"]["logicalPath"],
        "alpha/components"
    );
    assert_eq!(create_payload["data"]["item"]["kind"], "directory");
    assert!(temp.path().join("alpha/components").is_dir());
    assert!(!create_payload
        .to_string()
        .contains(temp.path().to_string_lossy().as_ref()));

    let replay_response = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-directory/directories",
            "tenant-001",
            "user-001",
            "appbase",
            "create-components-001",
            Body::from(r#"{"parentPath":"alpha","name":"components"}"#),
        ))
        .await
        .expect("idempotent replay should be handled");
    assert_eq!(replay_response.status(), StatusCode::CREATED);
    let replay_payload: Value = serde_json::from_slice(
        &to_bytes(replay_response.into_body(), usize::MAX)
            .await
            .expect("replay response should be readable"),
    )
    .expect("replay response should be json");
    assert_eq!(
        replay_payload["data"]["item"]["id"],
        create_payload["data"]["item"]["id"]
    );

    let reused_key_response = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-directory/directories",
            "tenant-001",
            "user-001",
            "appbase",
            "create-components-001",
            Body::from(r#"{"parentPath":"alpha","name":"different"}"#),
        ))
        .await
        .expect("reused idempotency key should be handled");
    assert_eq!(reused_key_response.status(), StatusCode::CONFLICT);
    assert!(!temp.path().join("alpha/different").exists());

    let audit = sqlx::query(
        "SELECT tenant_id, action, resource_type, resource_id, operator_id, request_id, trace_id
         FROM dr_drive_audit_event
         WHERE action = 'drive.sandbox.directory.created'",
    )
    .fetch_one(&pool)
    .await
    .expect("directory creation audit should exist");
    use sqlx::Row;
    assert_eq!(audit.get::<String, _>("tenant_id"), "tenant-001");
    assert_eq!(audit.get::<String, _>("resource_type"), "sandbox");
    assert_eq!(audit.get::<String, _>("resource_id"), "sandbox-directory");
    assert_eq!(audit.get::<String, _>("operator_id"), "user-001");
    assert!(!audit.get::<String, _>("request_id").is_empty());
    assert_eq!(
        audit.get::<String, _>("trace_id"),
        create_payload["traceId"]
            .as_str()
            .expect("response trace id")
    );
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dr_drive_audit_event
         WHERE action = 'drive.sandbox.directory.created'
           AND resource_id = 'sandbox-directory'",
    )
    .fetch_one(&pool)
    .await
    .expect("directory audit count should be readable");
    assert_eq!(
        audit_count, 1,
        "idempotent replay must not duplicate audit events"
    );
}

#[tokio::test]
async fn sandbox_directory_routes_enforce_grants_read_only_and_logical_path_validation() {
    let temp = tempfile::tempdir().expect("sandbox root should be created");
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_sandbox(
        &pool,
        "sandbox-read-only",
        "Read only",
        "active",
        temp.path().to_string_lossy().as_ref(),
    )
    .await;
    seed_grant(
        &pool,
        "grant-read-only-user",
        "sandbox-read-only",
        "user",
        "user-001",
        "read_only",
    )
    .await;
    let app = common::test_router_with_pool(pool);

    let list_response = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/sandboxes/sandbox-read-only/entries",
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("read-only list should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);

    let create_response = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-read-only/directories",
            "tenant-001",
            "user-001",
            "appbase",
            "create-denied-001",
            Body::from(r#"{"parentPath":"","name":"denied"}"#),
        ))
        .await
        .expect("read-only create should be handled");
    assert_eq!(create_response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        create_response.headers()["content-type"],
        "application/problem+json"
    );
    assert!(!temp.path().join("denied").exists());

    let create_file_response = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-read-only/files",
            "tenant-001",
            "user-001",
            "appbase",
            "create-file-denied-001",
            Body::from(
                r#"{"parentPath":"","name":"denied.txt","content":"blocked","encoding":"utf8"}"#,
            ),
        ))
        .await
        .expect("read-only file create should be handled");
    assert_eq!(create_file_response.status(), StatusCode::FORBIDDEN);
    assert!(!temp.path().join("denied.txt").exists());

    let traversal_file_response = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-read-only/files",
            "tenant-001",
            "user-001",
            "appbase",
            "create-file-traversal-001",
            Body::from(
                r#"{"parentPath":"..","name":"escape.txt","content":"blocked","encoding":"utf8"}"#,
            ),
        ))
        .await
        .expect("file traversal should be handled");
    assert_eq!(traversal_file_response.status(), StatusCode::BAD_REQUEST);

    let unknown_field_response = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-read-only/files",
            "tenant-001",
            "user-001",
            "appbase",
            "create-file-unknown-field-001",
            Body::from(
                r#"{"parentPath":"","name":"unknown.txt","content":"blocked","encoding":"utf8","providerPath":"C:\\private"}"#,
            ),
        ))
        .await
        .expect("unknown field request should be handled");
    assert_eq!(unknown_field_response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        unknown_field_response.headers()["content-type"],
        "application/problem+json"
    );
    assert!(!temp.path().join("unknown.txt").exists());

    let traversal_response = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/sandboxes/sandbox-read-only/entries?parent_path=..",
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("traversal request should be handled");
    assert_eq!(traversal_response.status(), StatusCode::BAD_REQUEST);

    let forbidden_pagination_alias = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/sandboxes/sandbox-read-only/entries?limit=20",
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("forbidden pagination alias should be handled");
    assert_eq!(forbidden_pagination_alias.status(), StatusCode::BAD_REQUEST);
    let forbidden_payload: Value = serde_json::from_slice(
        &to_bytes(forbidden_pagination_alias.into_body(), usize::MAX)
            .await
            .expect("forbidden alias body"),
    )
    .expect("forbidden alias json");
    assert_eq!(forbidden_payload["code"], 40003);

    let ungranted_response = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/sandboxes/sandbox-read-only/entries",
            "tenant-001",
            "user-without-grant",
            "appbase",
        ))
        .await
        .expect("ungranted list should be handled");
    assert_eq!(ungranted_response.status(), StatusCode::FORBIDDEN);

    let missing_idempotency_key = app
        .oneshot(common::authed_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-read-only/directories",
            "tenant-001",
            "user-001",
            "appbase",
            Body::from(r#"{"parentPath":"","name":"missing-key"}"#),
        ))
        .await
        .expect("missing idempotency key should be handled");
    assert_eq!(missing_idempotency_key.status(), StatusCode::BAD_REQUEST);
    let missing_key_payload: Value = serde_json::from_slice(
        &to_bytes(missing_idempotency_key.into_body(), usize::MAX)
            .await
            .expect("missing idempotency response"),
    )
    .expect("missing idempotency json");
    assert_eq!(missing_key_payload["code"], 40004);
}

#[tokio::test]
async fn sandbox_file_routes_complete_binary_safe_optimistic_and_recursive_workflow() {
    let temp = tempfile::tempdir().expect("sandbox root should be created");
    std::fs::create_dir(temp.path().join("src")).expect("src should be created");
    std::fs::create_dir(temp.path().join("archive")).expect("archive should be created");
    std::fs::create_dir(temp.path().join("tree")).expect("tree should be created");
    std::fs::write(temp.path().join("tree/leaf.txt"), b"leaf").expect("leaf should be created");

    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_sandbox(
        &pool,
        "sandbox-files",
        "Files",
        "active",
        temp.path().to_string_lossy().as_ref(),
    )
    .await;
    seed_grant(
        &pool,
        "grant-files-user",
        "sandbox-files",
        "user",
        "user-001",
        "full",
    )
    .await;
    let app = common::test_router_with_pool(pool.clone());

    let create = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-files/files",
            "tenant-001",
            "user-001",
            "appbase",
            "create-main-file-001",
            Body::from(
                r#"{"parentPath":"src","name":"main.rs","content":"fn main() {}","encoding":"utf8"}"#,
            ),
        ))
        .await
        .expect("create file should be handled");
    assert_eq!(create.status(), StatusCode::CREATED);
    let create_body = to_bytes(create.into_body(), usize::MAX)
        .await
        .expect("create body");
    let create_payload: Value = serde_json::from_slice(&create_body).expect("create json");
    let main_entry_id = create_payload["data"]["item"]["id"]
        .as_str()
        .expect("entry id")
        .to_string();
    let main_revision = create_payload["data"]["item"]["revision"]
        .as_str()
        .expect("entry revision")
        .to_string();
    assert_eq!(create_payload["data"]["item"]["logicalPath"], "src/main.rs");
    assert_eq!(
        std::fs::read(temp.path().join("src/main.rs")).expect("created content"),
        b"fn main() {}"
    );
    assert!(!String::from_utf8_lossy(&create_body).contains(temp.path().to_string_lossy().as_ref()));

    let replay = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-files/files",
            "tenant-001",
            "user-001",
            "appbase",
            "create-main-file-001",
            Body::from(
                r#"{"parentPath":"src","name":"main.rs","content":"fn main() {}","encoding":"utf8"}"#,
            ),
        ))
        .await
        .expect("create replay should be handled");
    assert_eq!(replay.status(), StatusCode::CREATED);

    let create_binary = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            "/app/v3/api/drive/sandboxes/sandbox-files/files",
            "tenant-001",
            "user-001",
            "appbase",
            "create-binary-file-001",
            Body::from(
                r#"{"parentPath":"src","name":"data.bin","content":"AJ+Slv8=","encoding":"base64"}"#,
            ),
        ))
        .await
        .expect("create binary should be handled");
    assert_eq!(create_binary.status(), StatusCode::CREATED);
    let binary_payload: Value = serde_json::from_slice(
        &to_bytes(create_binary.into_body(), usize::MAX)
            .await
            .expect("binary create body"),
    )
    .expect("binary create json");
    let binary_entry_id = binary_payload["data"]["item"]["id"]
        .as_str()
        .expect("binary entry id");

    let read_text = app
        .clone()
        .oneshot(common::authed_get(
            format!(
                "/app/v3/api/drive/sandboxes/sandbox-files/files/{main_entry_id}/content?logical_path=src/main.rs&encoding=utf8"
            ),
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("read text should be handled");
    assert_eq!(read_text.status(), StatusCode::OK);
    let read_text_payload: Value = serde_json::from_slice(
        &to_bytes(read_text.into_body(), usize::MAX)
            .await
            .expect("text body"),
    )
    .expect("text json");
    assert_eq!(read_text_payload["data"]["item"]["content"], "fn main() {}");
    assert_eq!(read_text_payload["data"]["item"]["encoding"], "utf8");
    assert_eq!(read_text_payload["data"]["item"]["sizeBytes"], "12");

    let read_binary = app
        .clone()
        .oneshot(common::authed_get(
            format!(
                "/app/v3/api/drive/sandboxes/sandbox-files/files/{binary_entry_id}/content?logical_path=src/data.bin&encoding=base64"
            ),
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("read binary should be handled");
    assert_eq!(read_binary.status(), StatusCode::OK);
    let read_binary_payload: Value = serde_json::from_slice(
        &to_bytes(read_binary.into_body(), usize::MAX)
            .await
            .expect("binary body"),
    )
    .expect("binary json");
    assert_eq!(read_binary_payload["data"]["item"]["content"], "AJ+Slv8=");

    let missing_if_match = app
        .clone()
        .oneshot(authed_sandbox_mutation(
            Method::PUT,
            &format!("/app/v3/api/drive/sandboxes/sandbox-files/files/{main_entry_id}/content"),
            "update-main-missing-precondition-001",
            None,
            r#"{"logicalPath":"src/main.rs","content":"updated","encoding":"utf8"}"#,
        ))
        .await
        .expect("missing precondition should be handled");
    assert_eq!(missing_if_match.status(), StatusCode::PRECONDITION_REQUIRED);
    assert_eq!(
        missing_if_match.headers()["content-type"],
        "application/problem+json"
    );

    let stale_update = app
        .clone()
        .oneshot(authed_sandbox_mutation(
            Method::PUT,
            &format!("/app/v3/api/drive/sandboxes/sandbox-files/files/{main_entry_id}/content"),
            "update-main-stale-001",
            Some("stale-revision"),
            r#"{"logicalPath":"src/main.rs","content":"wrong","encoding":"utf8"}"#,
        ))
        .await
        .expect("stale update should be handled");
    assert_eq!(stale_update.status(), StatusCode::PRECONDITION_FAILED);
    assert_eq!(
        std::fs::read(temp.path().join("src/main.rs")).expect("unchanged content"),
        b"fn main() {}"
    );

    let update = app
        .clone()
        .oneshot(authed_sandbox_mutation(
            Method::PUT,
            &format!("/app/v3/api/drive/sandboxes/sandbox-files/files/{main_entry_id}/content"),
            "update-main-file-001",
            Some(&main_revision),
            r#"{"logicalPath":"src/main.rs","content":"updated","encoding":"utf8"}"#,
        ))
        .await
        .expect("update should be handled");
    assert_eq!(update.status(), StatusCode::OK);
    let update_payload: Value = serde_json::from_slice(
        &to_bytes(update.into_body(), usize::MAX)
            .await
            .expect("update body"),
    )
    .expect("update json");
    let updated_revision = update_payload["data"]["item"]["revision"]
        .as_str()
        .expect("updated revision")
        .to_string();
    assert_eq!(
        std::fs::read(temp.path().join("src/main.rs")).expect("updated content"),
        b"updated"
    );

    let update_replay = app
        .clone()
        .oneshot(authed_sandbox_mutation(
            Method::PUT,
            &format!("/app/v3/api/drive/sandboxes/sandbox-files/files/{main_entry_id}/content"),
            "update-main-file-001",
            Some(&main_revision),
            r#"{"logicalPath":"src/main.rs","content":"updated","encoding":"utf8"}"#,
        ))
        .await
        .expect("update replay should be handled");
    assert_eq!(update_replay.status(), StatusCode::OK);

    let moved = app
        .clone()
        .oneshot(authed_sandbox_mutation(
            Method::PATCH,
            &format!(
                "/app/v3/api/drive/sandboxes/sandbox-files/entries/{main_entry_id}"
            ),
            "move-main-file-001",
            Some(&updated_revision),
            r#"{"logicalPath":"src/main.rs","destinationParentPath":"archive","destinationName":"renamed.rs"}"#,
        ))
        .await
        .expect("move should be handled");
    assert_eq!(moved.status(), StatusCode::OK);
    let moved_payload: Value = serde_json::from_slice(
        &to_bytes(moved.into_body(), usize::MAX)
            .await
            .expect("move body"),
    )
    .expect("move json");
    assert_eq!(
        moved_payload["data"]["item"]["logicalPath"],
        "archive/renamed.rs"
    );
    assert_ne!(moved_payload["data"]["item"]["id"], main_entry_id);
    assert!(!temp.path().join("src/main.rs").exists());
    assert_eq!(
        std::fs::read(temp.path().join("archive/renamed.rs")).expect("moved content"),
        b"updated"
    );

    let move_replay = app
        .clone()
        .oneshot(authed_sandbox_mutation(
            Method::PATCH,
            &format!(
                "/app/v3/api/drive/sandboxes/sandbox-files/entries/{main_entry_id}"
            ),
            "move-main-file-001",
            Some(&updated_revision),
            r#"{"logicalPath":"src/main.rs","destinationParentPath":"archive","destinationName":"renamed.rs"}"#,
        ))
        .await
        .expect("move replay should be handled");
    assert_eq!(move_replay.status(), StatusCode::OK);
    let move_replay_payload: Value = serde_json::from_slice(
        &to_bytes(move_replay.into_body(), usize::MAX)
            .await
            .expect("move replay body"),
    )
    .expect("move replay json");
    assert_eq!(
        move_replay_payload["data"]["item"]["logicalPath"],
        "archive/renamed.rs"
    );

    let root_entries = app
        .clone()
        .oneshot(common::authed_get(
            "/app/v3/api/drive/sandboxes/sandbox-files/entries?page_size=20",
            "tenant-001",
            "user-001",
            "appbase",
        ))
        .await
        .expect("root entries should be handled");
    let root_payload: Value = serde_json::from_slice(
        &to_bytes(root_entries.into_body(), usize::MAX)
            .await
            .expect("root entries body"),
    )
    .expect("root entries json");
    let tree = root_payload["data"]["items"]
        .as_array()
        .expect("root items")
        .iter()
        .find(|entry| entry["logicalPath"] == "tree")
        .expect("tree entry");
    let tree_id = tree["id"].as_str().expect("tree id");
    let tree_revision = tree["revision"].as_str().expect("tree revision");

    let non_recursive = app
        .clone()
        .oneshot(authed_sandbox_mutation(
            Method::POST,
            &format!("/app/v3/api/drive/sandboxes/sandbox-files/entries/{tree_id}/purge"),
            "purge-tree-empty-only-001",
            Some(tree_revision),
            r#"{"logicalPath":"tree","recursive":false}"#,
        ))
        .await
        .expect("non-recursive purge should be handled");
    assert_eq!(non_recursive.status(), StatusCode::CONFLICT);
    assert!(temp.path().join("tree/leaf.txt").exists());

    let recursive = app
        .clone()
        .oneshot(authed_sandbox_mutation(
            Method::POST,
            &format!("/app/v3/api/drive/sandboxes/sandbox-files/entries/{tree_id}/purge"),
            "purge-tree-recursive-001",
            Some(tree_revision),
            r#"{"logicalPath":"tree","recursive":true}"#,
        ))
        .await
        .expect("recursive purge should be handled");
    assert_eq!(recursive.status(), StatusCode::OK);
    let recursive_payload: Value = serde_json::from_slice(
        &to_bytes(recursive.into_body(), usize::MAX)
            .await
            .expect("purge body"),
    )
    .expect("purge json");
    assert_eq!(recursive_payload["data"]["accepted"], true);
    assert_eq!(recursive_payload["data"]["status"], "deleted");
    assert!(!temp.path().join("tree").exists());

    let recursive_replay = app
        .clone()
        .oneshot(authed_sandbox_mutation(
            Method::POST,
            &format!("/app/v3/api/drive/sandboxes/sandbox-files/entries/{tree_id}/purge"),
            "purge-tree-recursive-001",
            Some(tree_revision),
            r#"{"logicalPath":"tree","recursive":true}"#,
        ))
        .await
        .expect("recursive purge replay should be handled");
    assert_eq!(recursive_replay.status(), StatusCode::OK);

    let create_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dr_drive_audit_event
         WHERE action = 'drive.sandbox.file.created' AND resource_id = ?",
    )
    .bind(&main_entry_id)
    .fetch_one(&pool)
    .await
    .expect("create audit count");
    assert_eq!(
        create_audit_count, 1,
        "create replay must not duplicate audit"
    );
    let mutation_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dr_drive_audit_event
         WHERE action IN (
             'drive.sandbox.file.updated',
             'drive.sandbox.entry.moved',
             'drive.sandbox.entry.deleted'
         )",
    )
    .fetch_one(&pool)
    .await
    .expect("mutation audit count");
    assert_eq!(mutation_audit_count, 3);
}

fn authed_sandbox_mutation(
    method: Method,
    uri: &str,
    idempotency_key: &str,
    revision: Option<&str>,
    body: &str,
) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("idempotency-key", idempotency_key)
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
        );
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    request
        .body(Body::from(body.to_string()))
        .expect("sandbox mutation request should be built")
}

fn organization_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header(
            "authorization",
            format!(
                "Bearer {}",
                common::auth_token_for_organization(
                    "tenant-001",
                    "user-001",
                    "organization-001",
                    "appbase",
                )
            ),
        )
        .header(
            "access-token",
            common::access_token_for_organization(
                "tenant-001",
                "user-001",
                "organization-001",
                "appbase",
            ),
        )
        .body(Body::empty())
        .expect("organization request should be built")
}

async fn seed_sandbox(
    pool: &PgPool,
    id: &str,
    display_name: &str,
    lifecycle_status: &str,
    provider_root_ref: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_volume (
            id, tenant_id, organization_id, display_name, root_entry_id,
            provider_kind, provider_root_ref, lifecycle_status, default_access,
            version, created_by, updated_by
         ) VALUES (?, 'tenant-001', 'organization-001', ?, ?,
                   'local_filesystem', ?, ?, 'full', 1, 'admin-001', 'admin-001')",
    )
    .bind(id)
    .bind(display_name)
    .bind(format!("root-{id}"))
    .bind(provider_root_ref)
    .bind(lifecycle_status)
    .execute(pool)
    .await
    .expect("sandbox should be inserted");
}

async fn seed_grant(
    pool: &PgPool,
    id: &str,
    sandbox_id: &str,
    subject_type: &str,
    subject_id: &str,
    access_level: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_grant (
            id, sandbox_id, subject_type, subject_id, access_level, granted_by
         ) VALUES (?, ?, ?, ?, ?, 'admin-001')",
    )
    .bind(id)
    .bind(sandbox_id)
    .bind(subject_type)
    .bind(subject_id)
    .bind(access_level)
    .execute(pool)
    .await
    .expect("sandbox grant should be inserted");
}
