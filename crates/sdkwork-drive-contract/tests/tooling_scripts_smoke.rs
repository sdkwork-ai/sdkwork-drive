use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod common;
use common::run_node_command_in;

const GLOBAL_CANONICAL_SDK_GENERATOR_ROOT: &str = r"..\sdkwork-sdk-generator";
const GLOBAL_CANONICAL_SDK_GENERATOR_BIN: &str = r"..\sdkwork-sdk-generator\bin\sdkgen.js";
const DRIVE_CANONICAL_SDK_GENERATOR_ROOT: &str = "../../sdkwork-sdk-generator";
const DRIVE_CANONICAL_SDK_GENERATOR_BIN: &str = "../../sdkwork-sdk-generator/bin/sdkgen.js";
const GLOBAL_SDK_SPEC: &str = r"sdkwork-specs\SDK_SPEC.md";
const GLOBAL_SDK_WORKSPACE_SPEC: &str = r"sdkwork-specs\SDK_WORKSPACE_GENERATION_SPEC.md";
const CODE_GENERATOR_ALIAS_RULE: &str = "sdkwork-code-generator` is only an alias/wrapper name";

fn workspace_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

fn sdkwork_root() -> PathBuf {
    workspace_root()
        .parent()
        .expect("sdkwork-drive should live below the SDKWork workspace root")
        .to_path_buf()
}

fn assert_node_script_succeeds(args: &[PathBuf]) {
    let root = workspace_root();
    let output = run_node_command_in(&root, args.iter()).expect("node command should start");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn openapi_export_supports_explicit_input_flags() {
    let root = workspace_root();
    assert_node_script_succeeds(&[
        root.join("tools/drive_openapi_export.mjs"),
        PathBuf::from("--check"),
        PathBuf::from("--open-input"),
        root.join("apis/open-api/drive/drive-open-api.openapi.json"),
        PathBuf::from("--app-input"),
        root.join("apis/app-api/drive/drive-app-api.openapi.json"),
        PathBuf::from("--backend-input"),
        root.join("apis/backend-api/drive/drive-backend-api.openapi.json"),
        PathBuf::from("--admin-storage-input"),
        root.join("apis/backend-api/drive/drive-admin-storage-api.openapi.json"),
    ]);
}

#[test]
fn openapi_export_preserves_owner_contract_and_sdkwork_v3_problem_details() {
    let root = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_millis();
    let output_dir = std::env::temp_dir().join(format!("sdkwork-drive-openapi-export-{nonce}"));

    assert_node_script_succeeds(&[
        root.join("tools/drive_openapi_export.mjs"),
        PathBuf::from("--output-dir"),
        output_dir.clone(),
        PathBuf::from("--open-input"),
        root.join("apis/open-api/drive/drive-open-api.openapi.json"),
        PathBuf::from("--app-input"),
        root.join("apis/app-api/drive/drive-app-api.openapi.json"),
        PathBuf::from("--backend-input"),
        root.join("apis/backend-api/drive/drive-backend-api.openapi.json"),
        PathBuf::from("--admin-storage-input"),
        root.join("apis/backend-api/drive/drive-admin-storage-api.openapi.json"),
    ]);

    let app: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(output_dir.join("drive-app-api.openapi.json"))
            .expect("exported app OpenAPI should be readable"),
    )
    .expect("exported app OpenAPI should be JSON");
    let label_operation = app
        .get("paths")
        .and_then(|value| value.get("/app/v3/api/drive/nodes/{nodeId}/labels"))
        .and_then(|value| value.get("get"))
        .expect("node labels operation should exist");
    assert_eq!(
        label_operation
            .get("tags")
            .and_then(serde_json::Value::as_array)
            .and_then(|values| values.first())
            .and_then(serde_json::Value::as_str),
        Some("drive"),
        "OpenAPI export should preserve the canonical Drive domain tag"
    );

    let create_permission = app
        .get("components")
        .and_then(|value| value.get("schemas"))
        .and_then(|value| value.get("CreatePermissionRequest"))
        .and_then(|value| value.get("properties"))
        .expect("CreatePermissionRequest properties should exist");
    assert!(create_permission.get("subjectType").is_some());
    assert!(create_permission.get("subjectId").is_some());

    let backend: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(output_dir.join("drive-backend-api.openapi.json"))
            .expect("exported backend OpenAPI should be readable"),
    )
    .expect("exported backend OpenAPI should be JSON");
    for schema_name in ["SweepObjectStoreRequest", "SweepUploadSessionsRequest"] {
        for context_field in ["operatorId", "requestId", "correlationId", "traceId"] {
            assert!(
                backend
                    .pointer(&format!(
                        "/components/schemas/{schema_name}/properties/{context_field}"
                    ))
                    .is_none(),
                "exported {schema_name}.{context_field} must come from request context"
            );
        }
    }

    let admin_storage: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(output_dir.join("drive-admin-storage-api.openapi.json"))
            .expect("exported admin storage OpenAPI should be readable"),
    )
    .expect("exported admin storage OpenAPI should be JSON");
    assert!(
        admin_storage
            .pointer("/components/schemas/CopyProviderObjectRequest/properties/operatorId")
            .is_none(),
        "exported CopyProviderObjectRequest.operatorId must come from request context"
    );
    for (path_key, method) in [
        (
            "/backend/v3/api/drive/storage/providers/{providerId}/bucket",
            "put",
        ),
        (
            "/backend/v3/api/drive/storage/providers/{providerId}/bucket",
            "delete",
        ),
        (
            "/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}",
            "delete",
        ),
        ("/backend/v3/api/drive/storage/bindings/default", "delete"),
    ] {
        let exposes_operator_id = admin_storage
            .get("paths")
            .and_then(|value| value.get(path_key))
            .and_then(|value| value.get(method))
            .and_then(|value| value.get("parameters"))
            .and_then(serde_json::Value::as_array)
            .is_some_and(|parameters| {
                parameters.iter().any(|parameter| {
                    parameter.get("name").and_then(serde_json::Value::as_str) == Some("operatorId")
                })
            });
        assert!(
            !exposes_operator_id,
            "exported {method} {path_key} must not accept operatorId"
        );
    }

    let bad_request = app
        .get("paths")
        .and_then(|value| value.get("/app/v3/api/drive/changes"))
        .and_then(|value| value.get("get"))
        .and_then(|value| value.get("responses"))
        .and_then(|value| value.get("400"))
        .expect("changes.list 400 response should exist");
    assert!(
        bad_request
            .get("content")
            .and_then(|value| value.get("application/problem+json"))
            .is_some(),
        "OpenAPI export should use application/problem+json for error responses"
    );
}

