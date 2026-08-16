use std::path::{Path, PathBuf};

fn read_rust_source_tree(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    let mut entries = std::fs::read_dir(path)
        .unwrap_or_else(|error| {
            panic!(
                "failed to read source directory {}: {error}",
                path.display()
            )
        })
        .map(|entry| {
            entry
                .expect("source directory entry should be readable")
                .path()
        })
        .collect::<Vec<_>>();
    entries.sort();

    let mut source = String::new();
    for entry in entries {
        if entry.is_dir() {
            source.push_str(&read_rust_source_tree(entry));
        } else if entry.extension().is_some_and(|extension| extension == "rs") {
            source.push_str(
                &std::fs::read_to_string(&entry)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", entry.display())),
            );
            source.push('\n');
        }
    }
    source
}

#[test]
fn repository_root_declares_sdkwork_standard_directory_dictionary() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();

    for required_path in [
        ".sdkwork/README.md",
        ".sdkwork/.gitignore",
        ".sdkwork/skills/README.md",
        ".sdkwork/plugins/README.md",
        "apis/README.md",
        "apis/open-api/drive",
        "apis/app-api/drive",
        "apis/backend-api/drive",
        "etc/README.md",
        "etc/drive.database.example.toml",
        "deployments/README.md",
        "deployments/docker-compose.minio-test.yml",
        "jobs/README.md",
        "plugins/README.md",
        "examples/README.md",
        "tests/README.md",
        "sdks/README.md",
    ] {
        assert!(
            root.join(required_path).exists(),
            "SDKWork standard root path must exist: {required_path}"
        );
    }

    let read = |relative_path: &str| {
        let path = root.join(relative_path);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()))
            .replace("\r\n", "\n")
    };

    let readme = read("README.md");
    for required in [
        "SDKWork standard project root",
        "`apis/` contains Drive-owned API contract sources and materialized OpenAPI inputs",
        "`crates/` contains Rust service crates, route crates, workers, host/server crates, and reusable Rust libraries",
        "`sdks/` contains SDK family workspaces and generated SDK output",
        "`etc/` contains safe checked-in source config templates",
        "`deployments/` contains deployment descriptors",
    ] {
        assert!(
            readme.contains(required),
            "root README must document the standard workspace layout: {required}"
        );
    }

    let agents = read("AGENTS.md");
    for required in [
        "`apis/`: Drive-owned API authorities and OpenAPI inputs.",
        "`crates/`: Rust libraries and process hosts.",
        "`etc/`: safe source configuration and profile instances.",
        "`deployments/`: deployment and infrastructure descriptors.",
    ] {
        assert!(
            agents.contains(required),
            "AGENTS.md must document standard root dictionary entry: {required}"
        );
    }

    let sdkwork_readme = read(".sdkwork/README.md");
    for required in [
        "repository/application development metadata",
        "../../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md",
        "not runtime state",
    ] {
        assert!(
            sdkwork_readme.contains(required),
            ".sdkwork README must document workspace metadata boundary: {required}"
        );
    }
}

#[test]
fn workspace_declares_expected_members() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    let manifest_path = root.join("Cargo.toml");
    let manifest = std::fs::read_to_string(manifest_path).expect("Cargo.toml must exist");
    assert!(manifest.contains("crates/sdkwork-drive-contract"));
    assert!(manifest.contains("crates/sdkwork-drive-workspace-service"));
    assert!(manifest.contains("crates/sdkwork-routes-drive-open-api"));
    assert!(manifest.contains("crates/sdkwork-routes-drive-app-api"));
    assert!(manifest.contains("crates/sdkwork-routes-drive-backend-api"));
    assert!(manifest.contains("crates/sdkwork-drive-storage-opendal"));
    assert!(manifest.contains("crates/sdkwork-routes-storage-backend-api"));
    assert!(!manifest.contains("services/"));
    assert!(!manifest.contains("services/sdkwork-drive-admin-api"));
    assert!(!manifest.contains("sdkwork-drive-product"));
    assert!(!manifest.contains("sdkwork-drive-core"));
}

