use sdkwork_drive_security::{DriveAppContext, DRIVE_SANDBOXES_ADMIN_PERMISSION};
use sdkwork_drive_workspace_service::application::sandbox_admin_service::{
    CreateSandboxAdminGrantCommand, CreateSandboxAdminVolumeCommand, DriveSandboxAdminService,
    InitialSandboxUserGrant, ListSandboxAdminGrantsCommand, ListSandboxAdminVolumesCommand,
    UpdateSandboxAdminGrantCommand, UpdateSandboxAdminVolumeCommand,
};
use sdkwork_drive_workspace_service::application::sandbox_service::DriveSandboxService;
use sdkwork_drive_workspace_service::infrastructure::sql::sandbox_admin_store::SqlSandboxAdminStore;
use sdkwork_drive_workspace_service::infrastructure::sql::sandbox_store::SqlSandboxStore;
use sdkwork_drive_workspace_service::DriveServiceError;

#[tokio::test]
async fn volume_creation_canonicalizes_root_and_creates_explicit_current_user_grant() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let root = tempfile::tempdir().expect("sandbox root should be created");
    let canonical_root = std::fs::canonicalize(root.path())
        .expect("sandbox root should canonicalize")
        .to_string_lossy()
        .to_string();
    let context = admin_context("tenant-a", "organization-a", "user-a");
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(pool.clone()));

    let created = service
        .create_volume(
            &context,
            CreateSandboxAdminVolumeCommand {
                display_name: " Server workspace ".to_string(),
                provider_kind: None,
                provider_root_ref: root.path().to_string_lossy().to_string(),
                default_access: None,
                initial_user_grant: None,
            },
        )
        .await
        .expect("sandbox volume should be created");

    assert_eq!(created.tenant_id, "tenant-a");
    assert_eq!(created.organization_id, "organization-a");
    assert_eq!(created.display_name, "Server workspace");
    assert_eq!(created.provider_kind, "local_filesystem");
    assert_eq!(created.provider_root_ref, canonical_root);
    assert_eq!(created.default_access, "full");
    assert_eq!(created.lifecycle_status, "active");
    assert_eq!(created.version, 1);

    let grants = service
        .list_grants(
            &context,
            ListSandboxAdminGrantsCommand {
                sandbox_id: created.id.clone(),
                page: None,
                page_size: None,
            },
        )
        .await
        .expect("default grant should be listed");
    assert_eq!(grants.total_items, 1);
    assert_eq!(grants.items[0].subject_type, "user");
    assert_eq!(grants.items[0].subject_id, "user-a");
    assert_eq!(grants.items[0].access_level, "full");

    let explorer_service = DriveSandboxService::new(SqlSandboxStore::new(pool.clone()));
    let accessible = explorer_service
        .list_accessible("tenant-a", "user", "user-a", 0, 20)
        .await
        .expect("explicitly granted volume should be accessible");
    assert_eq!(accessible.len(), 1);
    assert_eq!(accessible[0].id, created.id);

    let audit_rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT tenant_id, action, resource_type
         FROM dr_drive_audit_event
         ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("sandbox audit events should be readable");
    assert_eq!(
        audit_rows,
        vec![
            (
                "tenant-a".to_string(),
                "drive.sandbox_volume.created".to_string(),
                "sandbox_volume".to_string(),
            ),
            (
                "tenant-a".to_string(),
                "drive.sandbox_grant.created".to_string(),
                "sandbox_grant".to_string(),
            ),
        ]
    );
    let audit_wire = serde_json::to_string(&audit_rows).expect("audit rows should serialize");
    assert!(
        !audit_wire.contains(&canonical_root),
        "physical provider roots must never enter audit facts"
    );
}