#[test]
fn schema_quality_gate_supports_explicit_input_flags() {
    let root = workspace_root();
    assert_node_script_succeeds(&[
        root.join("tools/drive_schema_quality_gate.mjs"),
        PathBuf::from("--open-openapi"),
        root.join("apis/open-api/drive/drive-open-api.openapi.json"),
        PathBuf::from("--app-openapi"),
        root.join("apis/app-api/drive/drive-app-api.openapi.json"),
        PathBuf::from("--backend-openapi"),
        root.join("apis/backend-api/drive/drive-backend-api.openapi.json"),
        PathBuf::from("--admin-storage-openapi"),
        root.join("apis/backend-api/drive/drive-admin-storage-api.openapi.json"),
        PathBuf::from("--special-spaces-schema"),
        root.join("docs/schema-registry/tables/002-drive-special-spaces.yaml"),
        PathBuf::from("--security-audit-schema"),
        root.join("docs/schema-registry/tables/004-drive-security-audit.yaml"),
        PathBuf::from("--storage-schema"),
        root.join("docs/schema-registry/tables/003-drive-storage.yaml"),
    ]);
}

#[test]
fn sdk_generate_check_supports_explicit_openapi_and_language_flags() {
    let root = workspace_root();
    assert_node_script_succeeds(&[
        root.join("tools/drive_sdk_generate.mjs"),
        PathBuf::from("--check"),
        PathBuf::from("--language"),
        PathBuf::from("rust"),
        PathBuf::from("--open-input"),
        root.join("apis/open-api/drive/drive-open-api.openapi.json"),
        PathBuf::from("--app-input"),
        root.join("apis/app-api/drive/drive-app-api.openapi.json"),
        PathBuf::from("--backend-input"),
        root.join("apis/backend-api/drive/drive-backend-api.openapi.json"),
        PathBuf::from("--admin-storage-input"),
        root.join("apis/backend-api/drive/drive-admin-storage-api.openapi.json"),
    ]);
}

