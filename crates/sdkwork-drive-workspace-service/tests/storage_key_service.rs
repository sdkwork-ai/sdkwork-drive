use sdkwork_drive_workspace_service::application::storage_key_service::{
    BuildStorageObjectKeyCommand, DriveStorageKeyService,
};

#[test]
fn storage_key_builder_uses_stable_internal_layout() {
    let key = DriveStorageKeyService::build_object_key(BuildStorageObjectKeyCommand {
        tenant_id: "tenant-8f3a9c",
        space_id: "space-team-01",
        node_id: "node-a74392",
        version_no: 7,
        object_id: "obj-01hrz4m5nn9k9",
    })
    .expect("valid ids should build an object key");

    assert_eq!(
        key,
        "sdkwork-drive/v1/t/f0/tenants/tenant-8f3a9c/spaces/space-team-01/nodes/n/4a/node-a74392/versions/0000000007/obj-01hrz4m5nn9k9/content"
    );
}

#[test]
fn storage_key_builder_rejects_empty_required_parts() {
    let err = DriveStorageKeyService::build_object_key(BuildStorageObjectKeyCommand {
        tenant_id: "tenant-001",
        space_id: " ",
        node_id: "node-001",
        version_no: 1,
        object_id: "obj-001",
    })
    .expect_err("space_id is required");

    assert!(err.contains("space_id"));
}

#[test]
fn storage_key_builder_rejects_zero_version() {
    let err = DriveStorageKeyService::build_object_key(BuildStorageObjectKeyCommand {
        tenant_id: "tenant-001",
        space_id: "space-001",
        node_id: "node-001",
        version_no: 0,
        object_id: "obj-001",
    })
    .expect_err("version_no must be positive");

    assert!(err.contains("version_no"));
}
