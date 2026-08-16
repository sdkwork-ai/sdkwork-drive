use std::path::PathBuf;

mod common;
use common::run_node_command_in;

fn workspace_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

#[test]
fn package_scripts_use_the_shared_lifecycle_without_sqlite_server_aliases() {
    let root = workspace_root();
    let package_json_path = root.join("package.json");
    let package_json =
        std::fs::read_to_string(&package_json_path).expect("root package.json should exist");
    let manifest: serde_json::Value =
        serde_json::from_str(&package_json).expect("package.json should be valid json");
    let scripts = manifest
        .get("scripts")
        .and_then(serde_json::Value::as_object)
        .expect("package.json scripts should exist");

    let dev_script = scripts
        .get("dev")
        .and_then(serde_json::Value::as_str)
        .expect("pnpm dev script should exist");
    let dev_standalone_script = scripts
        .get("dev:standalone")
        .and_then(serde_json::Value::as_str)
        .expect("pnpm dev:standalone script should exist");
    let dev_desktop_script = scripts
        .get("dev:desktop")
        .and_then(serde_json::Value::as_str)
        .expect("pnpm dev:desktop script should exist");
    let dev_cloud_script = scripts
        .get("dev:cloud")
        .and_then(serde_json::Value::as_str)
        .expect("pnpm dev:cloud script should exist");

    let dispatcher_path = root.join("scripts/sdkwork-command.mjs");
    let dispatcher = std::fs::read_to_string(&dispatcher_path)
        .expect("scripts/sdkwork-command.mjs should exist");

    assert!(
        dev_script == "pnpm dev:standalone",
        "pnpm dev must delegate exactly to dev:standalone, got: {dev_script}"
    );
    assert!(
        dev_standalone_script.contains("sdkwork-app dev")
            && dev_standalone_script.contains("--deployment-profile standalone"),
        "pnpm dev:standalone must use the shared sdkwork-app lifecycle, got: {dev_standalone_script}"
    );

    assert!(scripts.get("dev:browser:sqlite").is_none());
    assert!(scripts.get("dev:desktop:sqlite").is_none());

    assert!(
        dev_desktop_script == "pnpm dev:desktop:postgres:standalone",
        "pnpm dev:desktop must delegate to the canonical PostgreSQL standalone command, got: {dev_desktop_script}"
    );
    let dev_desktop_standalone = scripts
        .get("dev:desktop:postgres:standalone")
        .and_then(serde_json::Value::as_str)
        .expect("pnpm dev:desktop:postgres:standalone script should exist");
    assert!(
        dev_desktop_standalone.contains("sdkwork-app dev")
            && dev_desktop_standalone.contains("--runtime-target desktop")
            && dev_desktop_standalone.contains("--deployment-profile standalone"),
        "canonical desktop standalone command must use sdkwork-app, got: {dev_desktop_standalone}"
    );
    let retired_desktop_script = [
        "dev",
        "desktop",
        "postgres",
        "unified-process",
        "standalone",
    ]
    .join(":");
    assert!(
        !scripts.contains_key(&retired_desktop_script),
        "retired desktop dev script must not remain"
    );
    assert!(
        dev_cloud_script.contains("sdkwork-app dev")
            && dev_cloud_script.contains("--deployment-profile cloud")
            && !dev_cloud_script.contains("--database"),
        "pnpm dev:cloud must use sdkwork-app without a local database axis, got: {dev_cloud_script}"
    );
    assert!(
        !scripts.contains_key("dev:browser:postgres:cloud"),
        "cloud development must not retain a database-axis script"
    );
    assert!(
        dispatcher.contains("Object.hasOwn(flags, \"service-layout\")")
            && dispatcher.contains("--service-layout is internal topology detail")
            && !dispatcher.contains("\"--service-layout\","),
        "sdkwork-command.mjs must reject public --service-layout without forwarding it to drive-dev.mjs"
    );

    let package_script = scripts
        .get("gateway:package:standalone")
        .and_then(serde_json::Value::as_str)
        .expect("pnpm gateway:package:standalone script should exist");
    assert!(
        package_script.contains("sdkwork-command.mjs"),
        "pnpm gateway:package:standalone must delegate through sdkwork-command.mjs, got: {package_script}"
    );
    assert!(
        dispatcher.contains("gateway-standalone-pack.mjs"),
        "sdkwork-command.mjs must dispatch gateway:package:standalone to gateway-standalone-pack.mjs"
    );

    assert!(
        scripts.get("gateway:package:cloud").is_none(),
        "application repositories must not package the platform API gateway"
    );

    let drive_build = scripts
        .get("build")
        .and_then(serde_json::Value::as_str)
        .expect("pnpm build script should exist");
    assert_eq!(
        drive_build, "pnpm exec sdkwork-app build",
        "pnpm build must delegate to the manifest-driven sdkwork-app lifecycle"
    );
    let private_build = scripts
        .get("_sdkwork:build")
        .and_then(serde_json::Value::as_str)
        .expect("private build implementation should exist");
    assert!(
        private_build.contains("sdkwork-command.mjs")
            && dispatcher.contains("drive-build.mjs")
            && dispatcher.contains("'--deployment-profile', 'cloud'"),
        "the private build hook must dispatch to drive-build.mjs with the cloud default"
    );

    let build_standalone = scripts
        .get("build:standalone")
        .and_then(serde_json::Value::as_str)
        .expect("pnpm build:standalone script should exist");
    assert!(
        build_standalone.contains("sdkwork-app build"),
        "pnpm build:standalone must use sdkwork-app, got: {build_standalone}"
    );
    assert!(
        build_standalone.contains("standalone"),
        "pnpm build:standalone must target the standalone deployment profile, got: {build_standalone}"
    );
    assert!(
        dispatcher.contains("'--deployment-profile', 'standalone'"),
        "sdkwork-command.mjs must dispatch standalone build profiles"
    );

    let build_debug = scripts
        .get("build:debug")
        .and_then(serde_json::Value::as_str)
        .expect("pnpm build:debug script should exist");
    assert!(
        build_debug.contains("drive-build.mjs") && build_debug.contains("--debug"),
        "pnpm build:debug must invoke drive-build.mjs in debug mode, got: {build_debug}"
    );
}