#[test]
fn local_sdk_generator_stub_is_fail_closed_tombstone() {
    let root = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_millis();
    let output_dir = std::env::temp_dir().join(format!("sdkwork-drive-apigen-{nonce}"));

    let output = run_node_command_in(
        &root,
        [
            root.join("tools/sdkwork_sdk_generator_stub.mjs"),
            PathBuf::from("generate"),
            PathBuf::from("--input"),
            root.join("apis/app-api/drive/drive-app-api.openapi.json"),
            PathBuf::from("--output"),
            output_dir.clone(),
            PathBuf::from("--name"),
            PathBuf::from("sdkwork-drive-app-sdk"),
            PathBuf::from("--type"),
            PathBuf::from("app"),
            PathBuf::from("--language"),
            PathBuf::from("typescript"),
            PathBuf::from("--base-url"),
            PathBuf::from("http://127.0.0.1:18080"),
            PathBuf::from("--api-prefix"),
            PathBuf::from("/app/v3/api"),
            PathBuf::from("--fixed-sdk-version"),
            PathBuf::from("0.1.0"),
            PathBuf::from("--sdk-root"),
            root.join("sdks/sdkwork-drive-app-sdk"),
            PathBuf::from("--sdk-name"),
            PathBuf::from("sdkwork-drive-app-sdk"),
            PathBuf::from("--package-name"),
            PathBuf::from("sdkwork-drive-app-sdk-generated-typescript"),
            PathBuf::from("--standard-profile"),
            PathBuf::from("sdkwork-v3"),
        ],
    )
    .expect("node command should start");

    assert!(
        !output.status.success(),
        "local SDK generator tombstone must fail closed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output_text.contains("canonical sdkgen"),
        "local SDK generator tombstone should direct callers to canonical sdkgen, stdout/stderr:\n{output_text}"
    );
    assert!(
        !output_dir.join("sdk-manifest.json").exists(),
        "tombstone must not generate an SDK manifest",
    );
    assert!(
        !output_dir.join("source-openapi.json").exists(),
        "tombstone must not generate an OpenAPI snapshot",
    );
    assert!(
        !output_dir.join("src/index.ts").exists(),
        "tombstone must not generate a TypeScript entrypoint",
    );
}

