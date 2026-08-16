use serde_json::json;

/// Create a test file node fixture.
pub fn create_test_file_node(node_id: &str, space_id: &str) -> serde_json::Value {
    json!({
        "id": node_id,
        "space_id": space_id,
        "parent_id": null,
        "node_type": "file",
        "name": "test-file.txt",
        "version": 1,
        "content_state": "active",
        "created_at_ms": 1700000000000i64,
        "updated_at_ms": 1700000000000i64
    })
}

/// Create a test folder node fixture.
pub fn create_test_folder_node(node_id: &str, space_id: &str) -> serde_json::Value {
    json!({
        "id": node_id,
        "space_id": space_id,
        "parent_id": null,
        "node_type": "folder",
        "name": "test-folder",
        "version": 1,
        "content_state": "active",
        "created_at_ms": 1700000000000i64,
        "updated_at_ms": 1700000000000i64
    })
}
