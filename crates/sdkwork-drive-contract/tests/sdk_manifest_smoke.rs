use std::path::PathBuf;

use serde_json::Value;

fn workspace_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

#[test]
fn sdk_assemblies_use_canonical_drive_sdk_profiles() {
    let open = std::fs::read_to_string(
        workspace_root().join("sdks/sdkwork-drive-sdk/bin/generate-sdk.mjs"),
    )
    .expect("open sdk generate script missing");
    let app = std::fs::read_to_string(
        workspace_root().join("sdks/sdkwork-drive-app-sdk/bin/generate-sdk.mjs"),
    )
    .expect("app sdk generate script missing");
    let backend = std::fs::read_to_string(
        workspace_root().join("sdks/sdkwork-drive-backend-sdk/bin/generate-sdk.mjs"),
    )
    .expect("backend sdk generate script missing");
    let admin_storage = std::fs::read_to_string(
        workspace_root().join("sdks/sdkwork-drive-admin-storage-sdk/bin/generate-sdk.mjs"),
    )
    .expect("admin storage sdk generate script missing");
    assert!(open.contains(r#"sdkName: "sdkwork-drive-sdk""#));
    assert!(open.contains(r#"sdkType: "custom""#));
    assert!(open.contains("standardProfileArgs: []"));
    assert!(open.contains(r#"manifestStandardProfile: "sdkwork-drive-open-v3""#));
    assert!(!open.contains("--standard-profile"));

    assert!(app.contains(r#"sdkName: "sdkwork-drive-app-sdk""#));
    assert!(app.contains(r#"sdkType: "app""#));
    assert!(app.contains("--standard-profile"));
    assert!(app.contains("sdkwork-v3"));
    assert!(app.contains(r#"manifestStandardProfile: "sdkwork-v3""#));

    assert!(backend.contains(r#"sdkName: "sdkwork-drive-backend-sdk""#));
    assert!(backend.contains(r#"sdkType: "backend""#));
    assert!(backend.contains("--standard-profile"));
    assert!(backend.contains("sdkwork-v3"));
    assert!(backend.contains(r#"manifestStandardProfile: "sdkwork-v3""#));

    assert!(admin_storage.contains(r#"sdkName: "sdkwork-drive-admin-storage-sdk""#));
    assert!(admin_storage.contains(r#"sdkType: "custom""#));
    assert!(admin_storage.contains("standardProfileArgs: []"));
    assert!(admin_storage.contains(r#"manifestStandardProfile: "sdkwork-drive-admin-storage-v3""#));
    assert!(!admin_storage.contains("--standard-profile"));

    for sdk_name in [
        "sdkwork-drive-sdk",
        "sdkwork-drive-app-sdk",
        "sdkwork-drive-backend-sdk",
        "sdkwork-drive-admin-storage-sdk",
    ] {
        assert_sdk_assembly_declares_official_languages(sdk_name);
    }
}

fn assert_sdk_assembly_declares_official_languages(sdk_name: &str) {
    let assembly_path = workspace_root().join(format!("sdks/{sdk_name}/sdk-manifest.json"));
    let assembly = std::fs::read_to_string(&assembly_path)
        .unwrap_or_else(|_| panic!("{sdk_name} assembly manifest missing"));
    let value: Value = serde_json::from_str(&assembly)
        .unwrap_or_else(|_| panic!("{sdk_name} assembly manifest should be valid json"));
    let languages = value
        .get("languages")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{sdk_name} languages should be an array"));
    let mut actual = languages
        .iter()
        .filter_map(|item| item.get("language").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = ["go", "java", "python", "rust", "typescript"]
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    expected.sort();

    assert_eq!(
        actual, expected,
        "{sdk_name} should declare every official SDK language"
    );
}
