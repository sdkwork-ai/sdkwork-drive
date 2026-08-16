use serde_json::json;

/// Create a test space fixture.
pub fn create_test_space(space_id: &str, tenant_id: &str) -> serde_json::Value {
    json!({
        "id": space_id,
        "tenant_id": tenant_id,
        "owner_type": "user",
        "owner_id": tenant_id,
        "space_type": "personal",
        "name": "Test Space",
        "version": 1,
        "created_at_ms": 1700000000000i64,
        "updated_at_ms": 1700000000000i64
    })
}

/// Create a test team space fixture.
pub fn create_test_team_space(space_id: &str, tenant_id: &str) -> serde_json::Value {
    json!({
        "id": space_id,
        "tenant_id": tenant_id,
        "owner_type": "organization",
        "owner_id": "org-123",
        "space_type": "team",
        "name": "Team Space",
        "version": 1,
        "created_at_ms": 1700000000000i64,
        "updated_at_ms": 1700000000000i64
    })
}
