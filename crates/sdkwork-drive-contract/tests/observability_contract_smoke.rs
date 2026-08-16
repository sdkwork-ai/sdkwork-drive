use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root
}

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
fn observability_event_names_are_stable_across_code_and_spec() {
    let root = workspace_root();
    let observability_lib =
        std::fs::read_to_string(root.join("crates/sdkwork-drive-observability/src/lib.rs"))
            .expect("observability lib should exist");
    let backend_api =
        read_rust_source_tree(root.join("crates/sdkwork-routes-drive-backend-api/src"));
    let app_api = read_rust_source_tree(root.join("crates/sdkwork-routes-drive-app-api/src"));
    let spec = std::fs::read_to_string(
        root.join("docs/architecture/tech/TECH-drive-observability-event-dictionary.md"),
    )
    .expect("observability event dictionary spec should exist");

    let events = [
        "drive.audit_events.list",
        "drive.maintenance.jobs.list",
        "drive.maintenance.object_sweep",
        "drive.maintenance.upload_session_sweep",
        "drive.app.spaces.list",
        "drive.app.spaces.create",
        "drive.app.spaces.get",
        "drive.app.spaces.update",
        "drive.app.spaces.delete",
        "drive.app.upload_sessions.create",
        "drive.app.download_urls.create",
        "drive.app.download_tokens.resolve",
    ];
    for event_name in events {
        assert!(
            observability_lib.contains(event_name),
            "observability crate missing event constant value: {event_name}"
        );
        assert!(
            spec.contains(event_name),
            "observability event dictionary spec missing event: {event_name}"
        );
    }

    let error_kinds = [
        "validation",
        "conflict",
        "not_found",
        "permission_denied",
        "internal",
    ];
    for error_kind in error_kinds {
        assert!(
            observability_lib.contains(error_kind),
            "observability crate missing error_kind constant value: {error_kind}"
        );
        assert!(
            spec.contains(error_kind),
            "observability event dictionary spec missing error_kind: {error_kind}"
        );
    }

    for expected_reference in [
        "events::BACKEND_AUDIT_EVENTS_LIST",
        "events::BACKEND_MAINTENANCE_JOBS_LIST",
        "events::BACKEND_MAINTENANCE_OBJECT_SWEEP",
        "events::BACKEND_MAINTENANCE_UPLOAD_SESSION_SWEEP",
    ] {
        assert!(
            backend_api.contains(expected_reference),
            "backend api missing observability event reference: {expected_reference}"
        );
    }
    assert!(
        backend_api.contains("error_kinds::"),
        "backend api must reference observability error_kinds constants"
    );

    for expected_reference in [
        "events::APP_SPACES_LIST",
        "events::APP_SPACES_CREATE",
        "events::APP_SPACES_GET",
        "events::APP_SPACES_UPDATE",
        "events::APP_SPACES_DELETE",
        "events::APP_UPLOAD_SESSIONS_CREATE",
        "events::APP_DOWNLOAD_URLS_CREATE",
        "events::APP_DOWNLOAD_TOKENS_RESOLVE",
    ] {
        assert!(
            app_api.contains(expected_reference),
            "app api missing observability event reference: {expected_reference}"
        );
    }
    assert!(
        app_api.contains("error_kinds::"),
        "app api must reference observability error_kinds constants"
    );
}
