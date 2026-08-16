use serde_json::json;

/// Create a test S3 provider fixture.
pub fn create_test_s3_provider(provider_id: &str) -> serde_json::Value {
    json!({
        "id": provider_id,
        "provider_kind": "s3_compatible",
        "name": "Test S3 Provider",
        "endpoint_url": "https://s3.amazonaws.com",
        "region": "us-east-1",
        "bucket": "test-bucket",
        "path_style": false,
        "credential_ref": "env:AWS_ACCESS_KEY_ID",
        "status": "active",
        "version": 1,
        "created_at_ms": 1700000000000i64,
        "updated_at_ms": 1700000000000i64
    })
}

/// Create a test local filesystem provider fixture.
pub fn create_test_local_provider(provider_id: &str) -> serde_json::Value {
    json!({
        "id": provider_id,
        "provider_kind": "local_filesystem",
        "name": "Test Local Provider",
        "endpoint_url": "/tmp/test-storage",
        "region": null,
        "bucket": "local",
        "path_style": true,
        "credential_ref": null,
        "status": "active",
        "version": 1,
        "created_at_ms": 1700000000000i64,
        "updated_at_ms": 1700000000000i64
    })
}