#[test]
fn sdk_family_generators_prefer_canonical_sibling_sdk_generator() {
    let root = workspace_root();
    let runner_path = root.join("tools/drive_sdk_generator_runner.mjs");
    let runner = std::fs::read_to_string(&runner_path)
        .unwrap_or_else(|_| panic!("{} should be readable", runner_path.display()));
    assert!(
        runner.contains("resolveSdkGeneratorInvocation"),
        "shared SDK generator runner should expose resolveSdkGeneratorInvocation"
    );
    assert!(
        runner.contains(r#""..""#)
            && runner.contains("sdkwork-sdk-generator")
            && runner.contains("sdkgen.js"),
        "shared SDK generator runner should use the canonical sibling SDK generator path"
    );
    let retired_unix_dependency_root = [".sdkwork", "dependencies"].join("/");
    let retired_windows_dependency_root = [".sdkwork", "dependencies"].join("\\");
    assert!(
        !runner.contains(r#"" .sdkwork", "dependencies"#)
            && !runner.contains(r#"".sdkwork", "dependencies"#)
            && !runner.contains(&retired_unix_dependency_root)
            && !runner.contains(&retired_windows_dependency_root),
        "shared SDK generator runner must not use root .sdkwork as a dependency source"
    );
    assert!(
        !runner.contains("legacy-java-plus-workspace"),
        "shared SDK generator runner must not use the deprecated legacy-java-plus-workspace generator path"
    );

    for script_path in [
        "sdks/sdkwork-drive-sdk/bin/generate-sdk.mjs",
        "sdks/sdkwork-drive-app-sdk/bin/generate-sdk.mjs",
        "sdks/sdkwork-drive-backend-sdk/bin/generate-sdk.mjs",
        "sdks/sdkwork-drive-admin-storage-sdk/bin/generate-sdk.mjs",
    ] {
        let script = std::fs::read_to_string(root.join(script_path))
            .unwrap_or_else(|_| panic!("{script_path} should be readable"));
        assert!(
            script.contains("runDriveSdkGenerator"),
            "{script_path} should use the shared SDK generator runner"
        );
        assert!(
            !script.contains("legacy-java-plus-workspace"),
            "{script_path} must not use the deprecated legacy-java-plus-workspace generator path"
        );
    }
}

#[test]
fn sdk_standards_require_canonical_sdkgen_and_code_generator_alias_policy() {
    let root = workspace_root();
    for (name, path, generator_root, generator_bin) in [
        (
            "SDK_SPEC.md",
            sdkwork_root().join(GLOBAL_SDK_SPEC),
            GLOBAL_CANONICAL_SDK_GENERATOR_ROOT,
            GLOBAL_CANONICAL_SDK_GENERATOR_BIN,
        ),
        (
            "SDK_WORKSPACE_GENERATION_SPEC.md",
            sdkwork_root().join(GLOBAL_SDK_WORKSPACE_SPEC),
            GLOBAL_CANONICAL_SDK_GENERATOR_ROOT,
            GLOBAL_CANONICAL_SDK_GENERATOR_BIN,
        ),
        (
            "TECH-drive-sdk-integration-standard.md",
            root.join("docs/architecture/tech/TECH-drive-sdk-integration-standard.md"),
            DRIVE_CANONICAL_SDK_GENERATOR_ROOT,
            DRIVE_CANONICAL_SDK_GENERATOR_BIN,
        ),
    ] {
        let document = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{name} should be readable at {}", path.display()));
        assert!(
            document.contains(generator_root),
            "{name} should declare the canonical SDK generator root"
        );
        assert!(
            document.contains(generator_bin),
            "{name} should declare the canonical sdkgen.js entrypoint"
        );
        assert!(
            document.contains("@sdkwork/sdk-generator") && document.contains("sdkgen"),
            "{name} should identify the SDK generator package and CLI"
        );
        assert!(
            document.contains(CODE_GENERATOR_ALIAS_RULE),
            "{name} should state that sdkwork-code-generator is only a wrapper alias for canonical sdkgen.js"
        );
    }
}

#[test]
fn generated_drive_sdks_use_canonical_family_names_and_sdkgen_manifests() {
    let root = workspace_root();
    let families = [
        (
            "sdkwork-drive-sdk",
            "custom",
            "sdkwork-drive-open-v3",
            "/open/v3/api",
        ),
        ("sdkwork-drive-app-sdk", "app", "sdkwork-v3", "/app/v3/api"),
        (
            "sdkwork-drive-backend-sdk",
            "backend",
            "sdkwork-v3",
            "/backend/v3/api",
        ),
        (
            "sdkwork-drive-admin-storage-sdk",
            "custom",
            "sdkwork-drive-admin-storage-v3",
            "/backend/v3/api",
        ),
    ];
    let languages = ["typescript", "rust", "java", "python", "go"];

    let sdks_dir = root.join("sdks");
    let top_level_sdk_dirs: Vec<String> = std::fs::read_dir(&sdks_dir)
        .unwrap_or_else(|_| panic!("{} should be readable", sdks_dir.display()))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    for forbidden in [
        "sdkwork-drive-open-api",
        "sdkwork-drive-app-api",
        "sdkwork-drive-backend-api",
        "sdkwork-drive-admin-storage-api",
        "drive-open-sdk",
        "drive-app-sdk",
        "drive-backend-sdk",
        "drive-admin-storage-sdk",
    ] {
        assert!(
            !top_level_sdk_dirs.iter().any(|name| name == forbidden),
            "forbidden SDK family directory should not exist under sdks/: {forbidden}"
        );
    }

    for (sdk_name, sdk_type, standard_profile, api_prefix) in families {
        let family_manifest_path = root.join("sdks").join(sdk_name).join("sdk-manifest.json");
        let family_manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&family_manifest_path).unwrap_or_else(|_| {
                panic!("{} should be readable", family_manifest_path.display())
            }),
        )
        .unwrap_or_else(|_| panic!("{} should be JSON", family_manifest_path.display()));
        assert_eq!(
            family_manifest
                .get("sdkName")
                .and_then(serde_json::Value::as_str),
            Some(sdk_name),
            "{sdk_name} should use canonical SDK family name"
        );
        assert_eq!(
            family_manifest
                .get("generatorName")
                .and_then(serde_json::Value::as_str),
            Some("sdkwork-sdk-generator"),
            "{sdk_name} should record the canonical generator name"
        );
        assert_eq!(
            family_manifest
                .get("sdkType")
                .and_then(serde_json::Value::as_str),
            Some(sdk_type),
            "{sdk_name} should record the expected SDK type"
        );
        assert_eq!(
            family_manifest
                .get("standardProfile")
                .and_then(serde_json::Value::as_str),
            Some(standard_profile),
            "{sdk_name} should record the expected standard profile"
        );
        assert_eq!(
            family_manifest
                .get("apiPrefix")
                .and_then(serde_json::Value::as_str),
            Some(api_prefix),
            "{sdk_name} should record the expected API prefix"
        );

        for language in languages {
            let output_root = root
                .join("sdks")
                .join(sdk_name)
                .join(format!("{sdk_name}-{language}"))
                .join("generated/server-openapi");
            let sdkwork_manifest_path = output_root.join("sdkwork-sdk.json");
            let generator_manifest_path = output_root
                .join(".sdkwork")
                .join("sdkwork-generator-manifest.json");
            let generator_changes_path = output_root
                .join(".sdkwork")
                .join("sdkwork-generator-changes.json");
            let generator_report_path = output_root
                .join(".sdkwork")
                .join("sdkwork-generator-report.json");
            let custom_root = output_root.join("custom");

            for required_path in [
                &sdkwork_manifest_path,
                &generator_manifest_path,
                &generator_changes_path,
                &generator_report_path,
                &custom_root,
            ] {
                assert!(
                    required_path.exists(),
                    "{} should retain required sdkgen output/control-plane path",
                    required_path.display()
                );
            }
            assert!(
                !output_root.join("sdk-manifest.json").exists(),
                "{sdk_name}/{language} generated output must not carry SDK ownership manifest"
            );
            assert_eq!(
                family_manifest
                    .get("generatedPackages")
                    .and_then(|value| value.get(language))
                    .and_then(|value| value.get("packageName"))
                    .and_then(serde_json::Value::as_str),
                Some(format!("{sdk_name}-generated-{language}").as_str()),
                "{sdk_name}/{language} should use SDK-family-based generated package name"
            );

            let sdkwork_manifest: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(&sdkwork_manifest_path).unwrap_or_else(|_| {
                    panic!("{} should be readable", sdkwork_manifest_path.display())
                }),
            )
            .unwrap_or_else(|_| panic!("{} should be JSON", sdkwork_manifest_path.display()));
            assert_eq!(
                sdkwork_manifest
                    .get("generator")
                    .and_then(serde_json::Value::as_str),
                Some("@sdkwork/sdk-generator"),
                "{sdk_name}/{language} should retain sdkgen generator package metadata"
            );
            assert_eq!(
                sdkwork_manifest
                    .get("name")
                    .and_then(serde_json::Value::as_str),
                Some(sdk_name),
                "{sdk_name}/{language} sdkwork manifest should use SDK family name"
            );
            assert_eq!(
                sdkwork_manifest
                    .get("packageName")
                    .and_then(serde_json::Value::as_str),
                Some(format!("{sdk_name}-generated-{language}").as_str()),
                "{sdk_name}/{language} sdkwork manifest should use SDK-family-based generated package name"
            );

            let output_text =
                std::fs::read_to_string(&sdkwork_manifest_path).unwrap_or_else(|_| {
                    panic!("{} should be readable", sdkwork_manifest_path.display())
                });
            assert!(
                !output_text.contains("sdkwork_sdk_generator_stub")
                    && !output_text.contains("sdkwork-code-generator"),
                "{sdk_name}/{language} generated metadata must not name a stub or ambiguous code generator"
            );
        }
    }
}