#[test]
fn postgres_and_toml_examples_use_standard_drive_config_keys() {
    let root = workspace_root();
    let postgres_example = std::fs::read_to_string(root.join(".env.postgres.example"))
        .expect(".env.postgres.example should exist");

    for required in [
        "SDKWORK_DATABASE_ENGINE=postgresql",
        "SDKWORK_DATABASE_HOST=127.0.0.1",
        "SDKWORK_DATABASE_PORT=5432",
        "SDKWORK_DATABASE_NAME=sdkwork_ai_dev",
        "SDKWORK_DATABASE_SCHEMA=sdkwork_ai_dev",
        "SDKWORK_DATABASE_USERNAME=sdkwork_ai_dev",
        "SDKWORK_DATABASE_PASSWORD=sdkworkdev123",
        "SDKWORK_DATABASE_SSL_MODE=disable",
        "SDKWORK_DATABASE_MAX_CONNECTIONS=10",
        "SDKWORK_DATABASE_ADMIN_HOST=127.0.0.1",
        "SDKWORK_DATABASE_ADMIN_SSL_MODE=disable",
    ] {
        assert!(
            postgres_example.contains(required),
            ".env.postgres.example must include standard key {required}"
        );
    }
    for forbidden in ["SDKWORK_DATABASE_PROVIDER", "SDKWORK_DATABASE_SSLMODE"] {
        assert!(
            !postgres_example.contains(forbidden),
            ".env.postgres.example must not include legacy key {forbidden}"
        );
    }

    let toml_example = std::fs::read_to_string(root.join("etc/drive.database.example.toml"))
        .expect("etc/drive.database.example.toml should exist");
    for required in [
        "SDKWORK_DRIVE_CONFIG_FILE=./etc/drive.database.example.toml",
        "[database]",
        "engine = \"postgresql\"",
        "ssl_mode = \"require\"",
        "[database_sqlite_example]",
    ] {
        assert!(
            toml_example.contains(required),
            "drive database TOML example must include {required}"
        );
    }
}

#[test]
fn drive_launch_plan_reports_database_engine_without_leaking_secrets() {
    let root = workspace_root();
    let output = run_node_command_in(
        &root,
        [
            root.join("scripts/run-drive-standalone-gateway.mjs"),
            PathBuf::from("plan"),
            PathBuf::from("--dev-env-file"),
            root.join(".env.postgres.example"),
        ],
    )
    .expect("drive product runner should start");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("databaseEngine=postgresql"),
        "plan should report PostgreSQL engine, stdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("drive_pass") && !stdout.contains("sdkwork_ai_dev:"),
        "plan output must not leak credentials, stdout:\n{stdout}"
    );

    for required_service in [
        "drive standalone gateway: cargo",
        "sdkwork-api-drive-standalone-gateway",
    ] {
        assert!(
            stdout.contains(required_service),
            "launch plan must include {required_service}, stdout:\n{stdout}"
        );
    }
}

