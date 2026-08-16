use axum::body::{to_bytes, Body};
use axum::response::Response;
use http::{Method, Request, StatusCode};
use sdkwork_routes_drive_backend_api::build_router_with_pool;
use sqlx::Row;
use tower::util::ServiceExt;

fn envelope_page_info(payload: &serde_json::Value) -> &serde_json::Value {
    payload
        .pointer("/data/pageInfo")
        .or_else(|| payload.get("pageInfo"))
        .expect("response should expose pageInfo in data.pageInfo or pageInfo")
}

fn envelope_items(payload: &serde_json::Value) -> &serde_json::Value {
    if let Some(items) = payload.pointer("/data/items") {
        return items;
    }
    payload
        .get("items")
        .expect("response should expose list items in data.items or items")
}

fn envelope_next_page_token(payload: &serde_json::Value) -> Option<String> {
    payload
        .pointer("/data/pageInfo/nextCursor")
        .or_else(|| {
            payload
                .get("pageInfo")
                .and_then(|info| info.get("nextCursor"))
        })
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .or_else(|| {
            payload
                .get("nextPageToken")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
}

async fn assert_no_content_response(response: Response) {
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("204 response body should be readable");
    assert!(
        body.is_empty(),
        "204 delete response must not include a JSON body"
    );
}

async fn fetch_backend_paged_items(
    app: axum::Router,
    uri: &str,
) -> (Vec<serde_json::Value>, Option<String>) {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("paged backend request should be built"),
        )
        .await
        .expect("paged backend request should be handled");
    assert_eq!(response.status(), StatusCode::OK, "{uri} should return OK");
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("paged backend response should be read"),
    )
    .expect("paged backend response should be valid json");
    let items = envelope_items(&payload)
        .as_array()
        .expect("items should be an array")
        .clone();
    let next_page_token = envelope_next_page_token(&payload);
    (items, next_page_token)
}

#[tokio::test]
async fn backend_dr_drive_labels_manage_definition_lifecycle_with_pagination_and_audit() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool(pool.clone());
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/labels")
                .header("content-type", "application/json")
                .body(Body::from(
                    r##"{
                        "id":"label-confidential",
                        "labelKey":"classification.confidential",
                        "displayName":"Confidential",
                        "color":"#D92D20",
                        "description":"Restricted business content"
                    }"##,
                ))
                .expect("create label request should be built"),
        )
        .await
        .expect("create label request should be handled");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create label response should be read"),
    )
    .expect("create label response should be valid json");
    assert_eq!(
        create_payload["labelKey"].as_str(),
        Some("classification.confidential")
    );
    assert_eq!(create_payload["displayName"].as_str(), Some("Confidential"));
    assert_eq!(create_payload["color"].as_str(), Some("#D92D20"));
    assert_eq!(create_payload["lifecycleStatus"].as_str(), Some("active"));
    assert_eq!(create_payload["version"].as_i64(), Some(1));

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/backend/v3/api/drive/labels/label-confidential")
                .header("content-type", "application/json")
                .body(Body::from(
                    r##"{
                        "displayName":"Highly Confidential",
                        "color":"#B42318"
                    }"##,
                ))
                .expect("update label request should be built"),
        )
        .await
        .expect("update label request should be handled");
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(update_response.into_body(), usize::MAX)
            .await
            .expect("update label response should be read"),
    )
    .expect("update label response should be valid json");
    assert_eq!(
        update_payload["displayName"].as_str(),
        Some("Highly Confidential")
    );
    assert_eq!(update_payload["version"].as_i64(), Some(2));

    let create_second = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/labels")
                .header("content-type", "application/json")
                .body(Body::from(
                    r##"{
                        "id":"label-public",
                        "labelKey":"classification.public",
                        "displayName":"Public",
                        "color":"#027A48"
                    }"##,
                ))
                .expect("create second label request should be built"),
        )
        .await
        .expect("create second label request should be handled");
    assert_eq!(create_second.status(), StatusCode::CREATED);

    let (first_items, next_token) =
        fetch_backend_paged_items(app.clone(), "/backend/v3/api/drive/labels?page_size=1").await;
    assert_eq!(first_items.len(), 1);
    assert_eq!(
        first_items[0]["labelKey"].as_str(),
        Some("classification.confidential")
    );
    let next_token = next_token.expect("label list should expose next page token");
    let (second_items, final_token) = fetch_backend_paged_items(
        app.clone(),
        &format!("/backend/v3/api/drive/labels?page_size=1&cursor={next_token}"),
    )
    .await;
    assert_eq!(second_items.len(), 1);
    assert_eq!(
        second_items[0]["labelKey"].as_str(),
        Some("classification.public")
    );
    assert!(final_token.is_none());

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/backend/v3/api/drive/labels/label-public")
                .body(Body::empty())
                .expect("delete label request should be built"),
        )
        .await
        .expect("delete label request should be handled");
    assert_no_content_response(delete_response).await;

    let remaining = fetch_backend_paged_items(app, "/backend/v3/api/drive/labels")
        .await
        .0;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0]["id"].as_str(), Some("label-confidential"));

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_audit_event
         WHERE resource_type='label'
           AND resource_id IN ('label-confidential', 'label-public')",
    )
    .fetch_one(&pool)
    .await
    .expect("label audit rows should be queryable");
    assert!(audit_count >= 4);
}

