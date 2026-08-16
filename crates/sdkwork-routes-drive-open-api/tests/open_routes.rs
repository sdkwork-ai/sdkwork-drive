use axum::body::{to_bytes, Body};
use http::{Method, Request, StatusCode};
use sdkwork_drive_workspace_service::drive_share_token_hash;
use sdkwork_routes_drive_open_api::{build_router_with_pool, open_route_manifest};
use sdkwork_web_core::RouteAuth;
use sqlx::PgPool;
use tower::util::ServiceExt;

fn envelope_item<'a>(payload: &'a serde_json::Value) -> &'a serde_json::Value {
    payload
        .get("data")
        .and_then(|data| data.get("item"))
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.item, got {payload}"))
}

#[test]
fn open_route_manifest_declares_public_share_link_operations() {
    let manifest = open_route_manifest();
    for (method, path, operation_id) in [
        (
            "GET",
            "/open/v3/api/drive/share_links/{token}",
            "openShareLinks.resolve",
        ),
        (
            "POST",
            "/open/v3/api/drive/share_links/{token}/download_url",
            "openShareLinks.downloadUrls.create",
        ),
    ] {
        let route = manifest
            .match_route(method, path)
            .unwrap_or_else(|| panic!("missing http route manifest for {method} {path}"));
        assert_eq!(route.auth, RouteAuth::Public);
        assert_eq!(route.operation_id, operation_id);
    }
}

#[tokio::test]
async fn open_share_link_resolves_and_creates_download_url_without_exposing_token_hash() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-open', 'tenant-open', 'user', 'user-open', 'personal', 'Open', 'active', 1, 'user-open', 'user-open')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-open', 'tenant-open', 'space-open', NULL, 'file', 'public.pdf', 'ready', 'active', 1, 'user-open', 'user-open')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_storage_provider_fixture(
        &pool,
        StorageProviderFixture {
            provider_id: "provider-open",
            provider_kind: "s3_compatible",
            provider_name: "Open MinIO",
            endpoint_url: "http://127.0.0.1:9000",
            region: "us-east-1",
            bucket: "bucket-open",
            path_style: true,
            status: "active",
            actor_id: "user-open",
        },
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "object-open-v1",
            tenant_id: "tenant-open",
            node_id: "node-open",
            version_no: 1,
            provider_id: "provider-open",
            bucket: "bucket-open",
            object_key: "objects/node-open/v1.pdf",
            content_type: "application/pdf",
            content_length: 2048,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-open",
        },
    )
    .await
    .expect("storage object should be seeded");

    let token = "share-token-open-001";
    let token_hash = drive_share_token_hash(token);
    assert_eq!(
        token_hash.len(),
        "sha256:".len() + 64,
        "share link token hash should be a SHA-256 hex digest"
    );
    assert!(token_hash.starts_with("sha256:"));
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-open', 'tenant-open', 'node-open', $1, 'reader', 4102444800000,
            2, 0, 'active', 1, 'user-open', 'user-open'
        )",
    )
    .bind(&token_hash)
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = build_router_with_pool(pool.clone());

    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/open/v3/api/drive/share_links/{token}"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("resolve request should be handled");
    assert_eq!(resolve_response.status(), StatusCode::OK);
    let resolve_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(resolve_response.into_body(), usize::MAX)
            .await
            .expect("resolve response body should be read"),
    )
    .expect("resolve response json should be valid");
    assert_eq!(resolve_payload["code"].as_i64(), Some(0));
    let resolve_item = envelope_item(&resolve_payload);
    assert_eq!(resolve_item["node"]["id"], "node-open");
    assert_eq!(resolve_item["node"]["nodeName"], "public.pdf");
    assert!(
        resolve_item.get("tokenHash").is_none(),
        "public response must not expose token hash"
    );

    let download_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/open/v3/api/drive/share_links/{token}/download_url"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"requestedTtlSeconds":120}"#))
                .expect("request should be built"),
        )
        .await
        .expect("download request should be handled");
    assert_eq!(download_response.status(), StatusCode::CREATED);
    let download_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(download_response.into_body(), usize::MAX)
            .await
            .expect("download response body should be read"),
    )
    .expect("download response json should be valid");
    let download_item = envelope_item(&download_payload);
    assert!(download_item["downloadUrl"]
        .as_str()
        .expect("downloadUrl should be a string")
        .contains("http://127.0.0.1:9000/bucket-open/objects/node-open/v1.pdf"));
    assert!(download_item["downloadUrl"]
        .as_str()
        .expect("downloadUrl should be a string")
        .contains("X-Amz-Signature="));
    assert_eq!(download_item["method"], "GET");

    let download_count: i64 = sqlx::query_scalar(
        "SELECT download_count FROM dr_drive_node_share_link WHERE id='share-open'",
    )
    .fetch_one(&pool)
    .await
    .expect("download count should be readable");
    assert_eq!(download_count, 1);
}

