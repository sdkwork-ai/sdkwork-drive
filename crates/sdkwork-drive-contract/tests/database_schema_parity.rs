use std::collections::BTreeSet;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

fn read_workspace_file(relative: &str) -> String {
    std::fs::read_to_string(workspace_root().join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
        .replace("\r\n", "\n")
}

fn create_table_names(sql: &str) -> BTreeSet<String> {
    sql.lines()
        .filter_map(|line| {
            let normalized = line.trim().to_ascii_lowercase();
            normalized
                .strip_prefix("create table if not exists ")
                .and_then(|suffix| suffix.split_whitespace().next())
                .map(|name| name.trim_matches('"').to_string())
        })
        .collect()
}

fn create_index_names(sql: &str) -> BTreeSet<String> {
    sql.lines()
        .filter_map(|line| {
            let normalized = line.trim().to_ascii_lowercase();
            normalized
                .strip_prefix("create unique index if not exists ")
                .or_else(|| normalized.strip_prefix("create index if not exists "))
                .and_then(|suffix| suffix.split_whitespace().next())
                .map(|name| name.trim_matches('"').to_string())
        })
        .collect()
}

#[test]
fn postgres_schema_avoids_nul_byte_regex_fragments() {
    let postgres = read_workspace_file(
        "crates/sdkwork-drive-workspace-service/src/infrastructure/sql/postgres_core.sql",
    );

    assert!(
        !postgres.contains("chr(0)"),
        "PostgreSQL text values cannot contain NUL bytes; DDL must not construct NUL regex fragments"
    );
}

#[test]
fn postgres_schema_identifiers_fit_postgres_limit() {
    let postgres = read_workspace_file(
        "crates/sdkwork-drive-workspace-service/src/infrastructure/sql/postgres_core.sql",
    );
    let mut identifiers = create_table_names(&postgres);
    identifiers.extend(create_index_names(&postgres));
    for line in postgres.lines() {
        let normalized = line.trim();
        if let Some(suffix) = normalized.strip_prefix("CONSTRAINT ") {
            if let Some(identifier) = suffix.split_whitespace().next() {
                identifiers.insert(identifier.trim_matches('"').to_string());
            }
        }
    }

    for identifier in identifiers {
        assert!(
            identifier.len() <= 63,
            "PostgreSQL identifier exceeds the 63-byte limit and would be truncated: {identifier}"
        );
    }
}

#[test]
fn schema_registry_mentions_every_installed_table_and_index() {
    let postgres = read_workspace_file(
        "crates/sdkwork-drive-workspace-service/src/infrastructure/sql/postgres_core.sql",
    );
    let registry = [
        "docs/schema-registry/tables/001-drive-core.yaml",
        "docs/schema-registry/tables/002-drive-special-spaces.yaml",
        "docs/schema-registry/tables/003-drive-storage.yaml",
        "docs/schema-registry/tables/004-drive-security-audit.yaml",
        "docs/schema-registry/tables/005-drive-website-publishing.yaml",
    ]
    .into_iter()
    .map(read_workspace_file)
    .collect::<Vec<_>>()
    .join("\n");

    for table in create_table_names(&postgres) {
        assert!(
            registry.contains(&table),
            "schema registry must mention installed table {table}"
        );
    }
    for index in create_index_names(&postgres) {
        assert!(
            registry.contains(&index),
            "schema registry must mention installed index {index}"
        );
    }
}

#[test]
fn core_registry_matches_runtime_dr_drive_space_and_node_columns() {
    let core = read_workspace_file("docs/schema-registry/tables/001-drive-core.yaml");
    for required in [
        "- name: id\n        type: varchar(64)",
        "- name: owner_subject_type",
        "- name: owner_subject_id",
        "- name: display_name",
        "- name: parent_node_id",
        "- name: node_name",
        "- name: content_state",
        "- name: lifecycle_status",
        "- name: version",
        "- name: created_by",
        "- name: updated_by",
        "ux_dr_drive_node_root_name_live",
        "ux_dr_drive_node_child_name_live",
        "ix_dr_drive_node_space_parent",
    ] {
        assert!(
            core.contains(required),
            "core schema registry should include runtime column or index {required}"
        );
    }

    for stale in [
        "- name: uuid",
        "- name: tenant_id\n        type: bigint",
        "- name: normalized_name",
        "- name: status",
    ] {
        assert!(
            !core.contains(stale),
            "core schema registry should not keep stale shape {stale}"
        );
    }
}

#[test]
fn governed_database_migrations_exist_for_postgres() {
    // Ground this in the files that actually ship rather than a frozen list:
    // a hand-maintained list rots silently the moment a migration is added or
    // renamed, and then fails for the wrong reason. Assert the invariant the
    // list was standing in for — every numbered migration declares its engine
    // header and, except the initial baseline, ships its down-migration.
    let migrations_dir = workspace_root().join("database/migrations/postgres");
    let mut seen = BTreeSet::new();
    for entry in std::fs::read_dir(&migrations_dir)
        .unwrap_or_else(|error| panic!("failed to list {}: {error}", migrations_dir.display()))
    {
        let path = entry.expect("directory entry").path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".up.sql") else {
            continue;
        };
        seen.insert(stem.to_string());

        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        assert!(
            source.contains("-- sdkwork:migration") && source.contains("-- engine: postgres"),
            "{name} must carry the governed migration header for postgres"
        );

        // The first migration predates the up/down pairing convention.
        if stem.starts_with("0001_") {
            continue;
        }
        let down = migrations_dir.join(format!("{stem}.down.sql"));
        assert!(
            down.is_file(),
            "{stem} must ship a matching .down.sql migration"
        );
    }

    assert!(
        !seen.is_empty(),
        "database/migrations/postgres must contain at least one governed migration"
    );

    let manifest = read_workspace_file("database/database.manifest.json");
    assert!(
        manifest.contains("\"postgres\"") && !manifest.contains("\"sqlite\""),
        "database manifest must declare PostgreSQL as its only engine"
    );
}