#[tokio::test]
async fn backend_label_list_rejects_page_size_outside_contract() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/labels?page_size=0")
                .body(Body::empty())
                .expect("label list request should be built"),
        )
        .await
        .expect("label list request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error body should be readable"),
    )
    .expect("error body should be valid json");
    assert_eq!(payload["code"], 40001);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be present")
        .contains("page_size"));
}

#[tokio::test]
async fn list_spaces_backend_route_returns_tenant_filtered_result() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("seed first space should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-002")
    .bind("tenant-002")
    .bind("user")
    .bind("user-002")
    .bind("team")
    .bind("Other")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("seed second space should succeed");

    let app = build_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/spaces")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list spaces request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(
        envelope_items(&payload)
            .as_array()
            .expect("items should be an array")
            .len(),
        1
    );
}

#[tokio::test]
async fn list_quotas_route_returns_usage_aggregated_from_storage_objects() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("seed space should succeed");

    for node_id in ["node-001", "node-002"] {
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, $2, $3, NULL, 'file', $4, 'ready', 'active', 1, $5, $6)",
        )
        .bind(node_id)
        .bind("tenant-001")
        .bind("space-001")
        .bind(format!("{node_id}.bin"))
        .bind("admin-001")
        .bind("admin-001")
        .execute(&pool)
        .await
        .expect("seed node should succeed");
    }

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-001', 's3_compatible', 'Quota S3', 'https://s3.example.com',
            'us-east-1', 'bucket-001', 1, 1, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-001', 'admin-001'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', $11, $12)",
    )
    .bind("obj-001")
    .bind("tenant-001")
    .bind("node-001")
    .bind(1_i64)
    .bind("provider-001")
    .bind("bucket-001")
    .bind("objects/node-001/a.bin")
    .bind("application/octet-stream")
    .bind(128_i64)
    .bind("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("seed first object should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', $11, $12)",
    )
    .bind("obj-002")
    .bind("tenant-001")
    .bind("node-002")
    .bind(1_i64)
    .bind("provider-001")
    .bind("bucket-001")
    .bind("objects/node-002/b.bin")
    .bind("application/octet-stream")
    .bind(256_i64)
    .bind("sha256:2222222222222222222222222222222222222222222222222222222222222222")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("seed second object should succeed");

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/quotas")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list quotas request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(payload["totalBytes"].as_i64(), Some(384));
    assert_eq!(payload["objectCount"].as_i64(), Some(2));
}