#[test]
fn root_app_manifest_and_component_spec_do_not_keep_migration_baggage() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();

    let app_manifest_path = root.join("sdkwork.app.config.json");
    let app_manifest_raw = std::fs::read_to_string(&app_manifest_path).unwrap_or_else(|error| {
        panic!("{} must be readable: {error}", app_manifest_path.display())
    });
    let app_manifest: serde_json::Value =
        serde_json::from_str(&app_manifest_raw).expect("root sdkwork.app.config.json must be JSON");

    assert_eq!(
        app_manifest
            .pointer("/app/key")
            .and_then(serde_json::Value::as_str),
        Some("sdkwork-drive"),
        "root app manifest remains the sdkwork-drive workspace identity"
    );
    assert_eq!(
        app_manifest
            .pointer("/publish/config/workspaceRoot")
            .and_then(serde_json::Value::as_str),
        Some("."),
        "root publish workspaceRoot must point at the repository/application root"
    );
    assert_eq!(
        app_manifest
            .pointer("/artifacts/installConfig/metadata/workspaceRoot")
            .and_then(serde_json::Value::as_str),
        Some("."),
        "root install metadata workspaceRoot must point at the repository/application root"
    );
    assert_eq!(
        app_manifest
            .pointer("/devApp/sourceRoot")
            .and_then(serde_json::Value::as_str),
        Some("."),
        "root devApp sourceRoot must point at the repository/application root"
    );
    for relative_path in [
        app_manifest
            .pointer("/publish/config/workspaceRoot")
            .and_then(serde_json::Value::as_str)
            .expect("workspaceRoot should exist"),
        app_manifest
            .pointer("/devApp/sourceRoot")
            .and_then(serde_json::Value::as_str)
            .expect("sourceRoot should exist"),
    ] {
        assert!(
            root.join(relative_path).exists(),
            "root manifest path must resolve: {relative_path}"
        );
    }
    assert!(
        !app_manifest_raw.contains("product screenshot")
            && !app_manifest_raw.contains("migratedFromLegacyConfig"),
        "root app manifest must not retain product/legacy migration wording"
    );

    let component_spec_path = root.join("specs/component.spec.json");
    let component_spec_raw =
        std::fs::read_to_string(&component_spec_path).unwrap_or_else(|error| {
            panic!(
                "{} must be readable: {error}",
                component_spec_path.display()
            )
        });
    let component_spec: serde_json::Value =
        serde_json::from_str(&component_spec_raw).expect("root component spec must be JSON");
    assert_eq!(
        component_spec
            .pointer("/component/type")
            .and_then(serde_json::Value::as_str),
        Some("app"),
        "root component spec should describe the Drive app/workspace root, not one Rust crate"
    );
    assert!(
        component_spec
            .pointer("/component/languages")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|languages| {
                languages
                    .iter()
                    .any(|language| language.as_str() == Some("rust"))
                    && languages
                        .iter()
                        .any(|language| language.as_str() == Some("typescript"))
            }),
        "root component spec must declare both Rust and TypeScript ownership"
    );
}

#[test]
fn opendal_storage_plugin_is_not_enabled_by_default_in_api_crates() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();

    for manifest in [
        root.join("crates/sdkwork-routes-drive-app-api/Cargo.toml"),
        root.join("crates/sdkwork-routes-drive-open-api/Cargo.toml"),
        root.join("crates/sdkwork-routes-drive-backend-api/Cargo.toml"),
    ] {
        let source = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|error| panic!("{} must be readable: {error}", manifest.display()));
        assert!(
            !source.contains("sdkwork-drive-storage-opendal"),
            "{} must not depend on the OpenDAL plugin unless an explicit feature wires it in",
            manifest.display()
        );
    }
}

#[test]
fn s3_architecture_document_defines_plugin_and_admin_storage_boundaries() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    let doc_path = root.join("docs/architecture/tech/TECH-storage-s3-architecture.md");
    let doc = std::fs::read_to_string(&doc_path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", doc_path.display()));

    for required in [
        "crates/sdkwork-drive-storage-opendal",
        "OpenDAL S3 plugin is optional",
        "default disabled",
        "opendal-s3-plugin",
        "SDKWORK_DRIVE_ADMIN_STORAGE_OBJECT_STORE_ADAPTER",
        "Bucket administration remains on the AWS SDK S3 adapter",
        "crates/sdkwork-routes-storage-backend-api",
        "/backend/v3/api/drive/storage/providers",
        "Volcengine TOS",
        "sdkwork-routes-storage-backend-api",
    ] {
        assert!(
            doc.contains(required),
            "S3 architecture document must define {required}"
        );
    }
}

#[test]
fn admin_storage_router_exposes_explicit_plugin_config_entrypoints() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();

    let source = read_rust_source_tree(root.join("crates/sdkwork-routes-storage-backend-api/src"));

    for required in [
        "pub struct AdminStorageConfig",
        "DriveAdminStorageObjectStoreAdapter",
        "build_router_with_pool_and_config",
        "build_router_with_database_url_and_admin_storage_config",
        "build_router_with_database_config_and_admin_storage_config",
        "AdminStorageConfig::from_env()",
    ] {
        assert!(
            source.contains(required),
            "admin-storage router must expose explicit plugin config entrypoint {required}"
        );
    }
}

