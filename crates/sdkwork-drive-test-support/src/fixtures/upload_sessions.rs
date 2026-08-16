use serde_json::json;

/// Create a test upload session fixture.
pub fn create_test_upload_session(
    session_id: &str,
    space_id: &str,
    node_id: &str,
) -> serde_json::Value {
    json!({
        "id": session_id,
        "space_id": space_id,
        "node_id": node_id,
        "idempotency_key": "test-idempotency-key",
        "state": "created",
        "expires_at_ms": 1700003600000i64,
        "created_at_ms": 1700000000000i64,
        "updated_at_ms": 1700000000000i64
    })
}

/// Create a completed upload session fixture.
pub fn create_completed_upload_session(
    session_id: &str,
    space_id: &str,
    node_id: &str,
) -> serde_json::Value {
    json!({
        "id": session_id,
        "space_id": space_id,
        "node_id": node_id,
        "idempotency_key": "test-idempotency-key",
        "state": "completed",
        "expires_at_ms": 1700003600000i64,
        "created_at_ms": 1700000000000i64,
        "updated_at_ms": 1700000010000i64
    })
}