#[tokio::test]
async fn open_share_link_download_reads_object_from_its_bound_provider_when_bucket_is_shared() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-open-shared-provider', 'tenant-open-shared-provider',
            'user', 'user-open-shared-provider', 'personal', 'Open Shared Provider',
            'active', 1, 'user-open-shared-provider', 'user-open-shared-provider'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-open-shared-provider', 'tenant-open-shared-provider',
            'space-open-shared-provider', NULL, 'file', 'shared.pdf',
            'ready', 'active', 1, 'user-open-shared-provider',
            'user-open-shared-provider'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_storage_provider_fixture(
        &pool,
        StorageProviderFixture {
            provider_id: "provider-open-shared-wrong",
            provider_kind: "s3_compatible",
            provider_name: "Wrong Open Shared Provider",
            endpoint_url: "https://wrong-open-shared.example.test",
            region: "us-east-1",
            bucket: "bucket-open-shared",
            path_style: true,
            status: "active",
            actor_id: "user-open-shared-provider",
        },
    )
    .await;
    seed_storage_provider_fixture(
        &pool,
        StorageProviderFixture {
            provider_id: "provider-open-shared-bound",
            provider_kind: "s3_compatible",
            provider_name: "Bound Open Shared Provider",
            endpoint_url: "https://bound-open-shared.example.test",
            region: "us-east-1",
            bucket: "bucket-open-shared",
            path_style: true,
            status: "active",
            actor_id: "user-open-shared-provider",
        },
    )
    .await;
    sqlx::query(
        "UPDATE dr_drive_storage_provider
         SET updated_at='2999-01-01 00:00:00'
         WHERE id='provider-open-shared-wrong'",
    )
    .execute(&pool)
    .await
    .expect("wrong provider should be newest if looked up by bucket");
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "object-open-shared-provider-v1",
            tenant_id: "tenant-open-shared-provider",
            node_id: "node-open-shared-provider",
            version_no: 1,
            provider_id: "provider-open-shared-bound",
            bucket: "bucket-open-shared",
            object_key: "objects/node-open-shared-provider/v1.pdf",
            content_type: "application/pdf",
            content_length: 2048,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-open-shared-provider",
        },
    )
    .await
    .expect("storage object should be seeded");

    let token = "share-token-open-shared-provider";
    let token_hash = drive_share_token_hash(token);
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-open-shared-provider', 'tenant-open-shared-provider',
            'node-open-shared-provider', $1, 'reader', 4102444800000,
            2, 0, 'active', 1, 'user-open-shared-provider',
            'user-open-shared-provider'
        )",
    )
    .bind(&token_hash)
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/open/v3/api/drive/share_links/{token}/download_url"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"requestedTtlSeconds":120}"#))
                .expect("request should be built"),
        )
        .await
        .expect("download request should be handled");

    assert_eq!(response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("download response body should be read"),
    )
    .expect("download response json should be valid");
    let download_url = envelope_item(&payload)["downloadUrl"]
        .as_str()
        .expect("downloadUrl should be a string");
    assert!(
        download_url.starts_with("https://bound-open-shared.example.test/"),
        "download URL must be signed by the object's bound provider: {download_url}"
    );
    assert!(
        !download_url.starts_with("https://wrong-open-shared.example.test/"),
        "bucket lookup must not override the object's bound provider: {download_url}"
    );
}