#[tokio::test]
async fn default_access_never_creates_implicit_tenant_access() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let root = tempfile::tempdir().expect("sandbox root should be created");
    let context = admin_context("tenant-a", "organization-a", "user-a");
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(pool.clone()));

    let created = service
        .create_volume(
            &context,
            CreateSandboxAdminVolumeCommand {
                display_name: "Unassigned root".to_string(),
                provider_kind: Some("local_filesystem".to_string()),
                provider_root_ref: root.path().to_string_lossy().to_string(),
                default_access: Some("full".to_string()),
                initial_user_grant: Some(InitialSandboxUserGrant {
                    enabled: false,
                    access_level: None,
                }),
            },
        )
        .await
        .expect("unassigned sandbox volume should be created");

    let grants = service
        .list_grants(
            &context,
            ListSandboxAdminGrantsCommand {
                sandbox_id: created.id.clone(),
                page: Some(1),
                page_size: Some(20),
            },
        )
        .await
        .expect("grant list should be readable");
    assert!(grants.items.is_empty());

    let explorer_service = DriveSandboxService::new(SqlSandboxStore::new(pool));
    let accessible = explorer_service
        .list_accessible("tenant-a", "user", "user-a", 0, 20)
        .await
        .expect("accessible sandbox list should be readable");
    assert!(
        accessible.is_empty(),
        "defaultAccess=full must not imply tenant-wide or user access"
    );

    let override_root = tempfile::tempdir().expect("override sandbox root should be created");
    let overridden = service
        .create_volume(
            &context,
            CreateSandboxAdminVolumeCommand {
                display_name: "Read-only assignment".to_string(),
                provider_kind: None,
                provider_root_ref: override_root.path().to_string_lossy().to_string(),
                default_access: None,
                initial_user_grant: Some(InitialSandboxUserGrant {
                    enabled: true,
                    access_level: Some("read_only".to_string()),
                }),
            },
        )
        .await
        .expect("initial current-user grant access should be overridable");
    let overridden_grants = service
        .list_grants(
            &context,
            ListSandboxAdminGrantsCommand {
                sandbox_id: overridden.id,
                page: None,
                page_size: None,
            },
        )
        .await
        .expect("overridden initial grant should be listed");
    assert_eq!(overridden_grants.items.len(), 1);
    assert_eq!(overridden_grants.items[0].access_level, "read_only");
}

#[tokio::test]
async fn admin_volume_list_pages_inside_the_verified_organization() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let context = admin_context("tenant-a", "organization-a", "user-a");
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(pool));
    let mut roots = Vec::new();
    for display_name in ["Gamma", "Alpha", "Beta"] {
        let root = tempfile::tempdir().expect("sandbox root should be created");
        service
            .create_volume(
                &context,
                CreateSandboxAdminVolumeCommand {
                    display_name: display_name.to_string(),
                    provider_kind: None,
                    provider_root_ref: root.path().to_string_lossy().to_string(),
                    default_access: None,
                    initial_user_grant: Some(InitialSandboxUserGrant {
                        enabled: false,
                        access_level: None,
                    }),
                },
            )
            .await
            .expect("sandbox volume should be created");
        roots.push(root);
    }

    let page = service
        .list_volumes(
            &context,
            ListSandboxAdminVolumesCommand {
                lifecycle_status: Some("active".to_string()),
                provider_kind: Some("local_filesystem".to_string()),
                page: Some(2),
                page_size: Some(1),
            },
        )
        .await
        .expect("sandbox volume page should be listed");
    assert_eq!(page.page, 2);
    assert_eq!(page.page_size, 1);
    assert_eq!(page.total_items, 3);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].display_name, "Beta");
    drop(roots);
}