#[test]
fn schema_quality_gate_fails_when_maintenance_job_enum_is_missing() {
    let root = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("sdkwork-drive-schema-gate-{nonce}"));
    std::fs::create_dir_all(&temp_dir).expect("temp directory should be created");

    let app_src = root.join("apis/app-api/drive/drive-app-api.openapi.json");
    let backend_src = root.join("apis/backend-api/drive/drive-backend-api.openapi.json");
    let app_dst = temp_dir.join("drive-app-api.openapi.json");
    let backend_dst = temp_dir.join("drive-backend-api.openapi.json");
    std::fs::copy(&app_src, &app_dst).expect("app openapi should be copied");

    let backend_raw =
        std::fs::read_to_string(&backend_src).expect("backend openapi should be readable");
    let mut backend_json: serde_json::Value =
        serde_json::from_str(&backend_raw).expect("backend openapi should be valid json");
    let job_type = backend_json
        .get_mut("components")
        .and_then(|value| value.get_mut("schemas"))
        .and_then(|value| value.get_mut("MaintenanceJob"))
        .and_then(|value| value.get_mut("properties"))
        .and_then(|value| value.get_mut("jobType"))
        .expect("MaintenanceJob.jobType schema should exist");
    if let Some(object) = job_type.as_object_mut() {
        object.remove("enum");
    }
    let patched = serde_json::to_string_pretty(&backend_json)
        .expect("patched backend openapi should be serializable");
    std::fs::write(&backend_dst, format!("{patched}\n"))
        .expect("patched backend openapi should be writable");

    let output = run_node_command_in(
        &root,
        [
            root.join("tools/drive_schema_quality_gate.mjs"),
            PathBuf::from("--app-openapi"),
            app_dst.clone(),
            PathBuf::from("--backend-openapi"),
            backend_dst.clone(),
            PathBuf::from("--special-spaces-schema"),
            root.join("docs/schema-registry/tables/002-drive-special-spaces.yaml"),
            PathBuf::from("--security-audit-schema"),
            root.join("docs/schema-registry/tables/004-drive-security-audit.yaml"),
            PathBuf::from("--storage-schema"),
            root.join("docs/schema-registry/tables/003-drive-storage.yaml"),
        ],
    )
    .expect("schema quality gate command should start");
    assert!(
        !output.status.success(),
        "schema quality gate should fail when enum is missing, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn schema_quality_gate_fails_when_maintenance_jobs_query_enum_is_missing() {
    let root = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("sdkwork-drive-schema-gate-query-{nonce}"));
    std::fs::create_dir_all(&temp_dir).expect("temp directory should be created");

    let app_src = root.join("apis/app-api/drive/drive-app-api.openapi.json");
    let backend_src = root.join("apis/backend-api/drive/drive-backend-api.openapi.json");
    let app_dst = temp_dir.join("drive-app-api.openapi.json");
    let backend_dst = temp_dir.join("drive-backend-api.openapi.json");
    std::fs::copy(&app_src, &app_dst).expect("app openapi should be copied");

    let backend_raw =
        std::fs::read_to_string(&backend_src).expect("backend openapi should be readable");
    let mut backend_json: serde_json::Value =
        serde_json::from_str(&backend_raw).expect("backend openapi should be valid json");
    let parameters = backend_json
        .get_mut("paths")
        .and_then(|value| value.get_mut("/backend/v3/api/drive/maintenance/jobs"))
        .and_then(|value| value.get_mut("get"))
        .and_then(|value| value.get_mut("parameters"))
        .and_then(serde_json::Value::as_array_mut)
        .expect("maintenance jobs get parameters should exist");
    let status_param = parameters
        .iter_mut()
        .find(|item| item.get("name").and_then(serde_json::Value::as_str) == Some("status"))
        .expect("maintenance jobs status query parameter should exist");
    if let Some(schema) = status_param
        .get_mut("schema")
        .and_then(serde_json::Value::as_object_mut)
    {
        schema.remove("enum");
    }
    let patched = serde_json::to_string_pretty(&backend_json)
        .expect("patched backend openapi should be serializable");
    std::fs::write(&backend_dst, format!("{patched}\n"))
        .expect("patched backend openapi should be writable");

    let output = run_node_command_in(
        &root,
        [
            root.join("tools/drive_schema_quality_gate.mjs"),
            PathBuf::from("--app-openapi"),
            app_dst.clone(),
            PathBuf::from("--backend-openapi"),
            backend_dst.clone(),
            PathBuf::from("--special-spaces-schema"),
            root.join("docs/schema-registry/tables/002-drive-special-spaces.yaml"),
            PathBuf::from("--security-audit-schema"),
            root.join("docs/schema-registry/tables/004-drive-security-audit.yaml"),
            PathBuf::from("--storage-schema"),
            root.join("docs/schema-registry/tables/003-drive-storage.yaml"),
        ],
    )
    .expect("schema quality gate command should start");
    assert!(
        !output.status.success(),
        "schema quality gate should fail when maintenance jobs query enum is missing, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn schema_quality_gate_fails_when_maintenance_jobs_operator_id_pattern_is_missing() {
    let root = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("sdkwork-drive-schema-gate-pattern-{nonce}"));
    std::fs::create_dir_all(&temp_dir).expect("temp directory should be created");

    let app_src = root.join("apis/app-api/drive/drive-app-api.openapi.json");
    let backend_src = root.join("apis/backend-api/drive/drive-backend-api.openapi.json");
    let app_dst = temp_dir.join("drive-app-api.openapi.json");
    let backend_dst = temp_dir.join("drive-backend-api.openapi.json");
    std::fs::copy(&app_src, &app_dst).expect("app openapi should be copied");

    let backend_raw =
        std::fs::read_to_string(&backend_src).expect("backend openapi should be readable");
    let mut backend_json: serde_json::Value =
        serde_json::from_str(&backend_raw).expect("backend openapi should be valid json");
    let parameters = backend_json
        .get_mut("paths")
        .and_then(|value| value.get_mut("/backend/v3/api/drive/maintenance/jobs"))
        .and_then(|value| value.get_mut("get"))
        .and_then(|value| value.get_mut("parameters"))
        .and_then(serde_json::Value::as_array_mut)
        .expect("maintenance jobs get parameters should exist");
    let operator_param = parameters
        .iter_mut()
        .find(|item| item.get("name").and_then(serde_json::Value::as_str) == Some("operatorId"))
        .expect("maintenance jobs operatorId query parameter should exist");
    if let Some(schema) = operator_param
        .get_mut("schema")
        .and_then(serde_json::Value::as_object_mut)
    {
        schema.remove("pattern");
    }
    let patched = serde_json::to_string_pretty(&backend_json)
        .expect("patched backend openapi should be serializable");
    std::fs::write(&backend_dst, format!("{patched}\n"))
        .expect("patched backend openapi should be writable");

    let output = run_node_command_in(
        &root,
        [
            root.join("tools/drive_schema_quality_gate.mjs"),
            PathBuf::from("--app-openapi"),
            app_dst.clone(),
            PathBuf::from("--backend-openapi"),
            backend_dst.clone(),
            PathBuf::from("--special-spaces-schema"),
            root.join("docs/schema-registry/tables/002-drive-special-spaces.yaml"),
            PathBuf::from("--security-audit-schema"),
            root.join("docs/schema-registry/tables/004-drive-security-audit.yaml"),
            PathBuf::from("--storage-schema"),
            root.join("docs/schema-registry/tables/003-drive-storage.yaml"),
        ],
    )
    .expect("schema quality gate command should start");
    assert!(
        !output.status.success(),
        "schema quality gate should fail when operatorId pattern is missing, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn schema_quality_gate_fails_when_audit_events_correlation_id_pattern_is_missing() {
    let root = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_millis();
    let temp_dir =
        std::env::temp_dir().join(format!("sdkwork-drive-audit-schema-gate-pattern-{nonce}"));
    std::fs::create_dir_all(&temp_dir).expect("temp directory should be created");

    let app_src = root.join("apis/app-api/drive/drive-app-api.openapi.json");
    let backend_src = root.join("apis/backend-api/drive/drive-backend-api.openapi.json");
    let app_dst = temp_dir.join("drive-app-api.openapi.json");
    let backend_dst = temp_dir.join("drive-backend-api.openapi.json");
    std::fs::copy(&app_src, &app_dst).expect("app openapi should be copied");

    let backend_raw =
        std::fs::read_to_string(&backend_src).expect("backend openapi should be readable");
    let mut backend_json: serde_json::Value =
        serde_json::from_str(&backend_raw).expect("backend openapi should be valid json");
    let parameters = backend_json
        .get_mut("paths")
        .and_then(|value| value.get_mut("/backend/v3/api/drive/audit_events"))
        .and_then(|value| value.get_mut("get"))
        .and_then(|value| value.get_mut("parameters"))
        .and_then(serde_json::Value::as_array_mut)
        .expect("audit events get parameters should exist");
    let correlation_id_param = parameters
        .iter_mut()
        .find(|item| item.get("name").and_then(serde_json::Value::as_str) == Some("correlationId"))
        .expect("audit events correlationId query parameter should exist");
    if let Some(schema) = correlation_id_param
        .get_mut("schema")
        .and_then(serde_json::Value::as_object_mut)
    {
        schema.remove("pattern");
    }
    let patched = serde_json::to_string_pretty(&backend_json)
        .expect("patched backend openapi should be serializable");
    std::fs::write(&backend_dst, format!("{patched}\n"))
        .expect("patched backend openapi should be writable");

    let output = run_node_command_in(
        &root,
        [
            root.join("tools/drive_schema_quality_gate.mjs"),
            PathBuf::from("--app-openapi"),
            app_dst.clone(),
            PathBuf::from("--backend-openapi"),
            backend_dst.clone(),
            PathBuf::from("--special-spaces-schema"),
            root.join("docs/schema-registry/tables/002-drive-special-spaces.yaml"),
            PathBuf::from("--security-audit-schema"),
            root.join("docs/schema-registry/tables/004-drive-security-audit.yaml"),
            PathBuf::from("--storage-schema"),
            root.join("docs/schema-registry/tables/003-drive-storage.yaml"),
        ],
    )
    .expect("schema quality gate command should start");
    assert!(
        !output.status.success(),
        "schema quality gate should fail when audit_events correlationId pattern is missing, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn schema_quality_gate_fails_when_audit_index_is_missing_in_registry() {
    let root = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("sdkwork-drive-audit-index-gate-{nonce}"));
    std::fs::create_dir_all(&temp_dir).expect("temp directory should be created");

    let app_src = root.join("apis/app-api/drive/drive-app-api.openapi.json");
    let backend_src = root.join("apis/backend-api/drive/drive-backend-api.openapi.json");
    let special_spaces_src = root.join("docs/schema-registry/tables/002-drive-special-spaces.yaml");
    let security_audit_src = root.join("docs/schema-registry/tables/004-drive-security-audit.yaml");
    let storage_schema_src = root.join("docs/schema-registry/tables/003-drive-storage.yaml");
    let app_dst = temp_dir.join("drive-app-api.openapi.json");
    let backend_dst = temp_dir.join("drive-backend-api.openapi.json");
    let special_spaces_dst = temp_dir.join("002-drive-special-spaces.yaml");
    let security_audit_dst = temp_dir.join("004-drive-security-audit.yaml");
    let storage_schema_dst = temp_dir.join("003-drive-storage.yaml");
    std::fs::copy(&app_src, &app_dst).expect("app openapi should be copied");
    std::fs::copy(&backend_src, &backend_dst).expect("backend openapi should be copied");
    std::fs::copy(&special_spaces_src, &special_spaces_dst)
        .expect("special spaces schema should be copied");
    std::fs::copy(&storage_schema_src, &storage_schema_dst)
        .expect("storage schema should be copied");

    let audit_schema_raw = std::fs::read_to_string(&security_audit_src)
        .expect("security audit schema should be readable");
    let patched = audit_schema_raw.replace(
        "ix_dr_drive_audit_event_trace_created",
        "ix_dr_drive_audit_event_trace_removed",
    );
    std::fs::write(&security_audit_dst, patched)
        .expect("patched security audit schema should be writable");

    let output = run_node_command_in(
        &root,
        [
            root.join("tools/drive_schema_quality_gate.mjs"),
            PathBuf::from("--app-openapi"),
            app_dst.clone(),
            PathBuf::from("--backend-openapi"),
            backend_dst.clone(),
            PathBuf::from("--special-spaces-schema"),
            special_spaces_dst.clone(),
            PathBuf::from("--security-audit-schema"),
            security_audit_dst.clone(),
            PathBuf::from("--storage-schema"),
            storage_schema_dst.clone(),
        ],
    )
    .expect("schema quality gate command should start");
    assert!(
        !output.status.success(),
        "schema quality gate should fail when audit index is missing in registry, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn schema_quality_gate_fails_when_storage_provider_kind_pattern_is_missing() {
    let root = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("sdkwork-drive-storage-kind-gate-{nonce}"));
    std::fs::create_dir_all(&temp_dir).expect("temp directory should be created");

    let app_src = root.join("apis/app-api/drive/drive-app-api.openapi.json");
    let backend_src = root.join("apis/backend-api/drive/drive-backend-api.openapi.json");
    let admin_storage_src =
        root.join("apis/backend-api/drive/drive-admin-storage-api.openapi.json");
    let special_spaces_src = root.join("docs/schema-registry/tables/002-drive-special-spaces.yaml");
    let security_audit_src = root.join("docs/schema-registry/tables/004-drive-security-audit.yaml");
    let storage_schema_src = root.join("docs/schema-registry/tables/003-drive-storage.yaml");
    let app_dst = temp_dir.join("drive-app-api.openapi.json");
    let backend_dst = temp_dir.join("drive-backend-api.openapi.json");
    let admin_storage_dst = temp_dir.join("drive-admin-storage-api.openapi.json");
    let special_spaces_dst = temp_dir.join("002-drive-special-spaces.yaml");
    let security_audit_dst = temp_dir.join("004-drive-security-audit.yaml");
    let storage_schema_dst = temp_dir.join("003-drive-storage.yaml");
    std::fs::copy(&app_src, &app_dst).expect("app openapi should be copied");
    std::fs::copy(&backend_src, &backend_dst).expect("backend openapi should be copied");
    std::fs::copy(&admin_storage_src, &admin_storage_dst)
        .expect("admin storage openapi should be copied");
    std::fs::copy(&special_spaces_src, &special_spaces_dst)
        .expect("special spaces schema should be copied");
    std::fs::copy(&security_audit_src, &security_audit_dst)
        .expect("security audit schema should be copied");
    std::fs::copy(&storage_schema_src, &storage_schema_dst)
        .expect("storage schema should be copied");

    let admin_storage_raw = std::fs::read_to_string(&admin_storage_dst)
        .expect("admin storage openapi should be readable");
    let mut admin_storage_json: serde_json::Value = serde_json::from_str(&admin_storage_raw)
        .expect("admin storage openapi should be valid json");
    let provider_kind = admin_storage_json
        .get_mut("components")
        .and_then(|value| value.get_mut("schemas"))
        .and_then(|value| value.get_mut("CreateStorageProviderRequest"))
        .and_then(|value| value.get_mut("properties"))
        .and_then(|value| value.get_mut("providerKind"))
        .expect("CreateStorageProviderRequest.providerKind schema should exist");
    if let Some(object) = provider_kind.as_object_mut() {
        object.remove("pattern");
    }
    let patched = serde_json::to_string_pretty(&admin_storage_json)
        .expect("patched admin storage openapi should be serializable");
    std::fs::write(&admin_storage_dst, format!("{patched}\n"))
        .expect("patched admin storage openapi should be writable");

    let output = run_node_command_in(
        &root,
        [
            root.join("tools/drive_schema_quality_gate.mjs"),
            PathBuf::from("--app-openapi"),
            app_dst.clone(),
            PathBuf::from("--backend-openapi"),
            backend_dst.clone(),
            PathBuf::from("--admin-storage-openapi"),
            admin_storage_dst.clone(),
            PathBuf::from("--special-spaces-schema"),
            special_spaces_dst.clone(),
            PathBuf::from("--security-audit-schema"),
            security_audit_dst.clone(),
            PathBuf::from("--storage-schema"),
            storage_schema_dst.clone(),
        ],
    )
    .expect("schema quality gate command should start");
    assert!(
        !output.status.success(),
        "schema quality gate should fail when storage provider kind pattern is missing, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
