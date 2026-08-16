use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

fn table_block<'a>(doc: &'a str, table: &str) -> &'a str {
    let needle = format!("  - table: {table}");
    let start = doc
        .find(&needle)
        .unwrap_or_else(|| panic!("schema registry should include table {table}"));
    let rest = &doc[start..];
    let end = rest.find("\n  - table: ").unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn schema_registry_includes_special_space_profiles() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/002-drive-special-spaces.yaml"),
    )
    .expect("special spaces schema file missing");
    assert!(doc.contains("dr_drive_space_knowledge_profile"));
    assert!(doc.contains("dr_drive_space_ai_generation_profile"));
    assert!(doc.contains("dr_drive_space_app_upload_profile"));
    assert!(doc.contains("dr_drive_space_rtc_profile"));
}

#[test]
fn schema_registry_includes_audit_indexes_for_filters() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/004-drive-security-audit.yaml"),
    )
    .expect("security audit schema file missing")
    .replace("\r\n", "\n");
    assert!(doc.contains("ix_dr_drive_audit_event_tenant_created"));
    assert!(doc.contains("ix_dr_drive_audit_event_resource"));
    assert!(doc.contains("ix_dr_drive_audit_event_action_created"));
    assert!(doc.contains("ix_dr_drive_audit_event_request_created"));
    assert!(doc.contains("ix_dr_drive_audit_event_trace_created"));
}

#[test]
fn schema_registry_matches_runtime_audit_event_shape() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/004-drive-security-audit.yaml"),
    )
    .expect("security audit schema file missing")
    .replace("\r\n", "\n");

    for required in [
        "- name: tenant_id\n        type: varchar(64)",
        "- name: resource_type\n        type: varchar(64)",
        "- name: resource_id\n        type: varchar(128)",
        "- name: created_at\n        type: timestamptz",
    ] {
        assert!(
            doc.contains(required),
            "audit registry should include runtime shape {required}"
        );
    }

    for stale in [
        "- name: uuid",
        "- name: tenant_id\n        type: bigint",
        "- name: resource_id\n        type: bigint",
        "- name: created_at\n        type: timestamp\n",
    ] {
        assert!(
            !doc.contains(stale),
            "audit registry should not keep stale shape {stale}"
        );
    }
}

#[test]
fn schema_registry_documents_sandbox_security_and_provider_boundaries() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/004-drive-security-audit.yaml"),
    )
    .expect("security audit schema file missing")
    .replace("\r\n", "\n");
    let volume = table_block(&doc, "dr_drive_sandbox_volume");
    for required in [
        "enum: [local_filesystem]",
        "ix_dr_drive_sandbox_volume_tenant_organization_status",
        "protected backend-admin value",
        "access still requires an explicit grant",
    ] {
        assert!(
            volume.contains(required),
            "sandbox volume registry should document {required}"
        );
    }
    assert!(!volume.contains("enum: [local_filesystem, s3"));

    let operation = table_block(&doc, "dr_drive_sandbox_mutation_operation");
    for forbidden in [
        "provider_root_ref",
        "physical_path",
        "absolute_path",
        "idempotency_key\n",
    ] {
        assert!(
            !operation.contains(forbidden),
            "sandbox mutation registry must not persist {forbidden}"
        );
    }
    assert!(operation.contains("idempotency_key_hash"));
}

#[test]
fn schema_registry_documents_runtime_dictionary_constraints() {
    let doc = [
        "docs/schema-registry/tables/001-drive-core.yaml",
        "docs/schema-registry/tables/002-drive-special-spaces.yaml",
        "docs/schema-registry/tables/003-drive-storage.yaml",
        "docs/schema-registry/tables/004-drive-security-audit.yaml",
    ]
    .into_iter()
    .map(|relative_path| {
        std::fs::read_to_string(workspace_root().join(relative_path))
            .unwrap_or_else(|error| panic!("{relative_path} schema file missing: {error}"))
    })
    .collect::<Vec<_>>()
    .join("\n");

    for required in [
        "enum: [personal, team, knowledge_base, ai_generated, git_repository, deployment, app_upload, im, rtc, notary, website]",
        "enum: [user, group, domain, app]",
        "enum: [reader, commenter, writer, owner]",
        "enum: [reader, commenter, writer]",
        "enum: [created, uploading, completing, completed, aborted, expired]",
        "enum: [active, deleted]",
        "enum: [active, stopped, expired]",
        "enum: [object_sweep, upload_session_sweep, expired_upload_content_sweep, abandoned_upload_task_sweep]",
        "enum: [completed, failed]",
        "min: 0",
        "min: 1",
    ] {
        assert!(
            doc.contains(required),
            "schema registry should document runtime dictionary constraint {required}"
        );
    }

    assert!(
        !doc.contains("enum: [succeeded, failed]"),
        "maintenance job status dictionary must match runtime values"
    );
}

