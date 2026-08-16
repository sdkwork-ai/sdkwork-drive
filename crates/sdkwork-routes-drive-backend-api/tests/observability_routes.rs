use std::sync::{Arc, Mutex};

use axum::body::{to_bytes, Body};
use http::{Method, Request, StatusCode};
use sdkwork_routes_drive_backend_api::build_router_with_pool;
use tower::util::ServiceExt;
use tracing::subscriber::set_default;
use tracing_subscriber::fmt::MakeWriter;

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

#[tokio::test]
async fn list_audit_events_emits_structured_observability_log() {
    let (guard, log_buffer) = install_capture_subscriber();

    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO dr_drive_audit_event (
            id, tenant_id, action, resource_type, resource_id, operator_id, request_id, trace_id
        ) VALUES (97001, $1, $2, 'storage_provider', $3, $4, $5, $6)",
    )
    .bind("tenant-001")
    .bind("drive.storage_provider.created")
    .bind("provider-001")
    .bind("admin-001")
    .bind("request-001")
    .bind("trace-001")
    .execute(&pool)
    .await
    .expect("seed audit event should succeed");

    let app = build_router_with_pool(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/audit_events?action=drive.storage_provider.created&resourceType=storage_provider&resourceId=provider-001&correlationId=request-001&traceId=trace-001&page=1&page_size=10")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("list audit events request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let _ = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    drop(guard);
    let logs = buffer_to_string(&log_buffer);
    assert!(
        logs.contains("event=\"drive.audit_events.list\""),
        "expected audit list event log, got:\n{logs}"
    );
    for expected_field in [
        "filter_has_tenant_id=true",
        "filter_has_action=true",
        "filter_has_resource_type=true",
        "filter_has_resource_id=true",
        "filter_has_request_id=true",
        "filter_has_trace_id=true",
        "page=1",
        "page_size=10",
        "total=1",
        "returned_items=1",
    ] {
        assert!(
            logs.contains(expected_field),
            "expected field {expected_field} in logs, got:\n{logs}"
        );
    }
}

#[tokio::test]
async fn maintenance_sweeps_emit_structured_observability_logs() {
    let (guard, log_buffer) = install_capture_subscriber();

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
    let _ = to_bytes(object_sweep_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

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
    let _ = to_bytes(upload_sweep_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    for uri in [
        "/backend/v3/api/drive/maintenance/expired_upload_content_sweep",
        "/backend/v3/api/drive/maintenance/abandoned_upload_task_sweep",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(uri)
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
            .unwrap_or_else(|error| panic!("{uri} should be handled: {error}"));
        assert_eq!(response.status(), StatusCode::OK);
        let _ = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
    }

    let jobs_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/maintenance/jobs?jobType=object_sweep&status=completed&operatorId=user-001&page=1&page_size=10")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("maintenance jobs list request should be handled");
    assert_eq!(jobs_response.status(), StatusCode::OK);
    let _ = to_bytes(jobs_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    drop(guard);
    let logs = buffer_to_string(&log_buffer);

    assert!(
        logs.contains("event=\"drive.maintenance.object_sweep\""),
        "expected maintenance object sweep log, got:\n{logs}"
    );
    for expected_field in [
        "result=\"ok\"",
        "operator_id=\"user-001\"",
        "has_request_id=true",
        "has_trace_id=true",
        "scanned_count=1",
        "affected_count=1",
    ] {
        assert!(
            logs.contains(expected_field),
            "expected field {expected_field} in object sweep logs, got:\n{logs}"
        );
    }

    assert!(
        logs.contains("event=\"drive.maintenance.upload_session_sweep\""),
        "expected maintenance upload sweep log, got:\n{logs}"
    );

    assert!(
        logs.contains("event=\"drive.maintenance.expired_upload_content_sweep\""),
        "expected expired upload content sweep log, got:\n{logs}"
    );
    assert!(
        logs.contains("event=\"drive.maintenance.abandoned_upload_task_sweep\""),
        "expected abandoned upload task sweep log, got:\n{logs}"
    );
    for expected_field in [
        "result=\"ok\"",
        "now_epoch_ms=1800000000000",
        "operator_id=\"user-001\"",
        "has_request_id=true",
        "has_trace_id=true",
        "scanned_count=1",
        "affected_count=1",
    ] {
        assert!(
            logs.contains(expected_field),
            "expected field {expected_field} in upload sweep logs, got:\n{logs}"
        );
    }

    assert!(
        logs.contains("event=\"drive.maintenance.jobs.list\""),
        "expected maintenance jobs list log, got:\n{logs}"
    );
    for expected_field in [
        "filter_has_job_type=true",
        "filter_has_status=true",
        "filter_has_operator_id=true",
        "page=1",
        "page_size=10",
        "total=1",
        "returned_items=1",
    ] {
        assert!(
            logs.contains(expected_field),
            "expected field {expected_field} in maintenance jobs list logs, got:\n{logs}"
        );
    }
}

#[tokio::test]
async fn backend_route_errors_emit_structured_observability_logs() {
    let (guard, log_buffer) = install_capture_subscriber();

    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let app = build_router_with_pool(pool);
    let audit_error_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(
                    "/backend/v3/api/drive/audit_events?action=invalid%20action&page=1&page_size=10",
                )
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("audit error request should be handled");
    assert_eq!(audit_error_response.status(), StatusCode::BAD_REQUEST);
    let _ = to_bytes(audit_error_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    let maintenance_error_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/drive/maintenance/jobs?operatorId=invalid%20operator&page=1&page_size=10")
                .body(Body::empty())
                .expect("request should be built"),
        )
        .await
        .expect("maintenance jobs error request should be handled");
    assert_eq!(maintenance_error_response.status(), StatusCode::BAD_REQUEST);
    let _ = to_bytes(maintenance_error_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    drop(guard);
    let logs = buffer_to_string(&log_buffer);
    for expected_field in [
        "event=\"drive.audit_events.list\"",
        "result=\"err\"",
        "error_kind=\"validation\"",
        "filter_has_action=true",
        "event=\"drive.maintenance.jobs.list\"",
        "filter_has_operator_id=true",
    ] {
        assert!(
            logs.contains(expected_field),
            "expected field {expected_field} in backend error logs, got:\n{logs}"
        );
    }
}
