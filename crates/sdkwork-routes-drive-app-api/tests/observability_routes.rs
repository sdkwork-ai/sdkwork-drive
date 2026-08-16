use std::sync::{Arc, Mutex};

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use axum::Router;
use http::{Method, Request, StatusCode};

use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;
use tracing::subscriber::set_default;
use tracing_subscriber::fmt::MakeWriter;

mod common;

struct CapturedWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for CapturedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut guard = self
            .buffer
            .lock()
            .expect("captured writer mutex should not be poisoned");
        guard.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
struct CapturedWriterFactory {
    buffer: Arc<Mutex<Vec<u8>>>,
}

type CapturedS3Requests = Arc<Mutex<Vec<String>>>;

impl<'a> MakeWriter<'a> for CapturedWriterFactory {
    type Writer = CapturedWriter;

    fn make_writer(&'a self) -> Self::Writer {
        CapturedWriter {
            buffer: Arc::clone(&self.buffer),
        }
    }
}

fn install_capture_subscriber() -> (tracing::subscriber::DefaultGuard, Arc<Mutex<Vec<u8>>>) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(CapturedWriterFactory {
            buffer: Arc::clone(&buffer),
        })
        .without_time()
        .finish();
    (set_default(subscriber), buffer)
}

fn buffer_to_string(buffer: &Arc<Mutex<Vec<u8>>>) -> String {
    String::from_utf8(
        buffer
            .lock()
            .expect("captured buffer mutex should not be poisoned")
            .clone(),
    )
    .expect("captured log buffer should be valid utf8")
}

async fn start_s3_mock_server() -> String {
    let requests: CapturedS3Requests = Arc::new(Mutex::new(Vec::new()));
    let router = Router::new()
        .fallback(mock_s3_endpoint)
        .with_state(requests);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("mock s3 listener should bind");
    let address = listener
        .local_addr()
        .expect("mock s3 listener address should be available");
    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("mock s3 server should run");
    });
    format!("http://{address}")
}

async fn mock_s3_endpoint(
    State(requests): State<CapturedS3Requests>,
    method: Method,
    uri: Uri,
    body: Body,
) -> Response {
    let query = uri.query().unwrap_or_default().to_string();
    let _ = to_bytes(body, usize::MAX)
        .await
        .expect("mock s3 request body should be readable");
    requests
        .lock()
        .expect("captured s3 requests mutex should not be poisoned")
        .push(format!("{} {}?{}", method.as_str(), uri.path(), query));

    if method == Method::POST && query.contains("uploads") {
        return (
            StatusCode::OK,
            [("content-type", "application/xml")],
            r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>bucket-001</Bucket>
  <Key>objects/node-obs-001.bin</Key>
  <UploadId>mock-s3-upload-id</UploadId>
</InitiateMultipartUploadResult>"#,
        )
            .into_response();
    }

    StatusCode::OK.into_response()
}