#[tokio::test]
async fn update_quota_policy_route_persists_tenant_cap_and_returns_quota_bytes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let pool_for_queries = pool.clone();
    let app = build_router_with_pool(pool);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/quotas")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "quotaBytes": 4096
                    })
                    .to_string(),
                ))
                .expect("request should be built"),
        )
        .await
        .expect("update quota request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(payload["quotaBytes"].as_i64(), Some(4096));

    let row = sqlx::query("SELECT max_bytes FROM dr_drive_tenant_quota WHERE tenant_id = $1")
        .bind("tenant-001")
        .fetch_one(&pool_for_queries)
        .await
        .expect("tenant quota row should exist after update");
    let max_bytes: i64 = row.get("max_bytes");
    assert_eq!(max_bytes, 4096);

    let clear_response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/backend/v3/api/drive/quotas")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "clearTenantPolicy": true
                    })
                    .to_string(),
                ))
                .expect("request should be built"),
        )
        .await
        .expect("clear quota request should be handled");
    assert_eq!(clear_response.status(), StatusCode::OK);

    let cleared = sqlx::query("SELECT max_bytes FROM dr_drive_tenant_quota WHERE tenant_id = $1")
        .bind("tenant-001")
        .fetch_optional(&pool_for_queries)
        .await
        .expect("tenant quota lookup should succeed");
    assert!(
        cleared.is_none(),
        "tenant quota row should be removed after clear"
    );
}

#[tokio::test]
async fn list_audit_events_route_supports_filters_and_pagination() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    for (index, (tenant_id, action, resource_id, operator_id, request_id, trace_id)) in [
        (
            "tenant-001",
            "drive.storage_provider.created",
            "provider-001",
            "admin-001",
            "request-001",
            "trace-001",
        ),
        (
            "tenant-001",
            "drive.storage_provider.tested",
            "provider-001",
            "admin-002",
            "request-002",
            "trace-002",
        ),
        (
            "tenant-002",
            "drive.storage_provider.created",
            "provider-002",
            "admin-003",
            "request-003",
            "trace-003",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        sqlx::query(
            "INSERT INTO dr_drive_audit_event (
                id, tenant_id, action, resource_type, resource_id,
                operator_id, request_id, trace_id
            ) VALUES ($1, $2, $3, 'storage_provider', $4, $5, $6, $7)",
        )
        .bind(1_470_000_i64 + index as i64)
        .bind(tenant_id)
        .bind(action)
        .bind(resource_id)
        .bind(operator_id)
        .bind(request_id)
        .bind(trace_id)
        .execute(&pool)
        .await
        .expect("seed audit event should succeed");
    }

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/audit_events?resourceType=storage_provider&page=1&page_size=1")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list audit events request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(envelope_page_info(&payload)["page"].as_i64(), Some(1));
    assert_eq!(envelope_page_info(&payload)["pageSize"].as_i64(), Some(1));
    assert_eq!(
        envelope_page_info(&payload)["totalItems"].as_str(),
        Some("2")
    );
    assert_eq!(
        envelope_items(&payload)
            .as_array()
            .expect("items should be an array")
            .len(),
        1
    );
}

#[tokio::test]
async fn list_audit_events_route_supports_request_and_trace_filters() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    for (index, (tenant_id, action, resource_id, operator_id, request_id, trace_id)) in [
        (
            "tenant-001",
            "drive.storage_provider.created",
            "provider-001",
            "admin-001",
            "request-001",
            "trace-001",
        ),
        (
            "tenant-001",
            "drive.storage_provider.tested",
            "provider-001",
            "admin-002",
            "request-002",
            "trace-002",
        ),
        (
            "tenant-001",
            "drive.storage_provider.created",
            "provider-003",
            "admin-003",
            "request-002",
            "trace-003",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        sqlx::query(
            "INSERT INTO dr_drive_audit_event (
                id, tenant_id, action, resource_type, resource_id,
                operator_id, request_id, trace_id
            ) VALUES ($1, $2, $3, 'storage_provider', $4, $5, $6, $7)",
        )
        .bind(1_555_000_i64 + index as i64)
        .bind(tenant_id)
        .bind(action)
        .bind(resource_id)
        .bind(operator_id)
        .bind(request_id)
        .bind(trace_id)
        .execute(&pool)
        .await
        .expect("seed audit event should succeed");
    }

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/audit_events?resourceType=storage_provider&correlationId=request-002&traceId=trace-002&page=1&page_size=10")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list audit events request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(
        envelope_page_info(&payload)["totalItems"].as_str(),
        Some("1")
    );
    let items = envelope_items(&payload)
        .as_array()
        .expect("items should be an array");
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0]["action"].as_str(),
        Some("drive.storage_provider.tested")
    );
    assert_eq!(items[0]["correlationId"].as_str(), Some("request-002"));
    assert_eq!(items[0]["traceId"].as_str(), Some("trace-002"));
}

