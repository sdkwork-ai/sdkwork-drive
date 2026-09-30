use async_trait::async_trait;

use crate::domain::storage_migration::{
    DriveStorageMigration, DriveStorageMigrationItem, DriveStorageMigrationItemOutcome,
    DriveStorageMigrationItemStatus, DriveStorageMigrationSourceObject,
    DriveStorageMigrationStatus, NewDriveStorageMigration, NewDriveStorageMigrationItem,
};
use crate::DriveServiceError;

/// Persistence for cross-provider storage migrations.
///
/// The store owns both the run header and its per-object items because the two
/// have to move together: every counter on the header is derived from item
/// transitions, and a store that let the two drift would report a run as
/// complete while objects were still pending.
#[async_trait]
pub trait DriveStorageMigrationStore: Send + Sync {
    /// Open a run with no queued items.
    ///
    /// The queue is appended separately by
    /// [`Self::append_storage_migration_items`], because deriving it can be a
    /// large read that should not be held open inside the creating transaction.
    async fn create_storage_migration(
        &self,
        migration: &NewDriveStorageMigration,
    ) -> Result<DriveStorageMigration, DriveServiceError>;

    /// Append objects to a run's queue.
    ///
    /// Must be idempotent per object: the item id is derived from the run and
    /// the object, so a re-seed after a crash inserts nothing new and the run
    /// cannot end up copying the same object twice into two queue slots.
    async fn append_storage_migration_items(
        &self,
        migration_id: &str,
        items: &[NewDriveStorageMigrationItem],
    ) -> Result<(), DriveServiceError>;

    /// Live objects that currently resolve to a provider.
    ///
    /// `lifecycle_status = 'active'` only, because a soft-deleted object is on
    /// its way out and migrating it would move bytes nobody will read.
    async fn list_storage_provider_objects_for_migration(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Vec<DriveStorageMigrationSourceObject>, DriveServiceError>;

    async fn find_storage_migration(
        &self,
        migration_id: &str,
    ) -> Result<Option<DriveStorageMigration>, DriveServiceError>;

    async fn list_storage_migrations(
        &self,
        tenant_id: &str,
        status: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveStorageMigration>, DriveServiceError>;

    /// Items of a run filtered by status, ordered by id for a stable cursor.
    async fn list_storage_migration_items(
        &self,
        migration_id: &str,
        status: Option<DriveStorageMigrationItemStatus>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DriveStorageMigrationItem>, DriveServiceError>;

    /// Count of items of a run in each state, as `(pending, copied, failed)`.
    async fn count_storage_migration_items(
        &self,
        migration_id: &str,
    ) -> Result<[i64; 3], DriveServiceError>;

    /// Record one copy attempt and advance the run counters atomically.
    ///
    /// Atomicity is the point: a crash between "item marked copied" and "run
    /// counter incremented" would leave the run permanently under-reporting and
    /// the operator unsure whether the bytes are safe. Both writes therefore
    /// commit together, and the run's `bytes_copied` only grows on success.
    async fn record_storage_migration_item_outcome(
        &self,
        migration_id: &str,
        item_id: &str,
        outcome: &DriveStorageMigrationItemOutcome,
        updated_by: &str,
    ) -> Result<(), DriveServiceError>;

    /// Move a run to a new status, stamping `started_at`/`finished_at` as
    /// appropriate.
    ///
    /// Guarded by `expected_status` so two operators driving the same run
    /// cannot both advance it: the second caller loses the race and gets
    /// `false` instead of silently overwriting the winner's transition.
    async fn transition_storage_migration(
        &self,
        migration_id: &str,
        expected_status: DriveStorageMigrationStatus,
        next_status: DriveStorageMigrationStatus,
        failure_message: Option<&str>,
        updated_by: &str,
    ) -> Result<bool, DriveServiceError>;

    /// Re-point every `dr_drive_storage_object` row queued in a run at the
    /// target provider.
    ///
    /// Only called once every item is `copied` and verified. Doing it as one
    /// statement keeps the switch atomic: there is no window where half a
    /// tenant reads from the old provider and half from the new one.
    async fn apply_storage_migration_repoint(
        &self,
        migration_id: &str,
        target_provider_id: &str,
        target_bucket: &str,
        updated_by: &str,
    ) -> Result<i64, DriveServiceError>;
}
