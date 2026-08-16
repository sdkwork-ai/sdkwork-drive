use axum::body::{to_bytes, Body};
use axum::Router;
use chrono::{Duration, SecondsFormat, Utc};
use http::{Method, Response, StatusCode};
use sdkwork_drive_workspace_service::domain::website_sync::{
    validate_website_sync_tree, DriveWebsiteSyncTreeEntry,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::util::ServiceExt;

mod common;

#[tokio::test]
async fn website_sync_routes_create_retrieve_finalize_replay_and_enforce_tenant_scope() {
    let Some((pool, app, _database_guard)) = setup().await else {
        return;
    };
    create_space(&app).await;
    seed_storage_provider(&pool).await;
    let (root_uuid, root_node_id): (String, String) = sqlx::query_as(
        "SELECT uuid, active_node_id FROM dr_drive_website_root
         WHERE tenant_id='tenant-route-sync' AND space_id='space-route-sync'",
    )
    .fetch_one(&pool)
    .await
    .expect("default WebsiteRoot should exist");
    let entries = [file("index.html", 11, 'a'), file("app.js", 17, 'b')];
    let manifest = validate_website_sync_tree(&entries).expect("manifest should be valid");
    let expires_at = (Utc::now() + Duration::hours(1)).to_rfc3339_opts(SecondsFormat::Millis, true);

    let missing_key = app
        .clone()
        .oneshot(common::authed_post_json(
            format!("/app/v3/api/drive/website_roots/{root_uuid}/syncs"),
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            Body::from(create_body(&manifest, &expires_at).to_string()),
        ))
        .await
        .expect("missing-key request should be handled");
    assert_eq!(missing_key.status(), StatusCode::BAD_REQUEST);

    let created = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            format!("/app/v3/api/drive/website_roots/{root_uuid}/syncs"),
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            "route-sync-key-1",
            Body::from(create_body(&manifest, &expires_at).to_string()),
        ))
        .await
        .expect("create WebsiteSync request should be handled");
    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = response_json(created).await;
    let created_item = common::envelope_item(&created_json);
    assert_eq!(created_item["status"], "CREATED");
    assert_eq!(created_item["expectedGeneration"], "1");
    let sync_id = created_item["id"]
        .as_str()
        .expect("sync id should be a string")
        .to_string();
    let staging_node_id = created_item["stagingNodeId"]
        .as_str()
        .expect("staging node id should be a string")
        .to_string();

    let scratch_folder = app
        .clone()
        .oneshot(common::authed_post_json(
            "/app/v3/api/drive/nodes/folders",
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            Body::from(
                json!({
                    "spaceId": "space-route-sync",
                    "parentNodeId": staging_node_id,
                    "nodeName": "scratch"
                })
                .to_string(),
            ),
        ))
        .await
        .expect("writable staging folder create should be handled");
    assert_eq!(scratch_folder.status(), StatusCode::CREATED);
    let scratch_folder_json = response_json(scratch_folder).await;
    let scratch_folder_id = common::envelope_item(&scratch_folder_json)["id"]
        .as_str()
        .expect("scratch folder id should be returned")
        .to_string();
    let deleted_scratch = app
        .clone()
        .oneshot(common::authed_request(
            Method::DELETE,
            format!("/app/v3/api/drive/nodes/{scratch_folder_id}"),
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            Body::empty(),
        ))
        .await
        .expect("writable staging folder delete should be handled");
    assert_eq!(deleted_scratch.status(), StatusCode::NO_CONTENT);

    let replay = app
        .clone()
        .oneshot(common::authed_idempotent_post_json(
            format!("/app/v3/api/drive/website_roots/{root_uuid}/syncs"),
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            "route-sync-key-1",
            Body::from(create_body(&manifest, &expires_at).to_string()),
        ))
        .await
        .expect("WebsiteSync replay should be handled");
    assert_eq!(replay.status(), StatusCode::OK);

    let cross_tenant = app
        .clone()
        .oneshot(common::authed_get(
            format!("/app/v3/api/drive/website_roots/{root_uuid}/syncs/{sync_id}"),
            "tenant-other",
            "user-route-sync",
            "appbase",
        ))
        .await
        .expect("cross-tenant retrieve should be handled");
    assert_eq!(cross_tenant.status(), StatusCode::NOT_FOUND);

    for (index, entry) in entries.iter().enumerate() {
        let node_id = format!("route-sync-file-{index}");
        let content_type = if entry.relative_path.ends_with(".html") {
            "text/html"
        } else {
            "application/javascript"
        };
        sqlx::query(
            "INSERT INTO dr_drive_node (
                id, tenant_id, space_id, space_type, parent_node_id, node_type,
                node_name, content_state, head_content_type, head_content_type_group,
                head_content_length, head_version_no,
                head_checksum_sha256_hex, lifecycle_status, version,
                created_by, updated_by
             ) VALUES ($1, 'tenant-route-sync', 'space-route-sync', 'website', $2,
                       'file', $3, 'ready', $4, 'text', $5, 1, $6, 'active', 1,
                       'user-route-sync', 'user-route-sync')",
        )
        .bind(&node_id)
        .bind(&staging_node_id)
        .bind(&entry.relative_path)
        .bind(content_type)
        .bind(entry.content_length)
        .bind(entry.checksum_sha256_hex.as_deref())
        .execute(&pool)
        .await
        .expect("staging file should insert");
        sqlx::query(
            "INSERT INTO dr_drive_storage_object (
                id, tenant_id, node_id, version_no, storage_provider_id,
                bucket, object_key, content_type, content_length,
                checksum_sha256_hex, lifecycle_status, created_by, updated_by
             ) VALUES (
                $1, 'tenant-route-sync', $2, 1, 'provider-route-sync',
                'bucket-route-sync', $3, $4, $5, $6,
                'active', 'user-route-sync', 'user-route-sync'
             )",
        )
        .bind(format!("route-sync-object-{index}"))
        .bind(&node_id)
        .bind(format!("website-sync/{node_id}"))
        .bind(content_type)
        .bind(entry.content_length.expect("file length should exist"))
        .bind(
            entry
                .checksum_sha256_hex
                .as_deref()
                .expect("file checksum should exist"),
        )
        .execute(&pool)
        .await
        .expect("staging storage object should insert");
    }

    let finalized = app
        .clone()
        .oneshot(common::authed_post_json(
            format!("/app/v3/api/drive/website_roots/{root_uuid}/syncs/{sync_id}/finalize"),
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            Body::from(json!({ "expectedVersion": "1" }).to_string()),
        ))
        .await
        .expect("finalize WebsiteSync request should be handled");
    let finalized_status = finalized.status();
    let finalized_json = response_json(finalized).await;
    assert_eq!(
        finalized_status,
        StatusCode::OK,
        "finalize response: {finalized_json}"
    );
    let activation = common::envelope_item(&finalized_json);
    assert_eq!(activation["sync"]["status"], "COMPLETED");
    assert_eq!(activation["websiteRoot"]["activeGeneration"], "2");
    assert_eq!(
        activation["websiteRoot"]["contentMode"],
        "ATOMIC_GENERATION"
    );

    let immutable_create = app
        .clone()
        .oneshot(common::authed_post_json(
            "/app/v3/api/drive/nodes/folders",
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            Body::from(
                json!({
                    "spaceId": "space-route-sync",
                    "parentNodeId": staging_node_id,
                    "nodeName": "late-write"
                })
                .to_string(),
            ),
        ))
        .await
        .expect("immutable staging folder create should be handled");
    assert_eq!(immutable_create.status(), StatusCode::CONFLICT);

    let immutable_delete = app
        .clone()
        .oneshot(common::authed_request(
            Method::DELETE,
            "/app/v3/api/drive/nodes/route-sync-file-0",
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            Body::empty(),
        ))
        .await
        .expect("immutable generation node delete should be handled");
    assert_eq!(immutable_delete.status(), StatusCode::CONFLICT);

    let finalized_replay = app
        .clone()
        .oneshot(common::authed_post_json(
            format!("/app/v3/api/drive/website_roots/{root_uuid}/syncs/{sync_id}/finalize"),
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            Body::from(json!({ "expectedVersion": "1" }).to_string()),
        ))
        .await
        .expect("finalize replay should be handled");
    assert_eq!(finalized_replay.status(), StatusCode::OK);
    let generation_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM dr_drive_website_root_generation
         WHERE website_root_id=(SELECT id FROM dr_drive_website_root WHERE uuid=$1)",
    )
    .bind(&root_uuid)
    .fetch_one(&pool)
    .await
    .expect("generation count should be queryable");
    assert_eq!(generation_count, 2);
    assert_ne!(staging_node_id, root_node_id);
}