#[tokio::test]
async fn list_audit_events_route_rejects_invalid_identifier_filters() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool(pool);
    for (query, expected_detail) in [
        (
            "/backend/v3/api/drive/audit_events?action=storage%20provider.created&page=1&page_size=10",
            "action contains invalid characters",
        ),
        (
            "/backend/v3/api/drive/audit_events?resourceId=provider%2F001&page=1&page_size=10",
            "resource_id contains invalid characters",
        ),
        (
            "/backend/v3/api/drive/audit_events?correlationId=request%20id&page=1&page_size=10",
            "request_id contains invalid characters",
        ),
        (
            "/backend/v3/api/drive/audit_events?traceId=trace%20id&page=1&page_size=10",
            "trace_id contains invalid characters",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(query)
                    .body(Body::empty())
                    .expect("request should be built"),
            )
            .await
            .expect("invalid audit events query request should be handled");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("response body should be readable"),
        )
        .expect("response body should be valid json");
        assert_eq!(payload["code"].as_i64(), Some(40001));
        assert!(
            payload["detail"]
                .as_str()
                .is_some_and(|detail| detail.contains(expected_detail)),
            "unexpected detail for query {}: {}",
            query,
            payload["detail"]
        );
    }
}

#[tokio::test]
async fn maintenance_routes_sweep_objects_and_upload_sessions_and_emit_audit_events() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', 1, $7, $8)",
    )
    .bind("space-001")
    .bind("tenant-001")
    .bind("user")
    .bind("user-001")
    .bind("personal")
    .bind("Main")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("space should be inserted");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, 'file', $4, 'ready', 'active', 1, $5, $6)",
    )
    .bind("node-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("node-001.bin")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("node should be inserted");

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-001', 's3_compatible', 'Maintenance S3',
            'https://s3.example.com', 'us-east-1', 'bucket-001', 1,
            'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
            'active', 1, 'admin-001', 'admin-001'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider should be inserted");
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'deleted', $11, $12)",
    )
    .bind("obj-001")
    .bind("tenant-001")
    .bind("node-001")
    .bind(1_i64)
    .bind("provider-001")
    .bind("bucket-001")
    .bind("objects/node-001.bin")
    .bind("application/octet-stream")
    .bind(128_i64)
    .bind("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("deleted storage object should be inserted");

    sqlx::query(
        "INSERT INTO dr_drive_upload_session (
            id, tenant_id, space_id, node_id, bucket, object_key,
            idempotency_key, storage_provider_id, storage_upload_id, state,
            expires_at_epoch_ms, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'provider-001', $1, 'created', $8, 1, $9, $10)",
    )
    .bind("session-001")
    .bind("tenant-001")
    .bind("space-001")
    .bind("node-001")
    .bind("bucket-001")
    .bind("objects/node-001.bin")
    .bind("idem-001")
    .bind(1_700_000_000_000_i64)
    .bind("admin-001")
    .bind("admin-001")
    .execute(&pool)
    .await
    .expect("upload session should be inserted");

    let app = build_router_with_pool(pool.clone());
    let object_sweep_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/maintenance/object_sweep")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "dryRun": false,
                        "limit": 100
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("object sweep request should be handled");
    assert_eq!(object_sweep_response.status(), StatusCode::OK);
    let object_sweep_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(object_sweep_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(object_sweep_payload["affectedCount"].as_i64(), Some(1));

    let upload_sweep_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/maintenance/upload_session_sweep")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nowEpochMs": 1800000000000,
                        "dryRun": false,
                        "limit": 100
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("upload session sweep request should be handled");
    assert_eq!(upload_sweep_response.status(), StatusCode::OK);
    let upload_sweep_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(upload_sweep_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(upload_sweep_payload["affectedCount"].as_i64(), Some(1));

    let expired_content_sweep_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/maintenance/expired_upload_content_sweep")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nowEpochMs": 1800000000000,
                        "dryRun": true,
                        "limit": 100
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("expired upload content sweep request should be handled");
    assert_eq!(expired_content_sweep_response.status(), StatusCode::OK);

    let abandoned_task_sweep_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/maintenance/abandoned_upload_task_sweep")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nowEpochMs": 1800000000000,
                        "dryRun": true,
                        "limit": 100
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("abandoned upload task sweep request should be handled");
    assert_eq!(abandoned_task_sweep_response.status(), StatusCode::OK);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_audit_event
         WHERE resource_type='maintenance'
           AND action IN (
             'drive.maintenance.object_sweep.executed',
             'drive.maintenance.upload_session_sweep.executed',
             'drive.maintenance.expired_upload_content_sweep.executed',
             'drive.maintenance.abandoned_upload_task_sweep.executed'
           )",
    )
    .fetch_one(&pool)
    .await
    .expect("audit rows should be queryable");
    assert_eq!(audit_count, 4);
}