#[test]
fn admin_storage_api_has_standalone_binary_entrypoint() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();

    let main_path = root.join("crates/sdkwork-routes-storage-backend-api/src/main.rs");
    let source = std::fs::read_to_string(&main_path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", main_path.display()));

    for required in [
        "sdkwork_routes_storage_backend_api::build_router_with_database_config",
        "sdkwork_drive_config::DatabaseConfig",
        r#""127.0.0.1:18083""#,
        "serve admin storage api",
    ] {
        assert!(
            source.contains(required),
            "admin-storage binary entrypoint must include {required}"
        );
    }
}

#[test]
fn admin_storage_opendal_dependency_is_optional_and_feature_gated() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();

    let manifest_path = root.join("crates/sdkwork-routes-storage-backend-api/Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", manifest_path.display()));

    for required in [
        "default = []",
        r#"opendal-s3-plugin = ["dep:sdkwork-drive-storage-opendal"]"#,
        "sdkwork-drive-storage-opendal",
        "optional = true",
    ] {
        assert!(
            manifest.contains(required),
            "admin-storage manifest must keep OpenDAL feature gated by {required}"
        );
    }
}

#[test]
fn admin_storage_iam_runtime_boundary_is_documented() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();

    let read = |relative_path: &str| {
        let path = root.join(relative_path);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()))
            .replace("\r\n", "\n")
    };

    let readme = read("README.md");
    for required in [
        "Protected app, backend, and admin storage routes require:",
        "Open API share-link",
        "admin storage `/healthz` remain explicitly public",
        "must not send `x-sdkwork-tenant-id`",
    ] {
        assert!(
            readme.contains(required),
            "README IAM section must document admin-storage runtime auth boundary: {required}"
        );
    }

    let iam_standard = read("docs/architecture/tech/TECH-drive-iam-integration-standard.md");
    for required in [
        "App, backend, and admin-storage Drive routes must validate the same dual-token contract:",
        "Clients must not send AppContext projection headers",
        "Admin-storage `/healthz` is public; `/backend/v3/api/drive/storage/*` is protected.",
    ] {
        assert!(
            iam_standard.contains(required),
            "IAM standard must document admin-storage runtime auth boundary: {required}"
        );
    }

    let s3_architecture = read("docs/architecture/tech/TECH-storage-s3-architecture.md");
    for required in [
        "Admin-storage runtime routes under `/backend/v3/api/drive/storage/*` require the same dual-token contract as app and backend APIs.",
        "projection headers are forbidden",
        "`/healthz` is the only public admin-storage runtime route.",
    ] {
        assert!(
            s3_architecture.contains(required),
            "S3 architecture must document admin-storage runtime auth boundary: {required}"
        );
    }
}

#[test]
fn observability_event_dictionary_spec_exists() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    let spec_path =
        root.join("docs/architecture/tech/TECH-drive-observability-event-dictionary.md");
    let spec = std::fs::read_to_string(spec_path)
        .expect("observability event dictionary spec should exist");
    assert!(spec.contains("sdkwork.drive"));
    assert!(spec.contains("drive.audit_events.list"));
    assert!(spec.contains("drive.app.download_tokens.resolve"));
}

#[test]
fn drive_services_do_not_expose_application_local_iam_login_routes() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    let roots = [
        root.join("crates/sdkwork-drive-workspace-service/src"),
        root.join("crates/sdkwork-routes-drive-app-api/src"),
        root.join("crates/sdkwork-routes-drive-backend-api/src"),
        root.join("crates/sdkwork-routes-drive-open-api/src"),
        root.join("crates/sdkwork-routes-storage-backend-api/src"),
        root.join("crates/sdkwork-drive-http/src"),
        root.join("crates/sdkwork-drive-security/src"),
        root.join("crates/sdkwork-drive-observability/src"),
    ];
    let forbidden = [
        "/auth/login",
        "/auth/refresh",
        "user-center/session",
        "/app/v3/api/auth/login",
        "/app/v3/api/auth/oauth_sessions",
        "/app/v3/api/auth/oauth_authorization_urls",
        "/backend/v3/api/auth/login",
    ];

    let mut offenders = Vec::new();
    for scan_root in roots {
        collect_forbidden_auth_route_refs(&scan_root, &forbidden, &mut offenders);
    }

    assert_eq!(
        offenders,
        Vec::<String>::new(),
        "Drive must integrate IAM login instead of exposing application-local auth/session routes"
    );
}

fn collect_forbidden_auth_route_refs(
    root: &std::path::Path,
    forbidden: &[&str],
    offenders: &mut Vec<String>,
) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|value| value.to_str()) == Some("target") {
                continue;
            }
            collect_forbidden_auth_route_refs(&path, forbidden, offenders);
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("source file should be readable");
        for forbidden_ref in forbidden {
            if source.contains(forbidden_ref) {
                offenders.push(format!("{} contains {forbidden_ref}", path.display()));
            }
        }
    }
}
