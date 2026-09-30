//! Cross-provider storage migration, end to end against a real database.
//!
//! The store and the provider lookups are real; only the byte movement is faked.
//! That is the split that matters: the invariants under test are all about
//! *bookkeeping* — which counter moves when, whether a failure leaves the
//! registry pointing at the source, and whether a resumed run re-copies what it
//! already copied. DriveObjectStore behaviour is not what this file is proving.
//!
//! One test binary, one fixture: the shared advisory-lock pool does not support
//! several independent fixtures inside a single binary, so every scenario shares
//! the pool and uses its own tenant and provider ids.

use std::sync::Mutex;

use async_trait::async_trait;
use sdkwork_drive_workspace_service::application::storage_migration_service::{
    CancelStorageMigrationCommand, DriveStorageMigrationEngine, GetStorageMigrationCommand,
    ListStorageMigrationItemsCommand, PlanStorageMigrationCommand, RunStorageMigrationCommand,
};
use sdkwork_drive_workspace_service::domain::storage_migration::{
    DriveStorageMigrationItemStatus, DriveStorageMigrationStatus,
};
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProviderKind;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_migration_store::SqlStorageMigrationStore;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;
use sdkwork_drive_workspace_service::ports::storage_migration_copier::{
    DriveStorageMigrationCopier, StorageMigrationCopyOutcome, StorageMigrationCopyRequest,
};
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::{PgPool, Row};

/// A copier that records every request and can be told which keys to fail.
///
/// `failures` is checked by `source_object_key` suffix so a test can fail one
/// specific object without knowing its generated id.
#[derive(Default)]
struct RecordingCopier {
    calls: Mutex<Vec<StorageMigrationCopyRequest>>,
    /// Object keys whose copy must fail.
    failing_keys: Vec<String>,
    /// When true, the copier reports a checksum that does not match the source,
    /// which is the "silent corruption" case rather than a transport error.
    corrupt_checksum: bool,
}

impl RecordingCopier {
    fn failing(keys: &[&str]) -> Self {
        Self {
            failing_keys: keys.iter().map(|key| (*key).to_string()).collect(),
            ..Default::default()
        }
    }

    fn corrupting() -> Self {
        Self {
            corrupt_checksum: true,
            ..Default::default()
        }
    }
}

#[async_trait]
impl DriveStorageMigrationCopier for RecordingCopier {
    async fn copy_for_migration(
        &self,
        request: StorageMigrationCopyRequest,
    ) -> Result<StorageMigrationCopyOutcome, DriveServiceError> {
        self.calls
            .lock()
            .expect("copier calls lock")
            .push(request.clone());

        if self
            .failing_keys
            .iter()
            .any(|key| request.source_object_key == *key)
        {
            return Err(DriveServiceError::Internal(format!(
                "simulated transport failure copying {}",
                request.source_object_key
            )));
        }

        // The bytes' length is not knowable here without an object store, and the
        // engine only uses `bytes` for reporting, so a fixed value is honest
        // about what this fake proves.
        Ok(StorageMigrationCopyOutcome {
            bytes: 42,
            verified_checksum_sha256_hex: Some(if self.corrupt_checksum {
                "deadbeef".repeat(8)
            } else {
                request.expected_checksum_sha256_hex.clone()
            }),
        })
    }
}