#[test]
fn drive_launch_plan_url_encodes_structured_postgres_fields() {
    let root = workspace_root();
    let temp_dir = root.join("target").join("database-tooling-smoke");
    std::fs::create_dir_all(&temp_dir).expect("temp dir should be created");
    let env_file = temp_dir.join("postgres-special.env");
    std::fs::write(
        &env_file,
        [
            "SDKWORK_DATABASE_ENGINE=postgresql",
            "SDKWORK_DATABASE_HOST=db.internal",
            "SDKWORK_DATABASE_PORT=5432",
            "SDKWORK_DATABASE_NAME=sdkwork drive/dev",
            "SDKWORK_DATABASE_SCHEMA=sdkwork_ai_dev",
            "SDKWORK_DATABASE_USERNAME=sdkworkprod@2026++",
            "SDKWORK_DATABASE_PASSWORD=pa@ss+word/with space",
            "SDKWORK_DATABASE_SSL_MODE=require",
            "SDKWORK_DATABASE_MAX_CONNECTIONS=32",
        ]
        .join("\n"),
    )
    .expect("env file should be written");

    let output = run_node_command_in(
        &root,
        [
            root.join("scripts/run-drive-standalone-gateway.mjs"),
            PathBuf::from("plan"),
            PathBuf::from("--dev-env-file"),
            env_file.clone(),
        ],
    )
    .expect("drive product runner should start");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("databaseEngine=postgresql"),
        "plan should report PostgreSQL engine, stdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("pa@ss+word")
            && !stdout.contains("sdkworkprod@2026++")
            && !stdout.contains("db.internal"),
        "plan output must not leak structured database fields, stdout:\n{stdout}"
    );
}

#[test]
fn drive_launch_plan_rejects_legacy_database_aliases() {
    let root = workspace_root();
    let temp_dir = root.join("target").join("database-tooling-smoke");
    std::fs::create_dir_all(&temp_dir).expect("temp dir should be created");
    let env_file = temp_dir.join("postgres-legacy.env");
    std::fs::write(
        &env_file,
        [
            "SDKWORK_DATABASE_PROVIDER=postgresql",
            "SDKWORK_DATABASE_HOST=127.0.0.1",
            "SDKWORK_DATABASE_PORT=5432",
            "SDKWORK_DATABASE_NAME=sdkwork_ai_dev",
            "SDKWORK_DATABASE_USERNAME=sdkwork_ai_dev",
            "SDKWORK_DATABASE_PASSWORD=sdkworkdev123",
            "SDKWORK_DATABASE_SSLMODE=disable",
        ]
        .join("\n"),
    )
    .expect("env file should be written");

    let output = run_node_command_in(
        &root,
        [
            root.join("scripts/run-drive-standalone-gateway.mjs"),
            PathBuf::from("plan"),
            PathBuf::from("--dev-env-file"),
            env_file.clone(),
        ],
    )
    .expect("drive product runner should start");

    assert!(
        !output.status.success(),
        "legacy aliases should fail closed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("SDKWORK_DATABASE_PROVIDER") && stderr.contains("SDKWORK_DATABASE_SSLMODE"),
        "error should name rejected legacy aliases, stderr:\n{stderr}"
    );
}

#[test]
fn drive_launch_plan_rejects_explicit_sqlite_database_url() {
    let root = workspace_root();
    let output = run_node_command_in(
        &root,
        [
            root.join("scripts/run-drive-standalone-gateway.mjs"),
            PathBuf::from("plan"),
            PathBuf::from("--"),
            PathBuf::from("--database-url"),
            PathBuf::from("sqlite://target/dev/sdkwork-drive.sqlite"),
        ],
    )
    .expect("drive product runner should start");

    assert!(
        !output.status.success(),
        "SQLite must fail closed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("PostgreSQL") || stderr.contains("postgres"),
        "error should identify the PostgreSQL-only contract, stderr:\n{stderr}"
    );
}

#[test]
fn database_architecture_doc_records_runtime_boundary() {
    let root = workspace_root();
    let doc =
        std::fs::read_to_string(root.join("docs/architecture/tech/TECH-database-architecture.md"))
            .expect("database architecture doc should exist");

    for required in [
        "PostgreSQL is the only server runtime database",
        "pnpm dev",
        "SDKWORK_DRIVE_CONFIG_FILE=./etc/drive.database.example.toml",
        "SDKWORK_DATABASE_ENGINE=postgresql",
        "SDKWORK_DATABASE_SSL_MODE",
        "build_router_with_database_config",
        "sqlx::PgPool",
        "Runtime SQL must use PostgreSQL-compatible `$1`, `$2`, ... bind placeholders",
        "SQLite is rejected",
    ] {
        assert!(
            doc.contains(required),
            "database architecture doc should include {required}"
        );
    }
}