#[tokio::test]
async fn lifecycle_and_grant_mutations_are_tenant_scoped_versioned_and_audited() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let root = tempfile::tempdir().expect("sandbox root should be created");
    let context = admin_context("tenant-a", "organization-a", "user-a");
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(pool.clone()));
    let created = service
        .create_volume(
            &context,
            CreateSandboxAdminVolumeCommand {
                display_name: "Lifecycle root".to_string(),
                provider_kind: None,
                provider_root_ref: root.path().to_string_lossy().to_string(),
                default_access: None,
                initial_user_grant: None,
            },
        )
        .await
        .expect("sandbox volume should be created");

    let updated = service
        .update_volume(
            &context,
            UpdateSandboxAdminVolumeCommand {
                sandbox_id: created.id.clone(),
                display_name: None,
                provider_root_ref: None,
                lifecycle_status: Some("read_only".to_string()),
                default_access: Some("read_only".to_string()),
                expected_version: 1,
            },
        )
        .await
        .expect("sandbox lifecycle should update");
    assert_eq!(updated.lifecycle_status, "read_only");
    assert_eq!(updated.default_access, "read_only");
    assert_eq!(updated.version, 2);

    let stale_update = service
        .update_volume(
            &context,
            UpdateSandboxAdminVolumeCommand {
                sandbox_id: created.id.clone(),
                display_name: Some("Stale".to_string()),
                provider_root_ref: None,
                lifecycle_status: None,
                default_access: None,
                expected_version: 1,
            },
        )
        .await
        .expect_err("stale sandbox versions must conflict");
    assert!(matches!(stale_update, DriveServiceError::Conflict(_)));

    let organization_grant = service
        .create_grant(
            &context,
            CreateSandboxAdminGrantCommand {
                sandbox_id: created.id.clone(),
                subject_type: "organization".to_string(),
                subject_id: "organization-b".to_string(),
                access_level: "full".to_string(),
            },
        )
        .await
        .expect("organization grants should be supported");
    let retrieved_grant = service
        .get_grant(&context, &created.id, &organization_grant.id)
        .await
        .expect("organization grant should be retrievable");
    assert_eq!(retrieved_grant, organization_grant);
    let organization_grant = service
        .update_grant(
            &context,
            UpdateSandboxAdminGrantCommand {
                sandbox_id: created.id.clone(),
                grant_id: organization_grant.id.clone(),
                access_level: "read_only".to_string(),
            },
        )
        .await
        .expect("grant access should update");
    assert_eq!(organization_grant.access_level, "read_only");
    service
        .delete_grant(&context, &created.id, &organization_grant.id)
        .await
        .expect("grant should delete");

    let cross_tenant = admin_context("tenant-b", "organization-b", "user-b");
    let cross_tenant_read = service
        .get_volume(&cross_tenant, &created.id)
        .await
        .expect_err("another tenant must not retrieve the volume");
    assert!(matches!(cross_tenant_read, DriveServiceError::NotFound(_)));

    let cross_organization = admin_context("tenant-a", "organization-b", "user-b");
    let cross_organization_list = service
        .list_volumes(
            &cross_organization,
            ListSandboxAdminVolumesCommand::default(),
        )
        .await
        .expect("organization-scoped list should be readable");
    assert!(cross_organization_list.items.is_empty());
    let cross_organization_read = service
        .get_volume(&cross_organization, &created.id)
        .await
        .expect_err("another organization must not retrieve the volume");
    assert!(matches!(
        cross_organization_read,
        DriveServiceError::NotFound(_)
    ));
    let cross_organization_grants = service
        .list_grants(
            &cross_organization,
            ListSandboxAdminGrantsCommand {
                sandbox_id: created.id.clone(),
                page: None,
                page_size: None,
            },
        )
        .await
        .expect_err("another organization must not list volume grants");
    assert!(matches!(
        cross_organization_grants,
        DriveServiceError::NotFound(_)
    ));

    service
        .delete_volume(&context, &created.id)
        .await
        .expect("sandbox volume should delete");
    let deleted = service
        .get_volume(&context, &created.id)
        .await
        .expect_err("deleted sandbox volume must not remain visible");
    assert!(matches!(deleted, DriveServiceError::NotFound(_)));

    let audit_count: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_audit_event WHERE tenant_id='tenant-a'")
            .fetch_one(&pool)
            .await
            .expect("audit count should be readable");
    assert_eq!(audit_count, 7, "each successful mutation must be audited");
}

#[tokio::test]
async fn service_rejects_unverified_scope_invalid_roots_and_unresolved_subject_types() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let root = tempfile::tempdir().expect("sandbox root should be created");
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(pool));

    let mut unauthorized = admin_context("tenant-a", "organization-a", "user-a");
    unauthorized.permission_scope.clear();
    let denied = service
        .list_volumes(&unauthorized, ListSandboxAdminVolumesCommand::default())
        .await
        .expect_err("missing sandbox admin scope must be rejected in the service");
    assert!(matches!(denied, DriveServiceError::PermissionDenied(_)));

    let mut personal_session = admin_context("tenant-a", "organization-a", "user-a");
    personal_session.organization_id = None;
    let denied = service
        .list_volumes(&personal_session, ListSandboxAdminVolumesCommand::default())
        .await
        .expect_err("personal backend sessions must be rejected in the service");
    assert!(matches!(denied, DriveServiceError::PermissionDenied(_)));

    let zero_organization = admin_context("tenant-a", "0", "user-a");
    let denied = service
        .list_volumes(
            &zero_organization,
            ListSandboxAdminVolumesCommand::default(),
        )
        .await
        .expect_err("zero organization backend sessions must be rejected");
    assert!(matches!(denied, DriveServiceError::PermissionDenied(_)));

    let mut service_actor = admin_context("tenant-a", "organization-a", "service-a");
    service_actor.actor_kind = "service".to_string();
    let denied = service
        .create_volume(
            &service_actor,
            CreateSandboxAdminVolumeCommand {
                display_name: "Service-created root".to_string(),
                provider_kind: None,
                provider_root_ref: root.path().to_string_lossy().to_string(),
                default_access: None,
                initial_user_grant: None,
            },
        )
        .await
        .expect_err("service actors must not receive a synthetic user grant");
    assert!(
        matches!(denied, DriveServiceError::Validation(message) if message.contains("must set enabled=false"))
    );

    let context = admin_context("tenant-a", "organization-a", "user-a");
    for unavailable_provider in ["s3", "opendal"] {
        let error = service
            .create_volume(
                &context,
                CreateSandboxAdminVolumeCommand {
                    display_name: format!("Unavailable {unavailable_provider}"),
                    provider_kind: Some(unavailable_provider.to_string()),
                    provider_root_ref: root.path().to_string_lossy().to_string(),
                    default_access: None,
                    initial_user_grant: None,
                },
            )
            .await
            .expect_err("providers without runtime adapters must be rejected");
        assert!(
            matches!(error, DriveServiceError::Validation(message) if message.contains("only local_filesystem"))
        );
    }
    let missing_root = root.path().join("missing-private-root");
    let missing_root_text = missing_root.to_string_lossy().to_string();
    let invalid = service
        .create_volume(
            &context,
            CreateSandboxAdminVolumeCommand {
                display_name: "Invalid".to_string(),
                provider_kind: None,
                provider_root_ref: missing_root_text.clone(),
                default_access: None,
                initial_user_grant: None,
            },
        )
        .await
        .expect_err("missing local roots must be rejected");
    let DriveServiceError::Validation(message) = invalid else {
        panic!("missing local root should be a validation error");
    };
    assert!(
        !message.contains(&missing_root_text),
        "root validation errors must not echo physical paths"
    );

    let file_root = tempfile::NamedTempFile::new().expect("sandbox file root should be created");
    let error = service
        .create_volume(
            &context,
            CreateSandboxAdminVolumeCommand {
                display_name: "File root".to_string(),
                provider_kind: None,
                provider_root_ref: file_root.path().to_string_lossy().to_string(),
                default_access: None,
                initial_user_grant: None,
            },
        )
        .await
        .expect_err("a sandbox root must be a directory");
    assert!(
        matches!(error, DriveServiceError::Validation(message) if message.contains("must be a directory"))
    );

    let valid = service
        .create_volume(
            &context,
            CreateSandboxAdminVolumeCommand {
                display_name: "Valid".to_string(),
                provider_kind: None,
                provider_root_ref: root.path().to_string_lossy().to_string(),
                default_access: None,
                initial_user_grant: None,
            },
        )
        .await
        .expect("valid local root should be accepted");
    for subject_type in ["workspace", "role"] {
        let error = service
            .create_grant(
                &context,
                CreateSandboxAdminGrantCommand {
                    sandbox_id: valid.id.clone(),
                    subject_type: subject_type.to_string(),
                    subject_id: format!("{subject_type}-1"),
                    access_level: "full".to_string(),
                },
            )
            .await
            .expect_err("unresolved membership subject must be rejected");
        assert!(
            matches!(error, DriveServiceError::Validation(message) if message.contains("authoritative membership resolver"))
        );
    }
}