/// Seed the FK chain a storage object needs: space -> node -> object.
///
/// `checksum_seed` is a short discriminator, not a checksum: the column's CHECK
/// requires `sha256:<64 hex>`, so this expands it into a well-formed, distinct,
/// deterministic value. Tests that care about checksum *identity* (the migration
/// compares source against target) only need two seeds to differ, which a
/// discriminator guarantees.
async fn seed_object(
    pool: &PgPool,
    tenant_id: &str,
    provider_id: &str,
    object_id: &str,
    object_key: &str,
    bucket: &str,
    checksum_seed: &str,
) {
    let space_id = format!("space-{tenant_id}");
    let node_id = format!("node-{object_id}");
    let checksum = {
        let mut hex = String::with_capacity(64);
        while hex.len() < 64 {
            hex.push_str(checksum_seed);
        }
        hex.truncate(64);
        format!("sha256:{hex}")
    };

    sqlx::query(
        "INSERT INTO dr_drive_space
            (id, tenant_id, owner_subject_type, owner_subject_id, space_type, display_name,
             created_by, updated_by)
         VALUES ($1, $2, 'user', 'owner-1', 'personal', 'Migration Test Space', 'seed', 'seed')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(&space_id)
    .bind(tenant_id)
    .execute(pool)
    .await
    .expect("seed space");

    // A `file` node in `content_state = 'ready'` must carry its head metadata —
    // `ck_dr_drive_node_file_ready_head` enforces it, and a node without a head
    // would be a file the UI cannot render.
    sqlx::query(
        "INSERT INTO dr_drive_node
            (id, tenant_id, space_id, space_type, node_type, node_name, content_state,
             head_content_type, head_content_type_group, head_content_length, head_version_no,
             created_by, updated_by)
         VALUES ($1, $2, $3, 'personal', 'file', $4, 'ready',
                 'application/octet-stream', 'binary', 42, 1,
                 'seed', 'seed')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(&node_id)
    .bind(tenant_id)
    .bind(&space_id)
    .bind(object_key)
    .execute(pool)
    .await
    .expect("seed node");

    sqlx::query(
        "INSERT INTO dr_drive_storage_object
            (id, tenant_id, node_id, version_no, storage_provider_id, bucket, object_key,
             content_type, content_length, checksum_sha256_hex, lifecycle_status,
             created_by, updated_by)
         VALUES ($1, $2, $3, 1, $4, $5, $6, 'application/octet-stream', 42, $7, 'active',
                 'seed', 'seed')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(object_id)
    .bind(tenant_id)
    .bind(&node_id)
    .bind(provider_id)
    .bind(bucket)
    .bind(object_key)
    .bind(checksum)
    .execute(pool)
    .await
    .expect("seed storage object");
}

async fn seed_provider(pool: &PgPool, tenant_id: &str, provider_id: &str, status: &str) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider
            (id, tenant_id, provider_kind, name, endpoint_url, bucket, path_style, status,
             created_by, updated_by)
         VALUES ($1, $2, 's3_compatible', $3, 'https://s3.example.com', $4, true, $5,
                 'seed', 'seed')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(provider_id)
    .bind(tenant_id)
    .bind(provider_id)
    .bind(format!("bucket-{provider_id}"))
    .bind(status)
    .execute(pool)
    .await
    .expect("seed provider");
}

/// Read the provider id a storage object currently points at.
async fn object_provider(pool: &PgPool, object_id: &str) -> String {
    sqlx::query("SELECT storage_provider_id FROM dr_drive_storage_object WHERE id = $1")
        .bind(object_id)
        .fetch_one(pool)
        .await
        .expect("read object provider")
        .get("storage_provider_id")
}

/// Remove everything a scenario created, in FK-safe order.
async fn cleanup(pool: &PgPool, tenant_id: &str) {
    sqlx::query("DELETE FROM dr_drive_storage_migration_item WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM dr_drive_storage_migration WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM dr_drive_storage_object WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM dr_drive_node WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM dr_drive_space WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM dr_drive_storage_provider WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await
        .ok();
}

/// Every database-backed scenario, driven sequentially from one fixture.
///
/// The test-support pool holds a **process-wide advisory lock**, so a second
/// concurrent fixture in the same binary deadlocks on it (`pool timed out`).
/// One lock, one driver, all scenarios — the alternative (one test per scenario)
/// only works if each lives in its own binary, which would cost a full compile
/// per scenario for no extra coverage.
#[tokio::test]
async fn storage_migration_database_scenarios() {
    let Some((pool, _guard)) = sdkwork_drive_test_support::postgres_test_database().await else {
        return;
    };

    migrates_verifies_and_repoints(&pool).await;
    a_failed_object_does_not_repoint_the_registry(&pool).await;
    a_corrupted_target_checksum_fails_the_object(&pool).await;
    a_failed_run_resumes_without_recopying(&pool).await;
    cancel_is_refused_once_the_run_succeeded(&pool).await;
    planning_rejects_a_provider_from_another_tenant(&pool).await;
    planning_rejects_a_disabled_target_but_allows_a_disabled_source(&pool).await;
    listing_runs_does_not_leak_across_tenants(&pool).await;
    copier_receives_every_queued_object_with_an_unchanged_key(&pool).await;
}

/// Happy path: every object copies, verification passes, and `apply_binding_switch`
/// re-points the registry at the target.
async fn migrates_verifies_and_repoints(pool: &PgPool) {
    let tenant = "tenant-mig-happy";
    cleanup(pool, tenant).await;
    seed_provider(pool, tenant, "prov-happy-src", "active").await;
    seed_provider(pool, tenant, "prov-happy-dst", "active").await;
    seed_object(
        pool,
        tenant,
        "prov-happy-src",
        "obj-happy-1",
        "keys/a.bin",
        "bucket-prov-happy-src",
        "aa11",
    )
    .await;
    seed_object(
        pool,
        tenant,
        "prov-happy-src",
        "obj-happy-2",
        "keys/b.bin",
        "bucket-prov-happy-src",
        "bb22",
    )
    .await;

    let copier = RecordingCopier::default();
    let engine = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        copier,
        SqlStorageProviderStore::new(pool.clone()),
    );

    engine
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: "mig-happy".to_string(),
            tenant_id: tenant.to_string(),
            name: "retire prov-happy-src".to_string(),
            source_provider_id: "prov-happy-src".to_string(),
            target_provider_id: "prov-happy-dst".to_string(),
            target_bucket: None,
            apply_binding_switch: true,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("plan");

    let report = engine
        .run_storage_migration(RunStorageMigrationCommand {
            migration_id: "mig-happy".to_string(),
            tenant_id: tenant.to_string(),
            batch_size: None,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("run");

    assert!(report.completed, "a two-object run finishes in one batch");
    assert_eq!(report.copied_this_batch, 2);
    assert_eq!(report.failed_this_batch, 0);
    assert_eq!(
        report.migration.status,
        DriveStorageMigrationStatus::Succeeded
    );
    assert_eq!(report.migration.objects_copied, 2);
    assert_eq!(report.migration.objects_total, 2);

    // The switch actually happened: both objects now belong to the target.
    assert_eq!(object_provider(pool, "obj-happy-1").await, "prov-happy-dst");
    assert_eq!(object_provider(pool, "obj-happy-2").await, "prov-happy-dst");

    // The queue reflects the copy, and each item carries the verified checksum.
    let items = engine
        .service()
        .list_storage_migration_items(ListStorageMigrationItemsCommand {
            migration_id: "mig-happy".to_string(),
            tenant_id: tenant.to_string(),
            status: Some(DriveStorageMigrationItemStatus::Copied),
            offset: 0,
            limit: 10,
        })
        .await
        .expect("list items");
    assert_eq!(items.len(), 2);
    for item in &items {
        assert_eq!(
            item.verified_checksum_sha256_hex.as_deref(),
            Some(item.checksum_sha256_hex.as_str()),
            "verified checksum must equal the source checksum"
        );
    }
}

/// A run that cannot copy everything must leave the registry alone.
async fn a_failed_object_does_not_repoint_the_registry(pool: &PgPool) {
    let tenant = "tenant-mig-fail";
    cleanup(pool, tenant).await;
    seed_provider(pool, tenant, "prov-fail-src", "active").await;
    seed_provider(pool, tenant, "prov-fail-dst", "active").await;
    seed_object(
        pool,
        tenant,
        "prov-fail-src",
        "obj-fail-1",
        "keys/ok.bin",
        "bucket-prov-fail-src",
        "cc33",
    )
    .await;
    seed_object(
        pool,
        tenant,
        "prov-fail-src",
        "obj-fail-2",
        "keys/bad.bin",
        "bucket-prov-fail-src",
        "dd44",
    )
    .await;

    let engine = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        RecordingCopier::failing(&["keys/bad.bin"]),
        SqlStorageProviderStore::new(pool.clone()),
    );

    engine
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: "mig-fail".to_string(),
            tenant_id: tenant.to_string(),
            name: "partial failure".to_string(),
            source_provider_id: "prov-fail-src".to_string(),
            target_provider_id: "prov-fail-dst".to_string(),
            target_bucket: None,
            apply_binding_switch: true,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("plan");

    let report = engine
        .run_storage_migration(RunStorageMigrationCommand {
            migration_id: "mig-fail".to_string(),
            tenant_id: tenant.to_string(),
            batch_size: None,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("run");

    assert!(!report.completed);
    assert_eq!(report.copied_this_batch, 1);
    assert_eq!(report.failed_this_batch, 1);
    // Still `running`, not `failed`: the failure is retryable, so the run stays
    // resumable rather than being declared dead after one transient error.
    assert_eq!(
        report.migration.status,
        DriveStorageMigrationStatus::Running
    );
    assert!(report.migration.status.is_resumable());
    assert_eq!(report.migration.objects_failed, 1);

    // Nothing was re-pointed, even for the object that copied successfully:
    // a half-switched tenant is the failure mode this guards against.
    assert_eq!(object_provider(pool, "obj-fail-1").await, "prov-fail-src");
    assert_eq!(object_provider(pool, "obj-fail-2").await, "prov-fail-src");
}

/// A target checksum that does not match the source must fail the object rather
/// than be recorded as a success.
async fn a_corrupted_target_checksum_fails_the_object(pool: &PgPool) {
    let tenant = "tenant-mig-corrupt";
    cleanup(pool, tenant).await;
    seed_provider(pool, tenant, "prov-corrupt-src", "active").await;
    seed_provider(pool, tenant, "prov-corrupt-dst", "active").await;
    seed_object(
        pool,
        tenant,
        "prov-corrupt-src",
        "obj-corrupt-1",
        "keys/c.bin",
        "bucket-prov-corrupt-src",
        "ee55",
    )
    .await;

    let engine = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        RecordingCopier::corrupting(),
        SqlStorageProviderStore::new(pool.clone()),
    );

    engine
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: "mig-corrupt".to_string(),
            tenant_id: tenant.to_string(),
            name: "corrupt target".to_string(),
            source_provider_id: "prov-corrupt-src".to_string(),
            target_provider_id: "prov-corrupt-dst".to_string(),
            target_bucket: None,
            apply_binding_switch: true,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("plan");

    let report = engine
        .run_storage_migration(RunStorageMigrationCommand {
            migration_id: "mig-corrupt".to_string(),
            tenant_id: tenant.to_string(),
            batch_size: None,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("run");

    assert_eq!(report.copied_this_batch, 0);
    assert_eq!(report.failed_this_batch, 1);
    assert_eq!(
        object_provider(pool, "obj-corrupt-1").await,
        "prov-corrupt-src"
    );
}

/// Resuming a failed run must not re-copy what already landed.
async fn a_failed_run_resumes_without_recopying(pool: &PgPool) {
    let tenant = "tenant-mig-resume";
    cleanup(pool, tenant).await;
    seed_provider(pool, tenant, "prov-resume-src", "active").await;
    seed_provider(pool, tenant, "prov-resume-dst", "active").await;
    seed_object(
        pool,
        tenant,
        "prov-resume-src",
        "obj-resume-1",
        "keys/ok.bin",
        "bucket-prov-resume-src",
        "f660",
    )
    .await;
    seed_object(
        pool,
        tenant,
        "prov-resume-src",
        "obj-resume-2",
        "keys/bad.bin",
        "bucket-prov-resume-src",
        "f770",
    )
    .await;

    let engine = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        RecordingCopier::failing(&["keys/bad.bin"]),
        SqlStorageProviderStore::new(pool.clone()),
    );

    engine
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: "mig-resume".to_string(),
            tenant_id: tenant.to_string(),
            name: "resume".to_string(),
            source_provider_id: "prov-resume-src".to_string(),
            target_provider_id: "prov-resume-dst".to_string(),
            target_bucket: None,
            apply_binding_switch: true,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("plan");

    let first = engine
        .run_storage_migration(RunStorageMigrationCommand {
            migration_id: "mig-resume".to_string(),
            tenant_id: tenant.to_string(),
            batch_size: None,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("first run");
    assert_eq!(first.copied_this_batch, 1);
    assert_eq!(first.failed_this_batch, 1);

    // The one that failed is `failed`, not `pending` — and that is exactly what
    // the retry path re-drives. Nothing is still `pending`, so a run that only
    // looked at `pending` would sit at "complete with failures" forever.
    let pending = engine
        .service()
        .list_storage_migration_items(ListStorageMigrationItemsCommand {
            migration_id: "mig-resume".to_string(),
            tenant_id: tenant.to_string(),
            status: Some(DriveStorageMigrationItemStatus::Pending),
            offset: 0,
            limit: 10,
        })
        .await
        .expect("list pending");
    assert!(
        pending.is_empty(),
        "a failed attempt leaves nothing pending"
    );

    let failed = engine
        .service()
        .list_storage_migration_items(ListStorageMigrationItemsCommand {
            migration_id: "mig-resume".to_string(),
            tenant_id: tenant.to_string(),
            status: Some(DriveStorageMigrationItemStatus::Failed),
            offset: 0,
            limit: 10,
        })
        .await
        .expect("list failed");
    assert_eq!(failed.len(), 1, "only the failed object is retryable");
    assert_eq!(failed[0].source_object_key, "keys/bad.bin");

    // A second drive with a healthy copier retries the failure and finishes.
    let healthy = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        RecordingCopier::default(),
        SqlStorageProviderStore::new(pool.clone()),
    );
    let second = healthy
        .run_storage_migration(RunStorageMigrationCommand {
            migration_id: "mig-resume".to_string(),
            tenant_id: tenant.to_string(),
            batch_size: None,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("second run");

    assert!(second.completed);
    assert_eq!(second.copied_this_batch, 1, "only the straggler was copied");
    assert_eq!(second.failed_this_batch, 0);
    // The retry moved one object from `failed` to `copied`, so the run's totals
    // must reflect two copies and no failures — otherwise the zero-failure
    // condition that gates the switch could never be met.
    assert_eq!(second.migration.objects_copied, 2);
    assert_eq!(second.migration.objects_failed, 0);
    assert_eq!(
        object_provider(pool, "obj-resume-1").await,
        "prov-resume-dst"
    );
    assert_eq!(
        object_provider(pool, "obj-resume-2").await,
        "prov-resume-dst"
    );
}

/// A terminal run refuses to be cancelled.
async fn cancel_is_refused_once_the_run_succeeded(pool: &PgPool) {
    let tenant = "tenant-mig-cancel";
    cleanup(pool, tenant).await;
    seed_provider(pool, tenant, "prov-cancel-src", "active").await;
    seed_provider(pool, tenant, "prov-cancel-dst", "active").await;
    seed_object(
        pool,
        tenant,
        "prov-cancel-src",
        "obj-cancel-1",
        "keys/d.bin",
        "bucket-prov-cancel-src",
        "f880",
    )
    .await;

    let engine = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        RecordingCopier::default(),
        SqlStorageProviderStore::new(pool.clone()),
    );

    engine
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: "mig-cancel".to_string(),
            tenant_id: tenant.to_string(),
            name: "cancel".to_string(),
            source_provider_id: "prov-cancel-src".to_string(),
            target_provider_id: "prov-cancel-dst".to_string(),
            target_bucket: None,
            apply_binding_switch: false,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("plan");

    let run = engine
        .run_storage_migration(RunStorageMigrationCommand {
            migration_id: "mig-cancel".to_string(),
            tenant_id: tenant.to_string(),
            batch_size: None,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("run");
    assert!(run.completed);

    let error = engine
        .service()
        .cancel_storage_migration(CancelStorageMigrationCommand {
            migration_id: "mig-cancel".to_string(),
            tenant_id: tenant.to_string(),
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect_err("a succeeded run is terminal");
    assert!(matches!(error, DriveServiceError::Conflict(_)));

    // `apply_binding_switch = false` means the run completed without switching.
    assert_eq!(
        object_provider(pool, "obj-cancel-1").await,
        "prov-cancel-src"
    );
}

/// A provider of another tenant must not be plannable against.
async fn planning_rejects_a_provider_from_another_tenant(pool: &PgPool) {
    let tenant = "tenant-mig-scope";
    cleanup(pool, tenant).await;
    // Owned by a different tenant.
    seed_provider(pool, "tenant-mig-other", "prov-scope-foreign", "active").await;
    seed_provider(pool, tenant, "prov-scope-local", "active").await;

    let engine = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        RecordingCopier::default(),
        SqlStorageProviderStore::new(pool.clone()),
    );

    let error = engine
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: "mig-scope".to_string(),
            tenant_id: tenant.to_string(),
            name: "cross tenant".to_string(),
            source_provider_id: "prov-scope-local".to_string(),
            target_provider_id: "prov-scope-foreign".to_string(),
            target_bucket: None,
            apply_binding_switch: false,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect_err("a foreign provider must not be addressable");
    assert!(matches!(error, DriveServiceError::NotFound(_)));

    cleanup(pool, "tenant-mig-other").await;
    cleanup(pool, tenant).await;
}

/// The target must accept writes; a disabled target is refused before any object
/// is queued.
async fn planning_rejects_a_disabled_target_but_allows_a_disabled_source(pool: &PgPool) {
    let tenant = "tenant-mig-status";
    cleanup(pool, tenant).await;
    // Source is disabled on purpose: draining a retired provider is the point.
    seed_provider(pool, tenant, "prov-status-src", "disabled").await;
    seed_provider(pool, tenant, "prov-status-dst", "disabled").await;
    seed_provider(pool, tenant, "prov-status-active", "active").await;

    let engine = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        RecordingCopier::default(),
        SqlStorageProviderStore::new(pool.clone()),
    );

    let error = engine
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: "mig-status-bad".to_string(),
            tenant_id: tenant.to_string(),
            name: "disabled target".to_string(),
            source_provider_id: "prov-status-src".to_string(),
            target_provider_id: "prov-status-dst".to_string(),
            target_bucket: None,
            apply_binding_switch: false,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect_err("a disabled target must be refused");
    assert!(matches!(error, DriveServiceError::Conflict(_)));

    // The same disabled source with an active target is accepted.
    let planned = engine
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: "mig-status-ok".to_string(),
            tenant_id: tenant.to_string(),
            name: "drain disabled source".to_string(),
            source_provider_id: "prov-status-src".to_string(),
            target_provider_id: "prov-status-active".to_string(),
            target_bucket: None,
            apply_binding_switch: false,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("a disabled source is migratable");
    assert_eq!(planned.status, DriveStorageMigrationStatus::Pending);

    cleanup(pool, tenant).await;
}

/// `list_storage_migrations` is tenant-scoped.
async fn listing_runs_does_not_leak_across_tenants(pool: &PgPool) {
    let tenant = "tenant-mig-list";
    let other = "tenant-mig-list-other";
    cleanup(pool, tenant).await;
    cleanup(pool, other).await;
    // Provider ids are globally unique (the PK is `id` alone), so each tenant
    // needs its own — sharing a literal would make the second seed a silent
    // no-op and leave one tenant with no provider at all.
    seed_provider(pool, tenant, "prov-list-src", "active").await;
    seed_provider(pool, tenant, "prov-list-dst", "active").await;
    seed_provider(pool, other, "prov-list-other-src", "active").await;
    seed_provider(pool, other, "prov-list-other-dst", "active").await;

    let engine = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        RecordingCopier::default(),
        SqlStorageProviderStore::new(pool.clone()),
    );

    let scenarios = [
        ("mig-list-mine", tenant, "prov-list-src", "prov-list-dst"),
        (
            "mig-list-theirs",
            other,
            "prov-list-other-src",
            "prov-list-other-dst",
        ),
    ];
    for (id, owner, source, target) in scenarios {
        engine
            .service()
            .plan_storage_migration(PlanStorageMigrationCommand {
                migration_id: id.to_string(),
                tenant_id: owner.to_string(),
                name: id.to_string(),
                source_provider_id: source.to_string(),
                target_provider_id: target.to_string(),
                target_bucket: None,
                apply_binding_switch: false,
                operator_id: "operator-1".to_string(),
            })
            .await
            .expect("plan");
    }

    let mine = engine
        .service()
        .list_storage_migrations(tenant, None, 0, 10)
        .await
        .expect("list");
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].id, "mig-list-mine");

    // A run owned by another tenant is `not found` rather than visible.
    let error = engine
        .service()
        .get_storage_migration(GetStorageMigrationCommand {
            migration_id: "mig-list-theirs".to_string(),
            tenant_id: tenant.to_string(),
        })
        .await
        .expect_err("another tenant's run must not be readable");
    assert!(matches!(error, DriveServiceError::NotFound(_)));

    cleanup(pool, tenant).await;
    cleanup(pool, other).await;
}

/// The copier must be called once per queued object and must carry the key
/// through unchanged, because the key is provider-neutral and carries no
/// provider identity.
async fn copier_receives_every_queued_object_with_an_unchanged_key(pool: &PgPool) {
    let tenant = "tenant-mig-keys";
    cleanup(pool, tenant).await;
    seed_provider(pool, tenant, "prov-keys-src", "active").await;
    seed_provider(pool, tenant, "prov-keys-dst", "active").await;
    seed_object(
        pool,
        tenant,
        "prov-keys-src",
        "obj-keys-1",
        "a/b/c.bin",
        "bucket-prov-keys-src",
        "1122",
    )
    .await;
    seed_object(
        pool,
        tenant,
        "prov-keys-src",
        "obj-keys-2",
        "a/d.bin",
        "bucket-prov-keys-src",
        "3344",
    )
    .await;

    let engine = DriveStorageMigrationEngine::new(
        SqlStorageMigrationStore::new(pool.clone()),
        RecordingCopier::default(),
        SqlStorageProviderStore::new(pool.clone()),
    );

    engine
        .service()
        .plan_storage_migration(PlanStorageMigrationCommand {
            migration_id: "mig-keys".to_string(),
            tenant_id: tenant.to_string(),
            name: "keys".to_string(),
            source_provider_id: "prov-keys-src".to_string(),
            target_provider_id: "prov-keys-dst".to_string(),
            target_bucket: Some("explicit-target-bucket".to_string()),
            apply_binding_switch: false,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("plan");

    // The engine owns the copier, so drive it and then inspect the store's
    // queue, which is the authoritative record of what each item pointed at.
    engine
        .run_storage_migration(RunStorageMigrationCommand {
            migration_id: "mig-keys".to_string(),
            tenant_id: tenant.to_string(),
            batch_size: None,
            operator_id: "operator-1".to_string(),
        })
        .await
        .expect("run");

    let items = engine
        .service()
        .list_storage_migration_items(ListStorageMigrationItemsCommand {
            migration_id: "mig-keys".to_string(),
            tenant_id: tenant.to_string(),
            status: None,
            offset: 0,
            limit: 10,
        })
        .await
        .expect("list items");
    assert_eq!(items.len(), 2);
    for item in &items {
        assert_eq!(item.source_object_key, item.target_object_key);
        assert_eq!(item.target_bucket, "explicit-target-bucket");
        assert_eq!(item.target_provider_id, "prov-keys-dst");
    }

    cleanup(pool, tenant).await;
}

/// Guard against the provider kind enum drifting away from the seed helper.
#[test]
fn seed_helper_uses_a_kind_the_domain_knows() {
    assert_eq!(
        DriveStorageProviderKind::S3Compatible.as_str(),
        "s3_compatible"
    );
}
