use axum::body::{to_bytes, Body};
use http::{Method, Request, StatusCode};

use tower::util::ServiceExt;

mod common;

#[tokio::test]
async fn version_routes_prefer_logical_node_version_ids() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    seed_file_version(&pool).await;

    let app = common::test_router_with_pool(pool.clone());
    let list_response = app
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
                .uri("/app/v3/api/drive/nodes/node-001/versions")
                .body(Body::empty())
                .expect("version list request should be built"),
        )
        .await
        .expect("version list request should be handled");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("version list response should be read"),
    )
    .expect("version list response should be valid json");
    assert_eq!(
        common::envelope_items(&list_payload)[0]["id"].as_str(),
        Some("version-node-001-v2")
    );

    let detail_response = app
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
                .uri("/app/v3/api/drive/nodes/node-001/versions/version-node-001-v1")
                .body(Body::empty())
                .expect("version detail request should be built"),
        )
        .await
        .expect("version detail request should be handled");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(detail_response.into_body(), usize::MAX)
            .await
            .expect("version detail response should be read"),
    )
    .expect("version detail response should be valid json");
    let detail_item = common::envelope_item(&detail_payload);
    assert_eq!(detail_item["id"].as_str(), Some("version-node-001-v1"));
    assert_eq!(
        detail_item["storageObjectId"].as_str(),
        Some("object-node-001-v1")
    );

    let delete_response = app
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
                .method(Method::DELETE)
                .uri("/app/v3/api/drive/nodes/node-001/versions/version-node-001-v1")
                .body(Body::empty())
                .expect("version delete request should be built"),
        )
        .await
        .expect("version delete request should be handled");
    common::assert_no_content_response(delete_response).await;
    let deleted_statuses: (String, String) = sqlx::query_as(
        "SELECT v.lifecycle_status, o.lifecycle_status
         FROM dr_drive_node_version v
         INNER JOIN dr_drive_storage_object o ON o.id=v.storage_object_id
         WHERE v.tenant_id='tenant-001'
           AND v.node_id='node-001'
           AND v.id='version-node-001-v1'",
    )
    .fetch_one(&pool)
    .await
    .expect("deleted version statuses should be readable");
    assert_eq!(
        deleted_statuses,
        ("deleted".to_string(), "deleted".to_string())
    );

    let restore_response = app
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
                .uri("/app/v3/api/drive/nodes/node-001/versions/version-node-001-v1/restore")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("version restore request should be built"),
        )
        .await
        .expect("version restore request should be handled");
    assert_eq!(restore_response.status(), StatusCode::OK);
    let restored_statuses: (String, String) = sqlx::query_as(
        "SELECT v.lifecycle_status, o.lifecycle_status
         FROM dr_drive_node_version v
         INNER JOIN dr_drive_storage_object o ON o.id=v.storage_object_id
         WHERE v.tenant_id='tenant-001'
           AND v.node_id='node-001'
           AND v.id='version-node-001-v1'",
    )
    .fetch_one(&pool)
    .await
    .expect("restored version statuses should be readable");
    assert_eq!(
        restored_statuses,
        ("active".to_string(), "active".to_string())
    );
}

async fn seed_file_version(pool: &sqlx::PgPool) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-001', 's3_compatible', 'provider-001', 'https://s3.example.com', 'us-east-1',
            'bucket-001', 1, 1, 'plain:test-access:test-secret', NULL, NULL,
            'active', 1, 'user-001', 'user-001'
        )",
    )
    .execute(pool)
    .await
    .expect("storage provider should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
        ) VALUES ('space-001', 'tenant-001', 'user', 'user-001', 'personal', 'Main', 'active', 1, 'user-001', 'user-001')",
    )
    .execute(pool)
    .await
    .expect("space should be seeded");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ('node-001', 'tenant-001', 'space-001', NULL, 'file', 'page.md', 'ready', 'active', 1, 'user-001', 'user-001')",
    )
    .execute(pool)
    .await
    .expect("node should be seeded");

    for (version_no, object_id, version_id, checksum) in [
        (
            1_i64,
            "object-node-001-v1",
            "version-node-001-v1",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        (
            2_i64,
            "object-node-001-v2",
            "version-node-001-v2",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ),
    ] {
        sqlx::query(
            "INSERT INTO dr_drive_storage_object (
                id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
                content_type, content_length, checksum_sha256_hex, lifecycle_status,
                created_by, updated_by
            ) VALUES (
                $1, 'tenant-001', 'node-001', $2,
                'provider-001', 'bucket-001', $3,
                'text/markdown', 42, $4, 'active', 'user-001', 'user-001'
            )",
        )
        .bind(object_id)
        .bind(version_no)
        .bind(format!("objects/node-001/v{version_no}.md"))
        .bind(checksum)
        .execute(pool)
        .await
        .expect("storage object should be seeded");

        sqlx::query(
            "INSERT INTO dr_drive_node_version (
                id, tenant_id, space_id, node_id, version_no, storage_object_id,
                content_type, content_length, checksum_sha256_hex, version_kind,
                version_label, change_source, change_summary, restored_from_version_id,
                app_id, app_resource_type, app_resource_id, scene, source,
                lifecycle_status, created_by, updated_by
            ) VALUES (
                $1, 'tenant-001', 'space-001', 'node-001', $2, $3,
                'text/markdown', 42, $4, 'auto', 'Draft', 'uploader',
                'Created from upload', NULL, 'sdkwork-notes', 'page', 'page-001',
                'notes_page', 'editor', 'active', 'user-001', 'user-001'
            )",
        )
        .bind(version_id)
        .bind(version_no)
        .bind(object_id)
        .bind(checksum)
        .execute(pool)
        .await
        .expect("logical node version should be seeded");
    }
}