#[tokio::test]
async fn audit_failure_rolls_back_volume_and_initial_grant_atomically() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    sqlx::query(
        "CREATE TRIGGER reject_sandbox_grant_audit
         BEFORE INSERT ON dr_drive_audit_event
         WHEN NEW.action = 'drive.sandbox_grant.created'
         BEGIN
           SELECT RAISE(ABORT, 'sandbox audit unavailable');
         END",
    )
    .execute(&pool)
    .await
    .expect("audit failure trigger should be installed");
    let root = tempfile::tempdir().expect("sandbox root should be created");
    let context = admin_context("tenant-a", "organization-a", "user-a");
    let service = DriveSandboxAdminService::new(SqlSandboxAdminStore::new(pool.clone()));

    let error = service
        .create_volume(
            &context,
            CreateSandboxAdminVolumeCommand {
                display_name: "Atomic sandbox".to_string(),
                provider_kind: None,
                provider_root_ref: root.path().to_string_lossy().to_string(),
                default_access: None,
                initial_user_grant: None,
            },
        )
        .await
        .expect_err("audit failure must fail the mutation");
    assert!(matches!(error, DriveServiceError::Internal(_)));

    let volume_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_sandbox_volume")
        .fetch_one(&pool)
        .await
        .expect("volume count should be readable");
    let grant_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_sandbox_grant")
        .fetch_one(&pool)
        .await
        .expect("grant count should be readable");
    let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_audit_event")
        .fetch_one(&pool)
        .await
        .expect("audit count should be readable");
    assert_eq!((volume_count, grant_count, audit_count), (0, 0, 0));
}

fn admin_context(tenant_id: &str, organization_id: &str, user_id: &str) -> DriveAppContext {
    DriveAppContext {
        tenant_id: tenant_id.to_string(),
        user_id: user_id.to_string(),
        organization_id: Some(organization_id.to_string()),
        session_id: Some("session-admin".to_string()),
        app_id: Some("appbase".to_string()),
        environment: Some("test".to_string()),
        deployment_mode: Some("saas".to_string()),
        auth_level: Some("mfa".to_string()),
        data_scope: Vec::new(),
        permission_scope: vec![DRIVE_SANDBOXES_ADMIN_PERMISSION.to_string()],
        actor_id: user_id.to_string(),
        actor_kind: "user".to_string(),
        device_id: None,
        request_id: "request-admin".to_string(),
        trace_id: "trace-admin".to_string(),
    }
}