#[tokio::test]
async fn open_share_link_download_uses_explicit_cloud_s3_provider_kinds_with_s3_signer() {
    for (provider_kind, endpoint_url, region, bucket, token, node_id) in [
        (
            "tencent_cos",
            "https://cos.ap-guangzhou.myqcloud.com",
            "ap-guangzhou",
            "bucket-open-cos",
            "share-token-open-cos",
            "node-open-cos",
        ),
        (
            "huawei_obs",
            "https://obs.cn-north-4.myhuaweicloud.com",
            "cn-north-4",
            "bucket-open-obs",
            "share-token-open-obs",
            "node-open-obs",
        ),
        (
            "volcengine_tos",
            "https://tos-cn-beijing.volces.com",
            "cn-beijing",
            "bucket-open-tos",
            "share-token-open-tos",
            "node-open-tos",
        ),
    ] {
        let Some((pool, _database_guard)) =
            sdkwork_drive_test_support::postgres_test_database().await
        else {
            return;
        };

        let tenant_id = format!("tenant-{provider_kind}");
        let space_id = format!("space-{provider_kind}");
        let object_id = format!("object-{provider_kind}");
        let provider_id = format!("provider-{provider_kind}");
        let share_id = format!("share-{provider_kind}");
        let object_key = format!("objects/{node_id}/v1.pdf");
        let token_hash = drive_share_token_hash(token);

        sqlx::query(
            "INSERT INTO dr_drive_space (
                id, tenant_id, owner_subject_type, owner_subject_id, space_type,
                display_name, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, $2, 'user', 'user-open-cloud', 'personal',
                'Open Cloud', 'active', 1, 'user-open-cloud', 'user-open-cloud')",
        )
        .bind(&space_id)
        .bind(&tenant_id)
        .execute(&pool)
        .await
        .expect("space should be seeded");
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, parent_node_id, node_type, node_name,
                content_state, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, $2, $3, NULL, 'file', 'public.pdf',
                'ready', 'active', 1, 'user-open-cloud', 'user-open-cloud')",
        )
        .bind(node_id)
        .bind(&tenant_id)
        .bind(&space_id)
        .execute(&pool)
        .await
        .expect("node should be seeded");
        seed_storage_provider_fixture(
            &pool,
            StorageProviderFixture {
                provider_id: &provider_id,
                provider_kind,
                provider_name: "Open Cloud S3",
                endpoint_url,
                region,
                bucket,
                path_style: false,
                status: "active",
                actor_id: "user-open-cloud",
            },
        )
        .await;
        seed_storage_object_fixture(
            &pool,
            StorageObjectFixture {
                object_id: &object_id,
                tenant_id: &tenant_id,
                node_id,
                version_no: 1,
                provider_id: &provider_id,
                bucket,
                object_key: &object_key,
                content_type: "application/pdf",
                content_length: 2048,
                checksum_sha256_hex:
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                lifecycle_status: "active",
                actor_id: "user-open-cloud",
            },
        )
        .await
        .expect("storage object should be seeded");
        sqlx::query(
            "INSERT INTO dr_drive_node_share_link (
                id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
                download_limit, download_count, lifecycle_status, version, created_by, updated_by
            ) VALUES ($1, $2, $3, $4, 'reader', 4102444800000,
                2, 0, 'active', 1, 'user-open-cloud', 'user-open-cloud')",
        )
        .bind(&share_id)
        .bind(&tenant_id)
        .bind(node_id)
        .bind(&token_hash)
        .execute(&pool)
        .await
        .expect("share link should be seeded");

        let app = build_router_with_pool(pool);
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/open/v3/api/drive/share_links/{token}/download_url"
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"requestedTtlSeconds":120}"#))
                    .expect("request should be built"),
            )
            .await
            .expect("download request should be handled");
        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "{provider_kind} should be signed through the S3 adapter"
        );
        let payload: serde_json::Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("response body should be read"),
        )
        .expect("response json should be valid");
        let download_url = envelope_item(&payload)["downloadUrl"]
            .as_str()
            .expect("downloadUrl should be present");
        assert!(
            download_url.contains("X-Amz-Signature="),
            "{provider_kind} should return an S3 presigned URL: {download_url}"
        );
        assert!(
            download_url.contains(&object_key),
            "{provider_kind} should preserve the Drive object key: {download_url}"
        );
    }
}