#[tokio::test]
async fn list_maintenance_jobs_route_supports_filters_and_pagination() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    for (index, (job_type, status, operator_id, scanned_count, affected_count, created_at)) in [
        (
            "object_sweep",
            "completed",
            "admin-001",
            10_i64,
            5_i64,
            "2026-01-01T00:00:00Z",
        ),
        (
            "upload_session_sweep",
            "completed",
            "admin-001",
            8_i64,
            4_i64,
            "2026-01-01T00:00:01Z",
        ),
        (
            "object_sweep",
            "failed",
            "admin-002",
            3_i64,
            0_i64,
            "2026-01-01T00:00:02Z",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        sqlx::query(
            "INSERT INTO dr_drive_maintenance_job (
                id, job_type, status, dry_run, scanned_count, affected_count,
                operator_id, request_id, trace_id, error_message,
                started_at, finished_at, created_at
            ) VALUES ($1, $2, $3, 0, $4, $5, $6, 'request-001', 'trace-001', NULL, $7, $7, $7)",
        )
        .bind(1_877_000_i64 + index as i64)
        .bind(job_type)
        .bind(status)
        .bind(scanned_count)
        .bind(affected_count)
        .bind(operator_id)
        .bind(created_at)
        .execute(&pool)
        .await
        .expect("seed maintenance job should succeed");
    }

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/maintenance/jobs?jobType=object_sweep&status=completed&operatorId=admin-001&page=1&page_size=1")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list maintenance jobs request should be handled");
    assert_eq!(response.status(), StatusCode::OK);

    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(envelope_page_info(&payload)["page"].as_i64(), Some(1));
    assert_eq!(envelope_page_info(&payload)["pageSize"].as_i64(), Some(1));
    assert_eq!(
        envelope_page_info(&payload)["totalItems"].as_str(),
        Some("1")
    );
    let items = envelope_items(&payload)
        .as_array()
        .expect("items should be an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["jobType"].as_str(), Some("object_sweep"));
    assert_eq!(items[0]["status"].as_str(), Some("completed"));
    assert_eq!(items[0]["operatorId"].as_str(), Some("admin-001"));
    assert_eq!(items[0]["scannedCount"].as_i64(), Some(10));
    assert_eq!(items[0]["affectedCount"].as_i64(), Some(5));
}

