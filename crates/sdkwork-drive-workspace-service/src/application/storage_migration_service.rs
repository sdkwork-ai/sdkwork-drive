use crate::domain::storage_migration::{
    DriveStorageMigration, DriveStorageMigrationItem, DriveStorageMigrationItemOutcome,
    DriveStorageMigrationItemStatus, DriveStorageMigrationStatus,
};
use crate::ports::storage_migration_copier::{
    DriveStorageMigrationCopier, StorageMigrationCopyRequest,
};
use crate::ports::storage_migration_store::DriveStorageMigrationStore;
use crate::ports::storage_provider_store::DriveStorageProviderStore;
use crate::DriveServiceError;

/// Cap on how many objects one drive call copies.
///
/// Copying is I/O bound and unbounded by nature — a tenant can hold millions of
/// objects — so progress is driven in bounded batches. A run is therefore
/// resumable by construction rather than by an extra mechanism: stopping early
/// simply leaves `pending` items for the next call.
const DEFAULT_MIGRATION_BATCH: i64 = 200;
const MAX_MIGRATION_BATCH: i64 = 1000;

#[derive(Debug, Clone)]
pub struct PlanStorageMigrationCommand {
    pub migration_id: String,
    pub tenant_id: String,
    pub name: String,
    pub source_provider_id: String,
    pub target_provider_id: String,
    /// Copy targets into a different bucket on the target provider. Defaults to
    /// the target provider's own bucket.
    pub target_bucket: Option<String>,
    /// Repoint objects and bindings once every object has been copied.
    pub apply_binding_switch: bool,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct RunStorageMigrationCommand {
    pub migration_id: String,
    pub tenant_id: String,
    pub batch_size: Option<i64>,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct GetStorageMigrationCommand {
    pub migration_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct ListStorageMigrationItemsCommand {
    pub migration_id: String,
    pub tenant_id: String,
    pub status: Option<DriveStorageMigrationItemStatus>,
    pub offset: i64,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct CancelStorageMigrationCommand {
    pub migration_id: String,
    pub tenant_id: String,
    pub operator_id: String,
}

/// Result of driving a run forward by one batch.
#[derive(Debug, Clone)]
pub struct StorageMigrationRunReport {
    pub migration: DriveStorageMigration,
    pub copied_this_batch: i64,
    pub failed_this_batch: i64,
    /// Whether the run reached `succeeded` on this call.
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct DriveStorageMigrationService<S, C, P> {
    store: S,
    copier: C,
    providers: P,
}

impl<S, C, P> DriveStorageMigrationService<S, C, P>
where
    S: DriveStorageMigrationStore,
    C: DriveStorageMigrationCopier,
    P: DriveStorageProviderStore,
{
    pub fn new(store: S, copier: C, providers: P) -> Self {
        Self {
            store,
            copier,
            providers,
        }
    }

    /// Open a run and queue every object that currently resolves to the source
    /// provider.
    ///
    /// The target is validated as `active` before anything is queued: a
    /// migration pointed at a disabled or deleted provider would copy bytes into
    /// a store that accepts no writes, and discovering that after moving
    /// gigabytes is the expensive way to learn it.
    pub async fn plan_storage_migration(
        &self,
        command: PlanStorageMigrationCommand,
    ) -> Result<DriveStorageMigration, DriveServiceError> {
        let source = self
            .providers
            .find_storage_provider(&command.source_provider_id)
            .await?
            .filter(|provider| provider.tenant_id == command.tenant_id)
            .ok_or_else(|| {
                DriveServiceError::NotFound(format!(
                    "storage provider {} not found",
                    command.source_provider_id
                ))
            })?;
        let target = self
            .providers
            .find_storage_provider(&command.target_provider_id)
            .await?
            .filter(|provider| provider.tenant_id == command.tenant_id)
            .ok_or_else(|| {
                DriveServiceError::NotFound(format!(
                    "storage provider {} not found",
                    command.target_provider_id
                ))
            })?;

        if source.id == target.id {
            return Err(DriveServiceError::Validation(
                "source and target storage providers must differ".to_string(),
            ));
        }
        // The source may legitimately be `disabled`: draining a retired
        // provider is the primary reason to migrate at all. The target may not.
        if source.status == "deleted" {
            return Err(DriveServiceError::Conflict(
                "source storage provider is deleted; its objects cannot be read".to_string(),
            ));
        }
        if target.status != "active" {
            return Err(DriveServiceError::Conflict(format!(
                "target storage provider is {}; migrations require an active target",
                target.status
            )));
        }

        let target_bucket = command
            .target_bucket
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(target.bucket.as_str())
            .to_string();

        let migration = crate::domain::storage_migration::NewDriveStorageMigration {
            id: command.migration_id,
            tenant_id: command.tenant_id,
            source_provider_id: command.source_provider_id,
            target_provider_id: command.target_provider_id,
            name: command.name,
            target_bucket: Some(target_bucket),
            apply_binding_switch: command.apply_binding_switch,
            created_by: command.operator_id,
        };

        self.store.create_storage_migration(&migration).await
    }

    pub async fn get_storage_migration(
        &self,
        command: GetStorageMigrationCommand,
    ) -> Result<DriveStorageMigration, DriveServiceError> {
        self.store
            .find_storage_migration(&command.migration_id)
            .await?
            .filter(|migration| migration.tenant_id == command.tenant_id)
            .ok_or_else(|| {
                DriveServiceError::NotFound(format!(
                    "storage migration {} not found",
                    command.migration_id
                ))
            })
    }

    /// List a tenant's runs, newest first.
    ///
    /// `status` is passed through as a `&str` rather than parsed here: the
    /// persisted column is the wire form, so an unknown value simply matches no
    /// rows — which is the honest answer to "show me runs in state X" when X is
    /// not a state. Parsing it into the enum would be stricter but would also
    /// make a stale client's filter a hard error instead of an empty page.
    pub async fn list_storage_migrations(
        &self,
        tenant_id: &str,
        status: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveStorageMigration>, DriveServiceError> {
        self.store
            .list_storage_migrations(tenant_id, status, offset, limit)
            .await
    }

    pub async fn list_storage_migration_items(
        &self,
        command: ListStorageMigrationItemsCommand,
    ) -> Result<Vec<DriveStorageMigrationItem>, DriveServiceError> {
        // Resolve the run first so a caller from another tenant gets `not found`
        // rather than an empty page that leaks the run's existence.
        self.get_storage_migration(GetStorageMigrationCommand {
            migration_id: command.migration_id.clone(),
            tenant_id: command.tenant_id,
        })
        .await?;
        self.store
            .list_storage_migration_items(
                &command.migration_id,
                command.status,
                command.offset,
                command.limit,
            )
            .await
    }

    pub async fn cancel_storage_migration(
        &self,
        command: CancelStorageMigrationCommand,
    ) -> Result<DriveStorageMigration, DriveServiceError> {
        let migration = self
            .get_storage_migration(GetStorageMigrationCommand {
                migration_id: command.migration_id.clone(),
                tenant_id: command.tenant_id.clone(),
            })
            .await?;
        if migration.status.is_terminal() {
            return Err(DriveServiceError::Conflict(format!(
                "storage migration is {}; only a resumable run can be cancelled",
                migration.status.as_str()
            )));
        }
        let moved = self
            .store
            .transition_storage_migration(
                &command.migration_id,
                migration.status,
                DriveStorageMigrationStatus::Cancelled,
                None,
                &command.operator_id,
            )
            .await?;
        if !moved {
            return Err(DriveServiceError::Conflict(
                "storage migration was advanced by another operator; retry with the current state"
                    .to_string(),
            ));
        }
        self.get_storage_migration(GetStorageMigrationCommand {
            migration_id: command.migration_id,
            tenant_id: command.tenant_id,
        })
        .await
    }
}

/// Clamp a requested batch size into the supported range.
///
/// Out-of-range values are clamped rather than rejected because the batch size
/// is a throughput knob, not a contract: refusing a run because someone typed
/// `5000` would be pedantry, while accepting it unbounded would let one call
/// monopolize a worker.
fn resolve_batch_size(requested: Option<i64>) -> i64 {
    match requested {
        Some(value) if value > 0 => value.min(MAX_MIGRATION_BATCH),
        _ => DEFAULT_MIGRATION_BATCH,
    }
}

/// Actions requested by an operator, as recorded against the run.
pub mod audit_action {
    pub const PLANNED: &str = "storage_migration.planned";
    pub const STARTED: &str = "storage_migration.started";
    pub const ITEM_COPIED: &str = "storage_migration.item_copied";
    pub const ITEM_FAILED: &str = "storage_migration.item_failed";
    pub const COMPLETED: &str = "storage_migration.completed";
    pub const FAILED: &str = "storage_migration.failed";
    pub const CANCELLED: &str = "storage_migration.cancelled";
}

/// The engine that moves bytes for a run.
///
/// Split from the service so the copy loop can be exercised against a fake
/// copier while the store and provider checks stay real.
pub struct DriveStorageMigrationEngine<S, C, P> {
    inner: DriveStorageMigrationService<S, C, P>,
}

impl<S, C, P> DriveStorageMigrationEngine<S, C, P>
where
    S: DriveStorageMigrationStore,
    C: DriveStorageMigrationCopier,
    P: DriveStorageProviderStore,
{
    pub fn new(store: S, copier: C, providers: P) -> Self {
        Self {
            inner: DriveStorageMigrationService::new(store, copier, providers),
        }
    }

    pub fn service(&self) -> &DriveStorageMigrationService<S, C, P> {
        &self.inner
    }

    /// Drive a run forward by one bounded batch.
    ///
    /// Every copied object is verified before its item is marked `copied`, so
    /// the run's counters only ever describe bytes that are provably on the
    /// target. A verification failure marks the item `failed` instead, which
    /// keeps the object reading from the source provider — the safe direction.
    pub async fn run_storage_migration(
        &self,
        command: RunStorageMigrationCommand,
    ) -> Result<StorageMigrationRunReport, DriveServiceError> {
        let service = &self.inner;
        let migration = service
            .get_storage_migration(GetStorageMigrationCommand {
                migration_id: command.migration_id.clone(),
                tenant_id: command.tenant_id.clone(),
            })
            .await?;

        if migration.status.is_terminal() {
            return Err(DriveServiceError::Conflict(format!(
                "storage migration is {}; a terminal run cannot be driven further",
                migration.status.as_str()
            )));
        }

        // Promote `pending` to `running` on the first drive. A lost race means
        // another worker got there first, which is fine: the copy below is
        // idempotent per item.
        if migration.status == DriveStorageMigrationStatus::Pending {
            let _ = service
                .store
                .transition_storage_migration(
                    &command.migration_id,
                    DriveStorageMigrationStatus::Pending,
                    DriveStorageMigrationStatus::Running,
                    None,
                    &command.operator_id,
                )
                .await?;
        }

        // Seed the queue once, on the first drive. Doing it here rather than at
        // plan time means a run planned while objects were still uploading
        // still captures everything, because the seed reads the current object
        // set.
        let existing_items = service
            .store
            .count_storage_migration_items(&command.migration_id)
            .await?;
        if existing_items.iter().sum::<i64>() == 0 {
            self.seed_pending_items(&migration, &command.operator_id)
                .await?;
        }

        let batch_size = resolve_batch_size(command.batch_size);
        // Attempt `pending` items first, then `failed` ones.
        //
        // Both matter, and for different reasons. `pending` is the forward
        // progress of a fresh run. `failed` is the whole point of resuming: an
        // item that failed on a transient upstream error is still `failed`
        // forever otherwise, so a run that hit one bad object would report
        // `still_pending == 0` with `failed > 0` on every subsequent drive and
        // never be able to finish. Retrying is safe because the copy is
        // idempotent — the same key is written to the same target locator — and
        // because nothing is re-pointed until the whole run is clean.
        //
        // Pending is drained first so a fresh run makes forward progress rather
        // than spending its whole batch on a handful of poison objects.
        let mut batch = service
            .store
            .list_storage_migration_items(
                &command.migration_id,
                Some(DriveStorageMigrationItemStatus::Pending),
                0,
                batch_size,
            )
            .await?;
        let remaining = batch_size - batch.len() as i64;
        if remaining > 0 {
            let mut retryable = service
                .store
                .list_storage_migration_items(
                    &command.migration_id,
                    Some(DriveStorageMigrationItemStatus::Failed),
                    0,
                    remaining,
                )
                .await?;
            batch.append(&mut retryable);
        }

        let mut copied_this_batch = 0i64;
        let mut failed_this_batch = 0i64;
        for item in &batch {
            let outcome = self.copy_one_item(item).await;
            service
                .store
                .record_storage_migration_item_outcome(
                    &command.migration_id,
                    &item.id,
                    &outcome,
                    &command.operator_id,
                )
                .await?;
            if outcome.copied {
                copied_this_batch += 1;
            } else {
                failed_this_batch += 1;
            }
        }

        let counts = service
            .store
            .count_storage_migration_items(&command.migration_id)
            .await?;
        let still_pending = counts[0];
        let failed = counts[2];
        // Completion requires *both* an empty pending queue and zero failures.
        // Checking only `still_pending == 0` would declare a run complete the
        // moment its last attempt was made, even if some of those attempts
        // failed and the objects are still only on the source.
        let completed = still_pending == 0 && failed == 0;

        // Only a fully copied, failure-free run is allowed to re-point live
        // data. Anything else stays resumable so the operator can retry the
        // stragglers instead of discovering a half-switched tenant.
        if completed && failed == 0 {
            self.finalize_success(&migration, &command.operator_id)
                .await?;
        } else if completed {
            service
                .store
                .transition_storage_migration(
                    &command.migration_id,
                    DriveStorageMigrationStatus::Running,
                    DriveStorageMigrationStatus::Failed,
                    Some(&format!(
                        "{failed} object(s) failed verification; nothing was re-pointed"
                    )),
                    &command.operator_id,
                )
                .await?;
        }

        let migration = service
            .get_storage_migration(GetStorageMigrationCommand {
                migration_id: command.migration_id,
                tenant_id: command.tenant_id,
            })
            .await?;

        Ok(StorageMigrationRunReport {
            completed: migration.status == DriveStorageMigrationStatus::Succeeded,
            migration,
            copied_this_batch,
            failed_this_batch,
        })
    }

    async fn seed_pending_items(
        &self,
        migration: &DriveStorageMigration,
        _operator_id: &str,
    ) -> Result<(), DriveServiceError> {
        let target_bucket = migration
            .target_bucket
            .clone()
            .ok_or_else(|| DriveServiceError::Internal("migration target bucket missing".into()))?;
        let objects = self
            .inner
            .store
            .list_storage_provider_objects_for_migration(
                &migration.tenant_id,
                &migration.source_provider_id,
            )
            .await?;

        let items = objects
            .into_iter()
            .map(|object| {
                crate::domain::storage_migration::NewDriveStorageMigrationItem {
                    // Deterministic, so re-seeding after a crash cannot create a
                    // second queue for the same object; the unique constraint on
                    // `(migration_id, storage_object_id)` then makes the insert
                    // idempotent.
                    id: format!("{}:{}", migration.id, object.id),
                    storage_object_id: object.id,
                    source_bucket: object.bucket,
                    // The key is preserved exactly. It is provider-neutral and
                    // carries no provider identity, so the target keeps the same
                    // path and a rollback is a pure metadata change.
                    source_object_key: object.object_key.clone(),
                    target_bucket: target_bucket.clone(),
                    target_object_key: object.object_key,
                    content_type: object.content_type,
                    content_length: object.content_length,
                    checksum_sha256_hex: object.checksum_sha256_hex,
                }
            })
            .collect::<Vec<_>>();

        if items.is_empty() {
            return Ok(());
        }
        // Header already exists; only the queue is appended.
        self.inner
            .store
            .append_storage_migration_items(&migration.id, &items)
            .await
    }

    async fn copy_one_item(
        &self,
        item: &DriveStorageMigrationItem,
    ) -> DriveStorageMigrationItemOutcome {
        let request = StorageMigrationCopyRequest {
            source_provider_id: item.source_provider_id.clone(),
            source_bucket: item.source_bucket.clone(),
            source_object_key: item.source_object_key.clone(),
            target_provider_id: item.target_provider_id.clone(),
            target_bucket: item.target_bucket.clone(),
            target_object_key: item.target_object_key.clone(),
            content_type: item.content_type.clone(),
            expected_checksum_sha256_hex: item.checksum_sha256_hex.clone(),
        };
        match self.inner.copier.copy_for_migration(request).await {
            Ok(outcome) => {
                // Defence in depth: the copier is contractually required to
                // verify, but the engine will not mark an object copied on the
                // strength of a checksum it never saw match the source.
                if outcome.verified_checksum_sha256_hex.as_deref()
                    != Some(item.checksum_sha256_hex.as_str())
                {
                    return DriveStorageMigrationItemOutcome {
                        copied: false,
                        bytes: 0,
                        verified_checksum_sha256_hex: outcome.verified_checksum_sha256_hex,
                        failure_message: Some(
                            "target checksum does not match the recorded source checksum"
                                .to_string(),
                        ),
                    };
                }
                DriveStorageMigrationItemOutcome {
                    copied: true,
                    bytes: outcome.bytes,
                    verified_checksum_sha256_hex: outcome.verified_checksum_sha256_hex,
                    failure_message: None,
                }
            }
            // A failed object is never fatal to the run: the bytes simply stay
            // where they are, which is the safe direction.
            Err(error) => DriveStorageMigrationItemOutcome {
                copied: false,
                bytes: 0,
                verified_checksum_sha256_hex: None,
                failure_message: Some(error.message().to_string()),
            },
        }
    }

    async fn finalize_success(
        &self,
        migration: &DriveStorageMigration,
        operator_id: &str,
    ) -> Result<(), DriveServiceError> {
        let target_bucket = migration
            .target_bucket
            .clone()
            .ok_or_else(|| DriveServiceError::Internal("migration target bucket missing".into()))?;

        if migration.apply_binding_switch {
            self.inner
                .store
                .apply_storage_migration_repoint(
                    &migration.id,
                    &migration.target_provider_id,
                    &target_bucket,
                    operator_id,
                )
                .await?;
        }

        self.inner
            .store
            .transition_storage_migration(
                &migration.id,
                DriveStorageMigrationStatus::Running,
                DriveStorageMigrationStatus::Succeeded,
                None,
                operator_id,
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_size_defaults_when_absent_or_non_positive() {
        assert_eq!(resolve_batch_size(None), DEFAULT_MIGRATION_BATCH);
        assert_eq!(resolve_batch_size(Some(0)), DEFAULT_MIGRATION_BATCH);
        assert_eq!(resolve_batch_size(Some(-5)), DEFAULT_MIGRATION_BATCH);
    }

    #[test]
    fn batch_size_is_clamped_instead_of_rejected() {
        assert_eq!(resolve_batch_size(Some(50)), 50);
        assert_eq!(
            resolve_batch_size(Some(MAX_MIGRATION_BATCH)),
            MAX_MIGRATION_BATCH
        );
        assert_eq!(
            resolve_batch_size(Some(MAX_MIGRATION_BATCH + 1)),
            MAX_MIGRATION_BATCH
        );
        assert_eq!(resolve_batch_size(Some(i64::MAX)), MAX_MIGRATION_BATCH);
    }
}