#[tokio::test]
async fn open_share_link_download_requires_active_object_store_provider() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-open-no-provider', 'tenant-open-no-provider', 'user', 'user-open-no-provider', 'personal', 'Open', 'active', 1, 'user-open-no-provider', 'user-open-no-provider')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-open-no-provider', 'tenant-open-no-provider', 'space-open-no-provider', NULL, 'file', 'public.pdf', 'ready', 'active', 1, 'user-open-no-provider', 'user-open-no-provider')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_storage_provider_fixture(
        &pool,
        StorageProviderFixture {
            provider_id: "provider-open-disabled",
            provider_kind: "s3_compatible",
            provider_name: "Disabled Open Provider",
            endpoint_url: "http://127.0.0.1:9000",
            region: "us-east-1",
            bucket: "bucket-open-no-provider",
            path_style: true,
            status: "disabled",
            actor_id: "user-open-no-provider",
        },
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "object-open-no-provider-v1",
            tenant_id: "tenant-open-no-provider",
            node_id: "node-open-no-provider",
            version_no: 1,
            provider_id: "provider-open-disabled",
            bucket: "bucket-open-no-provider",
            object_key: "objects/node-open-no-provider/v1.pdf",
            content_type: "application/pdf",
            content_length: 2048,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-open-no-provider",
        },
    )
    .await
    .expect("storage object should be seeded");

    let token = "share-token-open-no-provider";
    let token_hash = drive_share_token_hash(token);
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-open-no-provider', 'tenant-open-no-provider', 'node-open-no-provider',
            $1, 'reader', 4102444800000, 2, 0, 'active', 1, 'user-open-no-provider',
            'user-open-no-provider'
        )",
    )
    .bind(&token_hash)
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = build_router_with_pool(pool.clone());
    let download_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/open/v3/api/drive/share_links/{token}/download_url"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"requestedTtlSeconds":120}"#))
                .expect("request should be built"),
        )
        .await
        .expect("download request should be handled");

    assert_eq!(download_response.status(), StatusCode::CONFLICT);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(download_response.into_body(), usize::MAX)
            .await
            .expect("download response body should be read"),
    )
    .expect("download response json should be valid");
    assert_eq!(payload["code"], 40901);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be a string")
        .contains("active storage provider"));

    let download_count: i64 = sqlx::query_scalar(
        "SELECT download_count FROM dr_drive_node_share_link WHERE id='share-open-no-provider'",
    )
    .fetch_one(&pool)
    .await
    .expect("download count should be readable");
    assert_eq!(download_count, 0);
}

#[tokio::test]
async fn open_share_link_download_requires_active_storage_object() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-open-no-object', 'tenant-open-no-object', 'user', 'user-open-no-object', 'personal', 'Open', 'active', 1, 'user-open-no-object', 'user-open-no-object')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-open-no-object', 'tenant-open-no-object', 'space-open-no-object', NULL, 'file', 'public.pdf', 'uploading', 'active', 1, 'user-open-no-object', 'user-open-no-object')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let token = "share-token-open-no-object";
    let token_hash = drive_share_token_hash(token);
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-open-no-object', 'tenant-open-no-object', 'node-open-no-object',
            $1, 'reader', 4102444800000, 2, 0, 'active', 1, 'user-open-no-object',
            'user-open-no-object'
        )",
    )
    .bind(&token_hash)
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = build_router_with_pool(pool.clone());
    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/open/v3/api/drive/share_links/{token}"))
                .body(Body::empty())
                .expect("resolve request should be built"),
        )
        .await
        .expect("resolve request should be handled");
    assert_eq!(resolve_response.status(), StatusCode::OK);

    let download_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/open/v3/api/drive/share_links/{token}/download_url"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"requestedTtlSeconds":120}"#))
                .expect("download request should be built"),
        )
        .await
        .expect("download request should be handled");
    assert_eq!(download_response.status(), StatusCode::NOT_FOUND);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(download_response.into_body(), usize::MAX)
            .await
            .expect("download response body should be read"),
    )
    .expect("download response json should be valid");
    assert_eq!(payload["code"], 40401);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be a string")
        .contains("storage object"));

    let download_count: i64 = sqlx::query_scalar(
        "SELECT download_count FROM dr_drive_node_share_link WHERE id='share-open-no-object'",
    )
    .fetch_one(&pool)
    .await
    .expect("download count should be readable");
    assert_eq!(download_count, 0);
}