#[tokio::test]
async fn list_download_packages_route_supports_filters_and_pagination() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-001', 's3_compatible', 'Download Package S3',
            'https://s3.example.com', 'us-east-1', 'bucket-001', 1,
            'plain:test-access-key:test-secret-key', 'AES256', 'STANDARD',
            'active', 1, 'admin-001', 'admin-001'
        )",
    )
    .execute(&pool)
    .await
    .expect("storage provider should be seeded");

    for (
        id,
        tenant_id,
        package_name,
        state,
        file_count,
        total_bytes,
        archive_size_bytes,
        created_at,
    ) in [
        (
            "pkg-001",
            "tenant-001",
            "January export",
            "ready",
            2_i64,
            128_i64,
            512_i64,
            "2026-01-01T00:00:00Z",
        ),
        (
            "pkg-002",
            "tenant-001",
            "February export",
            "expired",
            1_i64,
            64_i64,
            256_i64,
            "2026-01-01T00:00:01Z",
        ),
        (
            "pkg-003",
            "tenant-002",
            "Other tenant",
            "ready",
            1_i64,
            32_i64,
            128_i64,
            "2026-01-01T00:00:02Z",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_download_package (
                id, tenant_id, package_name, state, storage_provider_id, bucket,
                archive_object_key, content_type, file_count, total_bytes,
                archive_size_bytes, requested_node_ids_json, item_manifest_json,
                expires_at_epoch_ms, error_message, created_by, updated_by,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, 'provider-001', 'bucket-001',
                'sdkwork-drive/v1/t/aa/tenants/tenant-001/download-packages/pkg-001/archive.zip',
                'application/zip', $5, $6, $7, '[\"node-a\"]', '[]',
                1800000000000, NULL, 'admin-001', 'admin-001', $8, $8
            )",
        )
        .bind(id)
        .bind(tenant_id)
        .bind(package_name)
        .bind(state)
        .bind(file_count)
        .bind(total_bytes)
        .bind(archive_size_bytes)
        .bind(created_at)
        .execute(&pool)
        .await
        .expect("seed download package should succeed");
    }

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/download_packages?state=ready&page=1&page_size=1")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list download packages request should be handled");
    assert_eq!(response.status(), StatusCode::OK);

    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(envelope_page_info(&payload)["page"].as_i64(), Some(1));
    assert_eq!(envelope_page_info(&payload)["pageSize"].as_i64(), Some(1));
    assert_eq!(
        envelope_page_info(&payload)["totalItems"].as_str(),
        Some("1")
    );
    let items = envelope_items(&payload)
        .as_array()
        .expect("items should be an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"].as_str(), Some("pkg-001"));
    assert_eq!(items[0]["tenantId"].as_str(), Some("tenant-001"));
    assert_eq!(items[0]["packageName"].as_str(), Some("January export"));
    assert_eq!(items[0]["state"].as_str(), Some("ready"));
    assert_eq!(items[0]["contentType"].as_str(), Some("application/zip"));
    assert_eq!(items[0]["fileCount"].as_i64(), Some(2));
    assert_eq!(items[0]["totalBytes"].as_i64(), Some(128));
    assert_eq!(items[0]["archiveSizeBytes"].as_i64(), Some(512));
}

#[tokio::test]
async fn list_download_packages_rejects_page_and_page_size_outside_contract() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool(pool);
    for uri in [
        "/backend/v3/api/drive/download_packages?page=0",
        "/backend/v3/api/drive/download_packages?page_size=0",
        "/backend/v3/api/drive/download_packages?page=10001",
        "/backend/v3/api/drive/download_packages?page_size=101",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())
                    .expect("request should be built"),
            )
            .await
            .expect("list download packages request should be handled");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("error body should be readable"),
        )
        .expect("error body should be valid json");
        assert_eq!(payload["code"], 40001, "{uri}");
    }
}

