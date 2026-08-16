use sdkwork_drive_workspace_service::application::sandbox_service::DriveSandboxService;
use sdkwork_drive_workspace_service::infrastructure::sql::sandbox_store::SqlSandboxStore;
use sdkwork_drive_workspace_service::ports::sandbox_principal_resolver::EffectiveSandboxPrincipal;
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::PgPool;

#[tokio::test]
async fn sandbox_service_lists_only_explicit_grants_with_stable_pagination() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    insert_sandbox(&pool, "sandbox-alpha", "tenant-a", "Alpha", "active").await;
    insert_sandbox(&pool, "sandbox-bravo", "tenant-a", "Bravo", "read_only").await;
    insert_sandbox(
        &pool,
        "sandbox-default-but-ungranted",
        "tenant-a",
        "Default But Ungranted",
        "active",
    )
    .await;
    insert_sandbox(
        &pool,
        "sandbox-disabled",
        "tenant-a",
        "Disabled",
        "disabled",
    )
    .await;
    insert_sandbox(
        &pool,
        "sandbox-other-tenant",
        "tenant-b",
        "Other Tenant",
        "active",
    )
    .await;

    insert_grant(&pool, "grant-alpha", "sandbox-alpha", "tenant-user", "full").await;
    insert_grant(&pool, "grant-bravo", "sandbox-bravo", "tenant-user", "full").await;
    insert_grant(
        &pool,
        "grant-disabled",
        "sandbox-disabled",
        "tenant-user",
        "full",
    )
    .await;
    insert_grant(
        &pool,
        "grant-other-tenant",
        "sandbox-other-tenant",
        "tenant-user",
        "full",
    )
    .await;

    let service = DriveSandboxService::new(SqlSandboxStore::new(pool));

    let first_page = service
        .list_accessible("tenant-a", "user", "tenant-user", 0, 1)
        .await
        .expect("first sandbox page should be readable");
    assert_eq!(first_page.len(), 1);
    assert_eq!(first_page[0].id, "sandbox-alpha");

    let second_page = service
        .list_accessible("tenant-a", "user", "tenant-user", 1, 1)
        .await
        .expect("second sandbox page should be readable");
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page[0].id, "sandbox-bravo");
    assert_eq!(second_page[0].lifecycle_status, "read_only");

    let no_more_pages = service
        .list_accessible("tenant-a", "user", "tenant-user", 2, 1)
        .await
        .expect("empty final sandbox page should be readable");
    assert!(no_more_pages.is_empty());

    let other_tenant = service
        .list_accessible("tenant-b", "user", "tenant-user", 0, 20)
        .await
        .expect("other tenant sandbox page should be readable");
    assert_eq!(other_tenant.len(), 1);
    assert_eq!(other_tenant[0].id, "sandbox-other-tenant");
}