#[test]
fn schema_registry_documents_entity_version_minimums() {
    for relative_path in [
        "docs/schema-registry/tables/001-drive-core.yaml",
        "docs/schema-registry/tables/003-drive-storage.yaml",
        "docs/schema-registry/tables/004-drive-security-audit.yaml",
    ] {
        let doc = std::fs::read_to_string(workspace_root().join(relative_path))
            .unwrap_or_else(|error| panic!("{relative_path} schema file missing: {error}"));
        let lines = doc.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            if line.trim() != "- name: version" {
                continue;
            }
            let next_field_index = lines
                .iter()
                .enumerate()
                .skip(index + 1)
                .find_map(|(candidate_index, candidate)| {
                    (candidate.starts_with("      - name: ")).then_some(candidate_index)
                })
                .unwrap_or(lines.len());
            let block = lines[index..next_field_index].join("\n");
            assert!(
                block.contains("\"min: 1\""),
                "{relative_path} version field at line {} should document min: 1",
                index + 1
            );
        }
    }
}

#[test]
fn schema_registry_includes_drive_acl_share_and_change_log_tables() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/004-drive-security-audit.yaml"),
    )
    .expect("security audit schema file missing");

    for required in [
        "dr_drive_node_permission",
        "node_id",
        "subject_id",
        "inherited",
        "ix_dr_drive_node_permission_resource",
        "ix_dr_drive_node_permission_subject",
        "dr_drive_node_share_link",
        "token_hash",
        "expires_at_epoch_ms",
        "download_limit",
        "download_count",
        "ux_dr_drive_node_share_link_token_hash",
        "ix_dr_drive_node_share_link_resource",
        "dr_drive_node_favorite",
        "subject_type",
        "subject_id",
        "ux_dr_drive_node_favorite_subject_node",
        "ix_dr_drive_node_favorite_subject",
        "dr_drive_node_property",
        "property_key",
        "property_value",
        "visibility",
        "ux_dr_drive_node_property_key",
        "ix_dr_drive_node_property_node",
        "dr_drive_label",
        "label_key",
        "display_name",
        "color",
        "ux_dr_drive_label_key",
        "ix_dr_drive_label_tenant_status",
        "dr_drive_node_label",
        "label_id",
        "ux_dr_drive_node_label_node_label",
        "ix_dr_drive_node_label_node",
        "ix_dr_drive_node_label_label",
        "dr_drive_watch_channel",
        "resource_type",
        "resource_id",
        "channel_type",
        "address",
        "token_hash",
        "expiration_epoch_ms",
        "ix_dr_drive_watch_channel_tenant_status",
        "ix_dr_drive_watch_channel_resource",
        "ix_dr_drive_watch_channel_node",
        "ix_dr_drive_watch_channel_expires",
        "dr_drive_node_comment",
        "anchor",
        "resolved",
        "ix_dr_drive_node_comment_node",
        "ix_dr_drive_node_comment_resolved",
        "dr_drive_node_comment_reply",
        "comment_id",
        "ix_dr_drive_node_comment_reply_comment",
        "ix_dr_drive_node_comment_reply_node",
        "dr_drive_change_cursor",
        "last_sequence_no",
        "ux_dr_drive_change_cursor_scope",
        "dr_drive_change_log",
        "sequence_no",
        "event_type",
        "actor_id",
        "ux_dr_drive_change_log_space_sequence",
        "ix_dr_drive_change_log_tenant_space_created",
        "dr_drive_domain_outbox",
        "payload_json",
        "delivery_status",
        "attempt_count",
        "ix_dr_drive_domain_outbox_pending",
    ] {
        assert!(
            doc.contains(required),
            "schema registry should include {required}"
        );
    }
}