#[tokio::test]
async fn open_share_link_download_limit_is_consumed_atomically() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-open-limit', 'tenant-open-limit', 'user', 'user-open-limit',
            'personal', 'Open Limit', 'active', 1, 'user-open-limit', 'user-open-limit'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-open-limit', 'tenant-open-limit', 'space-open-limit', NULL,
            'file', 'limited.pdf', 'ready', 'active', 1, 'user-open-limit',
            'user-open-limit'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_storage_provider_fixture(
        &pool,
        StorageProviderFixture {
            provider_id: "provider-open-limit",
            provider_kind: "s3_compatible",
            provider_name: "Open Limit MinIO",
            endpoint_url: "http://127.0.0.1:9000",
            region: "us-east-1",
            bucket: "bucket-open-limit",
            path_style: true,
            status: "active",
            actor_id: "user-open-limit",
        },
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "object-open-limit-v1",
            tenant_id: "tenant-open-limit",
            node_id: "node-open-limit",
            version_no: 1,
            provider_id: "provider-open-limit",
            bucket: "bucket-open-limit",
            object_key: "objects/node-open-limit/v1.pdf",
            content_type: "application/pdf",
            content_length: 2048,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-open-limit",
        },
    )
    .await
    .expect("storage object should be seeded");

    let token = "share-token-open-limit";
    let token_hash = drive_share_token_hash(token);
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-open-limit', 'tenant-open-limit', 'node-open-limit',
            $1, 'reader', 4102444800000, 1, 0, 'active', 1,
            'user-open-limit', 'user-open-limit'
        )",
    )
    .bind(&token_hash)
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = build_router_with_pool(pool.clone());
    let first = app.clone().oneshot(
        Request::builder()
            .method(Method::POST)
            .uri(format!(
                "/open/v3/api/drive/share_links/{token}/download_url"
            ))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"requestedTtlSeconds":120}"#))
            .expect("request should be built"),
    );
    let second = app.oneshot(
        Request::builder()
            .method(Method::POST)
            .uri(format!(
                "/open/v3/api/drive/share_links/{token}/download_url"
            ))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"requestedTtlSeconds":120}"#))
            .expect("request should be built"),
    );

    let (first_response, second_response) = tokio::join!(first, second);
    let mut statuses = [
        first_response
            .expect("first request should be handled")
            .status(),
        second_response
            .expect("second request should be handled")
            .status(),
    ];
    statuses.sort();
    assert_eq!(
        statuses,
        [StatusCode::CREATED, StatusCode::TOO_MANY_REQUESTS]
    );

    let download_count: i64 = sqlx::query_scalar(
        "SELECT download_count FROM dr_drive_node_share_link WHERE id='share-open-limit'",
    )
    .fetch_one(&pool)
    .await
    .expect("download count should be readable");
    assert_eq!(download_count, 1);
}

#[tokio::test]
async fn open_share_link_download_rejects_ttl_outside_contract_before_consuming_limit() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-open-ttl', 'tenant-open-ttl', 'user', 'user-open-ttl',
            'personal', 'Open TTL', 'active', 1, 'user-open-ttl', 'user-open-ttl'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-open-ttl', 'tenant-open-ttl', 'space-open-ttl', NULL,
            'file', 'ttl.pdf', 'ready', 'active', 1, 'user-open-ttl',
            'user-open-ttl'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_storage_provider_fixture(
        &pool,
        StorageProviderFixture {
            provider_id: "provider-open-ttl",
            provider_kind: "s3_compatible",
            provider_name: "Open TTL MinIO",
            endpoint_url: "http://127.0.0.1:9000",
            region: "us-east-1",
            bucket: "bucket-open-ttl",
            path_style: true,
            status: "active",
            actor_id: "user-open-ttl",
        },
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "object-open-ttl-v1",
            tenant_id: "tenant-open-ttl",
            node_id: "node-open-ttl",
            version_no: 1,
            provider_id: "provider-open-ttl",
            bucket: "bucket-open-ttl",
            object_key: "objects/node-open-ttl/v1.pdf",
            content_type: "application/pdf",
            content_length: 2048,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-open-ttl",
        },
    )
    .await
    .expect("storage object should be seeded");

    let token = "share-token-open-ttl";
    let token_hash = drive_share_token_hash(token);
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-open-ttl', 'tenant-open-ttl', 'node-open-ttl',
            $1, 'reader', 4102444800000, 1, 0, 'active', 1,
            'user-open-ttl', 'user-open-ttl'
        )",
    )
    .bind(&token_hash)
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = build_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/open/v3/api/drive/share_links/{token}/download_url"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"requestedTtlSeconds":0}"#))
                .expect("request should be built"),
        )
        .await
        .expect("request should be handled");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    assert_eq!(payload["code"], 40001);
    assert!(payload["detail"]
        .as_str()
        .expect("detail should be a string")
        .contains("requestedTtlSeconds"));

    let download_count: i64 = sqlx::query_scalar(
        "SELECT download_count FROM dr_drive_node_share_link WHERE id='share-open-ttl'",
    )
    .fetch_one(&pool)
    .await
    .expect("download count should be readable");
    assert_eq!(download_count, 0);
}