#[tokio::test]
async fn sandbox_service_requires_active_full_grants_and_valid_requests() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    insert_sandbox(&pool, "sandbox-full", "tenant-a", "Full", "active").await;
    insert_sandbox(
        &pool,
        "sandbox-read-only-grant",
        "tenant-a",
        "Read Only Grant",
        "active",
    )
    .await;
    insert_sandbox(
        &pool,
        "sandbox-disabled",
        "tenant-a",
        "Disabled",
        "disabled",
    )
    .await;

    insert_grant(&pool, "grant-full", "sandbox-full", "user-full", "full").await;
    insert_grant(
        &pool,
        "grant-read-only",
        "sandbox-read-only-grant",
        "user-read-only",
        "read_only",
    )
    .await;
    insert_grant(
        &pool,
        "grant-disabled",
        "sandbox-disabled",
        "user-disabled",
        "full",
    )
    .await;

    let service = DriveSandboxService::new(SqlSandboxStore::new(pool));

    service
        .require_full_access("tenant-a", "sandbox-full", "user", "user-full")
        .await
        .expect("an active full grant should allow mutations");

    let read_only = service
        .require_full_access(
            "tenant-a",
            "sandbox-read-only-grant",
            "user",
            "user-read-only",
        )
        .await
        .expect_err("a read-only grant must not allow mutations");
    assert!(matches!(
        read_only,
        DriveServiceError::PermissionDenied(message) if message == "sandbox is read only"
    ));

    let disabled = service
        .require_full_access("tenant-a", "sandbox-disabled", "user", "user-disabled")
        .await
        .expect_err("a disabled sandbox must not allow mutations");
    assert!(matches!(
        disabled,
        DriveServiceError::PermissionDenied(message) if message == "sandbox access is denied"
    ));

    let cross_tenant = service
        .require_full_access("tenant-b", "sandbox-full", "user", "user-full")
        .await
        .expect_err("a grant from another tenant must not allow mutations");
    assert!(matches!(
        cross_tenant,
        DriveServiceError::PermissionDenied(message) if message == "sandbox access is denied"
    ));

    let blank_subject = service
        .require_full_access("tenant-a", "sandbox-full", "user", " ")
        .await
        .expect_err("blank subjects must be rejected before authorization lookup");
    assert!(matches!(
        blank_subject,
        DriveServiceError::Validation(message)
            if message == "tenant, sandbox, and subject are required"
    ));

    let invalid_pagination = service
        .list_accessible("tenant-a", "user", "user-full", -1, 0)
        .await
        .expect_err("invalid pagination must be rejected before store access");
    assert!(matches!(
        invalid_pagination,
        DriveServiceError::Validation(message) if message == "pagination is invalid"
    ));
}

#[tokio::test]
async fn sandbox_service_collapses_multi_principal_grants_with_distinct_total() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    insert_sandbox(&pool, "sandbox-alpha", "tenant-a", "Alpha", "active").await;
    insert_sandbox(&pool, "sandbox-bravo", "tenant-a", "Bravo", "active").await;
    insert_sandbox(
        &pool,
        "sandbox-disabled",
        "tenant-a",
        "Disabled",
        "disabled",
    )
    .await;

    insert_typed_grant(
        &pool,
        "grant-alpha-user",
        "sandbox-alpha",
        "user",
        "user-a",
        "read_only",
    )
    .await;
    insert_typed_grant(
        &pool,
        "grant-alpha-organization",
        "sandbox-alpha",
        "organization",
        "organization-a",
        "full",
    )
    .await;
    insert_typed_grant(
        &pool,
        "grant-bravo-workspace",
        "sandbox-bravo",
        "workspace",
        "workspace-a",
        "read_only",
    )
    .await;
    insert_typed_grant(
        &pool,
        "grant-bravo-role",
        "sandbox-bravo",
        "role",
        "role-a",
        "read_only",
    )
    .await;
    insert_typed_grant(
        &pool,
        "grant-disabled-user",
        "sandbox-disabled",
        "user",
        "user-a",
        "full",
    )
    .await;

    let service = DriveSandboxService::new(SqlSandboxStore::new(pool));
    let principals = vec![
        EffectiveSandboxPrincipal {
            subject_type: "user".to_string(),
            subject_id: "user-a".to_string(),
        },
        EffectiveSandboxPrincipal {
            subject_type: "organization".to_string(),
            subject_id: "organization-a".to_string(),
        },
        EffectiveSandboxPrincipal {
            subject_type: "workspace".to_string(),
            subject_id: "workspace-a".to_string(),
        },
        EffectiveSandboxPrincipal {
            subject_type: "role".to_string(),
            subject_id: "role-a".to_string(),
        },
    ];

    let (first_page, first_total) = service
        .list_accessible_for_principals("tenant-a", &principals, 0, 1)
        .await
        .expect("the first multi-principal sandbox page should be readable");
    assert_eq!(first_total, 2, "total must count distinct sandboxes");
    assert_eq!(first_page.len(), 1);
    assert_eq!(first_page[0].id, "sandbox-alpha");
    assert_eq!(
        first_page[0].effective_access, "full",
        "any matching full grant must take precedence over read-only grants"
    );

    let (second_page, second_total) = service
        .list_accessible_for_principals("tenant-a", &principals, 1, 1)
        .await
        .expect("the second multi-principal sandbox page should be readable");
    assert_eq!(second_total, 2, "the distinct total must be page invariant");
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page[0].id, "sandbox-bravo");
    assert_eq!(second_page[0].effective_access, "read_only");

    let (all_sandboxes, total) = service
        .list_accessible_for_principals("tenant-a", &principals, 0, 20)
        .await
        .expect("the complete multi-principal sandbox page should be readable");
    assert_eq!(total, 2);
    assert_eq!(
        all_sandboxes
            .iter()
            .filter(|sandbox| sandbox.id == "sandbox-alpha")
            .count(),
        1,
        "multiple matching grants must collapse to one sandbox row"
    );
    assert_eq!(
        all_sandboxes
            .iter()
            .filter(|sandbox| sandbox.id == "sandbox-bravo")
            .count(),
        1,
        "multiple read-only grants must collapse to one sandbox row"
    );
}