#[test]
fn schema_registry_includes_core_node_shortcut_metadata() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/001-drive-core.yaml"),
    )
    .expect("core schema file missing");

    for required in [
        "dr_drive_node",
        "space_type",
        "ix_dr_drive_node_space_type_parent",
        "shortcut_target_node_id",
        "ix_dr_drive_node_shortcut_target",
    ] {
        assert!(
            doc.contains(required),
            "schema registry should include {required}"
        );
    }
}

#[test]
fn schema_registry_documents_user_owned_git_repository_spaces() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/001-drive-core.yaml"),
    )
    .expect("core schema file missing");

    for required in [
        "ck_dr_drive_space_git_repository_owner_user",
        "space_type != 'git_repository' OR owner_subject_type = 'user'",
        "Git repository spaces are always user-owned",
        "space_type != 'rtc' OR owner_subject_type = 'user'",
        "RTC recording spaces are always user-owned",
    ] {
        assert!(
            doc.contains(required),
            "schema registry should document git repository space ownership constraint {required}"
        );
    }
}

#[test]
fn schema_registry_includes_storage_provider_kind_dictionary() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/003-drive-storage.yaml"),
    )
    .expect("storage schema file missing");
    assert!(doc.contains("provider_kind"));
    assert!(doc.contains("local_filesystem"));
    assert!(doc.contains("s3_compatible"));
    assert!(doc.contains("google_cloud_storage"));
    assert!(doc.contains("aliyun_oss"));
    assert!(doc.contains("tencent_cos"));
    assert!(doc.contains("huawei_obs"));
    assert!(doc.contains("volcengine_tos"));
    assert!(doc.contains("custom:[a-z0-9_-]{2,32}"));
    assert!(
        !doc.contains("azure_blob"),
        "schema registry must not expose Azure Blob until a real adapter module exists"
    );
}

#[test]
fn schema_registry_matches_storage_object_and_upload_session_columns() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/003-drive-storage.yaml"),
    )
    .expect("storage schema file missing");

    for required in [
        "dr_drive_storage_object",
        "node_id",
        "version_no",
        "content_type",
        "content_length",
        "checksum_sha256_hex",
        "lifecycle_status",
        "ux_dr_drive_storage_object_node_version",
        "ux_dr_drive_storage_object_active_locator",
        "ix_dr_drive_storage_object_node_latest",
        "dr_drive_upload_session",
        "node_id",
        "bucket",
        "expires_at_epoch_ms",
        "created_by",
        "updated_by",
        "ux_dr_drive_upload_session_idempotency",
        "ix_dr_drive_upload_session_expires",
    ] {
        assert!(
            doc.contains(required),
            "schema registry should include {required}"
        );
    }

    let storage_object = table_block(&doc, "dr_drive_storage_object");
    let upload_session = table_block(&doc, "dr_drive_upload_session");
    for (block, stale) in [
        (storage_object, "- name: size_bytes"),
        (upload_session, "- name: expires_at\n"),
    ] {
        assert!(
            !block.contains(stale),
            "schema registry should not keep stale field {stale}"
        );
    }
}

#[test]
fn schema_registry_includes_download_package_metadata() {
    let doc = std::fs::read_to_string(
        workspace_root().join("docs/schema-registry/tables/003-drive-storage.yaml"),
    )
    .expect("storage schema file missing");

    for required in [
        "dr_drive_download_package",
        "package_name",
        "storage_provider_id",
        "archive_object_key",
        "file_count",
        "total_bytes",
        "archive_size_bytes",
        "requested_node_ids_json",
        "item_manifest_json",
        "expires_at_epoch_ms",
        "ix_dr_drive_download_package_tenant_state_created",
        "ix_dr_drive_download_package_expires",
    ] {
        assert!(
            doc.contains(required),
            "schema registry should include {required}"
        );
    }
}