#[tokio::test]
async fn open_share_link_download_treats_subsecond_remaining_share_ttl_as_expired() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'space-open-short-ttl', 'tenant-open-short-ttl', 'user', 'user-open-short-ttl',
            'personal', 'Open Short TTL', 'active', 1, 'user-open-short-ttl', 'user-open-short-ttl'
        )",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'node-open-short-ttl', 'tenant-open-short-ttl', 'space-open-short-ttl', NULL,
            'file', 'short-ttl.pdf', 'ready', 'active', 1, 'user-open-short-ttl',
            'user-open-short-ttl'
        )",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");
    seed_storage_provider_fixture(
        &pool,
        StorageProviderFixture {
            provider_id: "provider-open-short-ttl",
            provider_kind: "s3_compatible",
            provider_name: "Open Short TTL MinIO",
            endpoint_url: "http://127.0.0.1:9000",
            region: "us-east-1",
            bucket: "bucket-open-short-ttl",
            path_style: true,
            status: "active",
            actor_id: "user-open-short-ttl",
        },
    )
    .await;
    seed_storage_object_fixture(
        &pool,
        StorageObjectFixture {
            object_id: "object-open-short-ttl-v1",
            tenant_id: "tenant-open-short-ttl",
            node_id: "node-open-short-ttl",
            version_no: 1,
            provider_id: "provider-open-short-ttl",
            bucket: "bucket-open-short-ttl",
            object_key: "objects/node-open-short-ttl/v1.pdf",
            content_type: "application/pdf",
            content_length: 2048,
            checksum_sha256_hex:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            lifecycle_status: "active",
            actor_id: "user-open-short-ttl",
        },
    )
    .await
    .expect("storage object should be seeded");

    let token = "share-token-open-short-ttl";
    let token_hash = drive_share_token_hash(token);
    let expires_at_epoch_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis() as i64
        + 500;
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-open-short-ttl', 'tenant-open-short-ttl', 'node-open-short-ttl',
            $1, 'reader', $2, 1, 0, 'active', 1,
            'user-open-short-ttl', 'user-open-short-ttl'
        )",
    )
    .bind(&token_hash)
    .bind(expires_at_epoch_ms)
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = build_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/open/v3/api/drive/share_links/{token}/download_url"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"requestedTtlSeconds":120}"#))
                .expect("request should be built"),
        )
        .await
        .expect("request should be handled");

    assert_eq!(response.status(), StatusCode::GONE);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    assert_eq!(payload["code"], 41001);

    let download_count: i64 = sqlx::query_scalar(
        "SELECT download_count FROM dr_drive_node_share_link WHERE id='share-open-short-ttl'",
    )
    .fetch_one(&pool)
    .await
    .expect("download count should be readable");
    assert_eq!(download_count, 0);
}