async fn setup() -> Option<(
    PgPool,
    Router,
    sdkwork_drive_test_support::test_database::PostgresTestDatabaseGuard,
)> {
    let (pool, database_guard) =
        sdkwork_drive_test_support::test_database::postgres_test_database().await?;
    let app = common::test_router_with_pool(pool.clone());
    Some((pool, app, database_guard))
}

async fn create_space(app: &Router) {
    let response = app
        .clone()
        .oneshot(common::authed_post_json(
            "/app/v3/api/drive/spaces",
            "tenant-route-sync",
            "user-route-sync",
            "appbase",
            Body::from(
                json!({
                    "id": "space-route-sync",
                    "ownerSubjectType": "user",
                    "ownerSubjectId": "user-route-sync",
                    "displayName": "Route Atomic Website",
                    "spaceType": "website"
                })
                .to_string(),
            ),
        ))
        .await
        .expect("create Space request should be handled");
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn seed_storage_provider(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode,
            default_storage_class, status, version, created_by, updated_by
         ) VALUES (
            'provider-route-sync', 's3_compatible', 'Route Sync Provider',
            'https://s3.example.com', 'us-east-1', 'bucket-route-sync', 1,
            1, 'plain:test-access-key:test-secret-key', 'AES256',
            'STANDARD', 'active', 1, 'test', 'test'
         )",
    )
    .execute(pool)
    .await
    .expect("storage provider should be seeded");
}

async fn response_json(response: Response<Body>) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response body should be JSON")
}

fn create_body(
    manifest: &sdkwork_drive_workspace_service::domain::website_sync::DriveWebsiteManifestSummary,
    expires_at: &str,
) -> Value {
    json!({
        "expectedRootVersion": "1",
        "expectedGeneration": "1",
        "manifestSha256": manifest.sha256,
        "manifestFileCount": manifest.file_count.to_string(),
        "manifestTotalBytes": manifest.total_bytes.to_string(),
        "expiresAt": expires_at
    })
}

fn file(path: &str, length: i64, digest_character: char) -> DriveWebsiteSyncTreeEntry {
    DriveWebsiteSyncTreeEntry {
        relative_path: path.to_string(),
        depth: 1,
        node_type: "file".to_string(),
        content_state: "ready".to_string(),
        content_length: Some(length),
        checksum_sha256_hex: Some(format!(
            "sha256:{}",
            digest_character.to_string().repeat(64)
        )),
        shortcut_target_node_id: None,
    }
}
