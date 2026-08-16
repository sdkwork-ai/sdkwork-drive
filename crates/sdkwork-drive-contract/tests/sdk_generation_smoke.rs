use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

#[test]
fn sdk_generation_scripts_reference_drive_openapi_inputs() {
    let script = std::fs::read_to_string(workspace_root().join("tools/drive_sdk_generate.mjs"))
        .expect("drive_sdk_generate script missing");
    assert!(script.contains("apis/open-api/drive/drive-open-api.openapi.json"));
    assert!(script.contains("apis/app-api/drive/drive-app-api.openapi.json"));
    assert!(script.contains("apis/backend-api/drive/drive-backend-api.openapi.json"));
    assert!(script.contains("drive-admin-storage-api.openapi.json"));
    assert!(script.contains("sdks/sdkwork-drive-admin-storage-sdk/bin/generate-sdk.mjs"));
}