#[tokio::test]
async fn open_share_link_requires_valid_access_code_when_configured() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-open-access-code', 'tenant-open-access-code', 'user', 'user-open-access-code', 'personal', 'Open', 'active', 1, 'user-open-access-code', 'user-open-access-code')",
    )
    .execute(&pool)
    .await
    .expect("space should be seeded");
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-open-access-code', 'tenant-open-access-code', 'space-open-access-code', NULL, 'file', 'protected.pdf', 'ready', 'active', 1, 'user-open-access-code', 'user-open-access-code')",
    )
    .execute(&pool)
    .await
    .expect("node should be seeded");

    let token = "share-token-open-access-code-123456789012345678901234567890";
    let token_hash = drive_share_token_hash(token);
    let access_code_hash = sdkwork_drive_workspace_service::drive_share_access_code_hash("9876");
    sqlx::query(
        "INSERT INTO dr_drive_node_share_link (
            id, tenant_id, node_id, token_hash, access_code_hash, role, expires_at_epoch_ms,
            download_limit, download_count, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'share-open-access-code', 'tenant-open-access-code', 'node-open-access-code',
            $1, $2, 'reader', 4102444800000, NULL, 0, 'active', 1, 'user-open-access-code',
            'user-open-access-code'
        )",
    )
    .bind(&token_hash)
    .bind(&access_code_hash)
    .execute(&pool)
    .await
    .expect("share link should be seeded");

    let app = build_router_with_pool(pool.clone());
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/open/v3/api/drive/share_links/{token}"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("resolve request should be handled");
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let allowed = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/open/v3/api/drive/share_links/{token}?accessCode=9876"
                ))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("resolve request should be handled");
    assert_eq!(allowed.status(), StatusCode::OK);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(allowed.into_body(), usize::MAX)
            .await
            .expect("response body should be read"),
    )
    .expect("response json should be valid");
    assert_eq!(envelope_item(&payload)["accessCodeRequired"], true);
}

#[tokio::test]
async fn open_metrics_endpoint_exposes_http_histogram_and_counters() {
    let app = build_router_with_pool(sdkwork_drive_test_support::lazy_postgres_test_pool());

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .expect("health request should be built"),
        )
        .await
        .expect("health request should be handled");
    assert_eq!(health.status(), StatusCode::OK);

    let metrics_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/metrics")
                .body(Body::empty())
                .expect("metrics request should be built"),
        )
        .await
        .expect("metrics request should be handled");
    assert_eq!(metrics_response.status(), StatusCode::OK);
    let body = to_bytes(metrics_response.into_body(), usize::MAX)
        .await
        .expect("metrics body should be read");
    let rendered = String::from_utf8(body.to_vec()).expect("metrics body should be utf8");
    assert!(rendered.contains("drive_http_request_duration_seconds_bucket"));
    assert!(rendered.contains("drive_http_requests_total"));
}

struct StorageProviderFixture<'a> {
    provider_id: &'a str,
    provider_kind: &'a str,
    provider_name: &'a str,
    endpoint_url: &'a str,
    region: &'a str,
    bucket: &'a str,
    path_style: bool,
    status: &'a str,
    actor_id: &'a str,
}

async fn seed_storage_provider_fixture(pool: &PgPool, fixture: StorageProviderFixture<'_>) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8,
            'plain:test-access:test-secret', NULL, NULL, $9, 1, $10, $10
        )",
    )
    .bind(fixture.provider_id)
    .bind(fixture.provider_kind)
    .bind(fixture.provider_name)
    .bind(fixture.endpoint_url)
    .bind(fixture.region)
    .bind(fixture.bucket)
    .bind(fixture.path_style)
    .bind(
        !fixture
            .endpoint_url
            .trim()
            .to_ascii_lowercase()
            .starts_with("http://"),
    )
    .bind(fixture.status)
    .bind(fixture.actor_id)
    .execute(pool)
    .await
    .expect("storage provider should be seeded");
}

struct StorageObjectFixture<'a> {
    object_id: &'a str,
    tenant_id: &'a str,
    node_id: &'a str,
    version_no: i64,
    provider_id: &'a str,
    bucket: &'a str,
    object_key: &'a str,
    content_type: &'a str,
    content_length: i64,
    checksum_sha256_hex: &'a str,
    lifecycle_status: &'a str,
    actor_id: &'a str,
}

async fn seed_storage_object_fixture(
    pool: &PgPool,
    fixture: StorageObjectFixture<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket,
            object_key, content_type, content_length, checksum_sha256_hex,
            lifecycle_status, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)",
    )
    .bind(fixture.object_id)
    .bind(fixture.tenant_id)
    .bind(fixture.node_id)
    .bind(fixture.version_no)
    .bind(fixture.provider_id)
    .bind(fixture.bucket)
    .bind(fixture.object_key)
    .bind(fixture.content_type)
    .bind(fixture.content_length)
    .bind(fixture.checksum_sha256_hex)
    .bind(fixture.lifecycle_status)
    .bind(fixture.actor_id)
    .execute(pool)
    .await?;
    Ok(())
}