#[tokio::test]
async fn app_routes_emit_standardized_observability_events() {
    let (guard, log_buffer) = install_capture_subscriber();
    let s3_endpoint = start_s3_mock_server().await;

    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool.clone());
    let create_space_response = app
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
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"space-obs-001",
                        "ownerSubjectType":"user",
                        "ownerSubjectId":"user-001",
                        "displayName":"Obs Space",
                        "spaceType":"personal"
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create space request should be handled");
    assert_eq!(create_space_response.status(), StatusCode::CREATED);
    let _ = to_bytes(create_space_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, NULL, 'file', $4, 'ready', 'active', 1, $5, $6)",
    )
    .bind("node-obs-001")
    .bind("tenant-001")
    .bind("space-obs-001")
    .bind("node-obs-001.bin")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed node should succeed");
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            'provider-obs-001', 's3_compatible', 'Obs S3', $1, 'us-east-1',
            'bucket-001', 1, 0, 'plain:test-access-key:test-secret-key',
            'AES256', 'STANDARD', 'active', 1, 'admin-001', 'admin-001'
        )",
    )
    .bind(&s3_endpoint)
    .execute(&pool)
    .await
    .expect("seed storage provider should succeed");

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
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "sessionId":"upload-obs-001",
                        "spaceId":"space-obs-001",
                        "nodeId":"node-obs-001",
                        "bucket":"bucket-001",
                        "objectKey":"objects/node-obs-001.bin",
                        "idempotencyKey":"idem-obs-001",
                        "expiresAtEpochMs":1800000000000
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create upload session request should be handled");
    assert_eq!(upload_response.status(), StatusCode::CREATED);
    let _ = to_bytes(upload_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    sqlx::query(
        "INSERT INTO dr_drive_storage_object (
            id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
            content_type, content_length, checksum_sha256_hex, lifecycle_status,
            created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', $11, $12)",
    )
    .bind("obj-obs-001")
    .bind("tenant-001")
    .bind("node-obs-001")
    .bind(1_i64)
    .bind("provider-obs-001")
    .bind("bucket-001")
    .bind("objects/node-obs-001.bin")
    .bind("application/octet-stream")
    .bind(128_i64)
    .bind("sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd")
    .bind("user-001")
    .bind("user-001")
    .execute(&pool)
    .await
    .expect("seed storage object should succeed");

    let download_url_response = app
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
                .uri("/app/v3/api/drive/download_urls")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "nodeId":"node-obs-001",
                        "requestedTtlSeconds":120
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create download url request should be handled");
    assert_eq!(download_url_response.status(), StatusCode::CREATED);
    let payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(download_url_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("response body should be valid json");
    let token = common::envelope_data(&payload)["downloadUrl"]
        .as_str()
        .expect("download url should exist")
        .rsplit('/')
        .next()
        .expect("download token should exist")
        .to_string();

    let list_spaces_response = app
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
                .uri("/app/v3/api/drive/spaces?ownerSubjectType=user&ownerSubjectId=user-001")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list spaces request should be handled");
    assert_eq!(list_spaces_response.status(), StatusCode::OK);
    let _ = to_bytes(list_spaces_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    let get_space_response = app
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
                .uri("/app/v3/api/drive/spaces/space-obs-001")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("get space request should be handled");
    assert_eq!(get_space_response.status(), StatusCode::OK);
    let _ = to_bytes(get_space_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    let update_space_response = app
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
                .method(Method::PATCH)
                .uri("/app/v3/api/drive/spaces/space-obs-001")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "displayName":"Obs Space Updated"
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("update space request should be handled");
    assert_eq!(update_space_response.status(), StatusCode::OK);
    let _ = to_bytes(update_space_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    let resolve_response = app
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
                .uri(format!("/app/v3/api/drive/download_tokens/{token}"))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("resolve token request should be handled");
    assert_eq!(resolve_response.status(), StatusCode::OK);
    let resolve_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(resolve_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable"),
    )
    .expect("resolve download token response should be valid json");
    assert!(
        common::envelope_data(&resolve_payload)["signedSourceUrl"].is_string(),
        "resolve download token should return an SDKWork envelope with signedSourceUrl"
    );

    let delete_space_response = app
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
                .uri("/app/v3/api/drive/spaces/space-obs-001")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("delete space request should be handled");
    common::assert_no_content_response(delete_space_response).await;

    drop(guard);
    let logs = buffer_to_string(&log_buffer);
    for event_name in [
        "drive.app.spaces.create",
        "drive.app.upload_sessions.create",
        "drive.app.download_urls.create",
        "drive.app.spaces.list",
        "drive.app.spaces.get",
        "drive.app.spaces.update",
        "drive.app.spaces.delete",
        "drive.app.download_tokens.resolve",
    ] {
        assert!(
            logs.contains(&format!("event=\"{event_name}\"")),
            "expected observability event {event_name}, got:\n{logs}"
        );
    }
    for expected_field in [
        "sdkwork.drive:",
        "drive.http.request",
        "result=\"ok\"",
        "filter_has_owner_subject_type=true",
        "filter_has_owner_subject_id=true",
        "space_id=\"space-obs-001\"",
        "lifecycle_status=\"deleted\"",
        "method=\"GET\"",
    ] {
        assert!(
            logs.contains(expected_field),
            "expected field {expected_field} in logs, got:\n{logs}"
        );
    }
}

#[tokio::test]
async fn app_route_errors_emit_standardized_observability_events() {
    let (guard, log_buffer) = install_capture_subscriber();

    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = common::test_router_with_pool(pool);
    let create_space_error_response = app
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
                .uri("/app/v3/api/drive/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"space-obs-err-001",
                        "ownerSubjectType":"user",
                        "ownerSubjectId":"user-001",
                        "displayName":"Obs Space",
                        "spaceType":"invalid-space-type"
                    }"#,
                ))
                .expect("request should be built"),
        )
        .await
        .expect("create space error request should be handled");
    assert_eq!(
        create_space_error_response.status(),
        StatusCode::BAD_REQUEST
    );
    let _ = to_bytes(create_space_error_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    let resolve_error_response = app
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
                .uri(format!(
                    "/app/v3/api/drive/download_tokens/{}",
                    build_download_token(
                        "tenant-001",
                        "node-not-found-001",
                        now_epoch_ms() + 120_000
                    )
                ))
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("resolve token error request should be handled");
    assert_eq!(resolve_error_response.status(), StatusCode::NOT_FOUND);
    let _ = to_bytes(resolve_error_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    drop(guard);
    let logs = buffer_to_string(&log_buffer);
    for expected_field in [
        "event=\"drive.app.spaces.create\"",
        "result=\"err\"",
        "error_kind=\"validation\"",
        "input_space_type=\"invalid-space-type\"",
        "event=\"drive.app.download_tokens.resolve\"",
        "error_kind=\"not_found\"",
        "method=\"GET\"",
    ] {
        assert!(
            logs.contains(expected_field),
            "expected field {expected_field} in app error logs, got:\n{logs}"
        );
    }
}

fn build_download_token(tenant_id: &str, node_id: &str, expires_at_epoch_ms: i64) -> String {
    sdkwork_drive_workspace_service::application::download_service::build_download_token(
        tenant_id,
        node_id,
        expires_at_epoch_ms,
    )
    .expect("download token should be signed")
}

fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_millis() as i64
}