#[tokio::test]
async fn read_only_sandbox_lifecycle_allows_reads_but_rejects_writes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    insert_sandbox(
        &pool,
        "sandbox-lifecycle-read-only",
        "tenant-a",
        "Read Only Lifecycle",
        "read_only",
    )
    .await;
    insert_grant(
        &pool,
        "grant-lifecycle-read-only",
        "sandbox-lifecycle-read-only",
        "user-a",
        "full",
    )
    .await;

    let service = DriveSandboxService::new(SqlSandboxStore::new(pool));

    service
        .require_read_access("tenant-a", "sandbox-lifecycle-read-only", "user", "user-a")
        .await
        .expect("a granted read-only lifecycle sandbox must remain readable");

    let write_error = service
        .require_full_access("tenant-a", "sandbox-lifecycle-read-only", "user", "user-a")
        .await
        .expect_err("a read-only lifecycle sandbox must reject writes");
    assert!(matches!(
        write_error,
        DriveServiceError::PermissionDenied(message) if message == "sandbox is read only"
    ));
}

async fn insert_sandbox(
    pool: &PgPool,
    sandbox_id: &str,
    tenant_id: &str,
    display_name: &str,
    lifecycle_status: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_volume (
            id, tenant_id, display_name, root_entry_id, provider_kind, provider_root_ref,
            lifecycle_status, default_access, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, 'local_filesystem', $5, $6, 'full', 1, 'test', 'test')",
    )
    .bind(sandbox_id)
    .bind(tenant_id)
    .bind(display_name)
    .bind(format!("root-entry:{sandbox_id}"))
    .bind(format!("opaque-provider-root:{sandbox_id}"))
    .bind(lifecycle_status)
    .execute(pool)
    .await
    .expect("sandbox fixture should be inserted");
}

async fn insert_grant(
    pool: &PgPool,
    grant_id: &str,
    sandbox_id: &str,
    subject_id: &str,
    access_level: &str,
) {
    insert_typed_grant(pool, grant_id, sandbox_id, "user", subject_id, access_level).await;
}

async fn insert_typed_grant(
    pool: &PgPool,
    grant_id: &str,
    sandbox_id: &str,
    subject_type: &str,
    subject_id: &str,
    access_level: &str,
) {
    sqlx::query(
        "INSERT INTO dr_drive_sandbox_grant (
            id, sandbox_id, subject_type, subject_id, access_level, granted_by
         ) VALUES ($1, $2, $3, $4, $5, 'test')",
    )
    .bind(grant_id)
    .bind(sandbox_id)
    .bind(subject_type)
    .bind(subject_id)
    .bind(access_level)
    .execute(pool)
    .await
    .expect("sandbox grant fixture should be inserted");
}