#[tokio::test]
async fn maintenance_routes_record_failed_jobs_with_server_context() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query("DROP TABLE dr_drive_storage_object")
        .execute(&pool)
        .await
        .expect("drop storage object table should succeed");

    let app = build_router_with_pool(pool.clone());
    let failed_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/maintenance/object_sweep")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "dryRun": false,
                        "limit": 100
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("failed object sweep request should be handled");
    assert_eq!(failed_response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let failed_list_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/maintenance/jobs?status=failed&operatorId=user-001&page=1&page_size=10")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list failed maintenance jobs request should be handled");
    assert_eq!(failed_list_response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(failed_list_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(
        envelope_page_info(&payload)["totalItems"].as_str(),
        Some("1")
    );
    let items = envelope_items(&payload)
        .as_array()
        .expect("items should be an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["jobType"].as_str(), Some("object_sweep"));
    assert_eq!(items[0]["status"].as_str(), Some("failed"));
    assert_eq!(items[0]["correlationId"].as_str(), Some("request-test"));
    assert_eq!(items[0]["traceId"].as_str(), Some("trace-test"));
    assert!(
        items[0]["errorMessage"]
            .as_str()
            .is_some_and(|value| value.contains("count deleted dr_drive_storage_object failed")),
        "unexpected errorMessage: {}",
        items[0]["errorMessage"]
    );

    let failed_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_audit_event
         WHERE resource_type='maintenance'
           AND action='drive.maintenance.object_sweep.failed'
           AND operator_id='user-001'
           AND request_id='request-test'
           AND trace_id='trace-test'",
    )
    .fetch_one(&pool)
    .await
    .expect("failed maintenance audit rows should be queryable");
    assert_eq!(failed_audit_count, 1);
}

#[tokio::test]
async fn maintenance_upload_sweep_failure_records_failed_job_and_audit() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query("DROP TABLE dr_drive_upload_session")
        .execute(&pool)
        .await
        .expect("drop upload session table should succeed");

    let app = build_router_with_pool(pool.clone());
    let failed_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/maintenance/upload_session_sweep")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nowEpochMs": 1800000000000,
                        "dryRun": false,
                        "limit": 100
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("failed upload session sweep request should be handled");
    assert_eq!(failed_response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let failed_list_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/maintenance/jobs?jobType=upload_session_sweep&status=failed&operatorId=user-001&page=1&page_size=10")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list failed upload maintenance jobs request should be handled");
    assert_eq!(failed_list_response.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(failed_list_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(
        envelope_page_info(&payload)["totalItems"].as_str(),
        Some("1")
    );
    let items = envelope_items(&payload)
        .as_array()
        .expect("items should be an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["jobType"].as_str(), Some("upload_session_sweep"));
    assert_eq!(items[0]["status"].as_str(), Some("failed"));
    assert_eq!(items[0]["correlationId"].as_str(), Some("request-test"));
    assert_eq!(items[0]["traceId"].as_str(), Some("trace-test"));
    assert!(
        items[0]["errorMessage"]
            .as_str()
            .is_some_and(|value| value.contains("count expired dr_drive_upload_session failed")),
        "unexpected errorMessage: {}",
        items[0]["errorMessage"]
    );

    let failed_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM dr_drive_audit_event
         WHERE resource_type='maintenance'
           AND action='drive.maintenance.upload_session_sweep.failed'
           AND operator_id='user-001'
           AND request_id='request-test'
           AND trace_id='trace-test'",
    )
    .fetch_one(&pool)
    .await
    .expect("failed upload maintenance audit rows should be queryable");
    assert_eq!(failed_audit_count, 1);
}

#[tokio::test]
async fn maintenance_routes_reject_client_owned_audit_identity_fields() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/drive/maintenance/object_sweep")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "dryRun": true,
                        "limit": 1,
                        "correlationId": "request id with spaces"
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("client-owned correlationId sweep request should be handled");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40002));
    assert!(
        payload["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("matching the operation schema")),
        "unexpected detail: {}",
        payload["detail"]
    );
}

#[tokio::test]
async fn maintenance_jobs_route_rejects_invalid_operator_id_filter() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/maintenance/jobs?operatorId=admin%20ops&page=1&page_size=10")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("invalid operatorId filter request should be handled");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    assert_eq!(payload["code"].as_i64(), Some(40001));
    assert!(
        payload["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("operator_id contains invalid characters")),
        "unexpected detail: {}",
        payload["detail"]
    );
}
