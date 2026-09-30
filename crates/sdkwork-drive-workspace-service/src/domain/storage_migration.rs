use crate::DriveServiceError;

/// Lifecycle of a cross-provider storage migration.
///
/// The states are ordered by how far the run has committed, not by wall-clock
/// time, because that is what the operator-facing invariants depend on:
///
/// - `Pending` and `Running` have copied bytes but re-pointed **nothing**, so a
///   run abandoned at either state leaves every object reading from its
///   original provider. That is what makes the operation safe: the worst case
///   is duplicate bytes in the target bucket, never a broken read.
/// - `Succeeded` means every object was copied, verified, and the storage-object
///   rows were re-pointed — the switch already happened.
/// - `Failed` stops the copy but keeps what already landed, so a retry resumes
///   instead of restarting.
/// - `Cancelled` is terminal and explicit, so an operator can see the
///   difference between "we gave up" and "the process died".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveStorageMigrationStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl DriveStorageMigrationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Parse the persisted wire form.
    ///
    /// Unknown values are an error rather than a default: silently coercing a
    /// corrupt status to `Pending` would make a broken row look resumable.
    pub fn parse(value: &str) -> Result<Self, DriveServiceError> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(DriveServiceError::Internal(format!(
                "unknown storage migration status {other:?}"
            ))),
        }
    }

    /// Whether the run may still copy more objects.
    ///
    /// `Failed` is resumable on purpose — a transient upstream 500 should not
    /// force the operator to re-declare a run that already moved most of the
    /// data. Terminal states are the two that represent a *decision*.
    pub const fn is_resumable(self) -> bool {
        matches!(self, Self::Pending | Self::Running | Self::Failed)
    }

    /// Whether the run has finished and can no longer be driven forward.
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Cancelled)
    }
}

/// Per-object copy state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveStorageMigrationItemStatus {
    Pending,
    Copied,
    Failed,
}

impl DriveStorageMigrationItemStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Copied => "copied",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DriveServiceError> {
        match value {
            "pending" => Ok(Self::Pending),
            "copied" => Ok(Self::Copied),
            "failed" => Ok(Self::Failed),
            other => Err(DriveServiceError::Internal(format!(
                "unknown storage migration item status {other:?}"
            ))),
        }
    }
}

/// A cross-provider storage migration run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveStorageMigration {
    pub id: String,
    pub tenant_id: String,
    pub source_provider_id: String,
    pub target_provider_id: String,
    pub name: String,
    pub status: DriveStorageMigrationStatus,
    pub objects_total: i64,
    pub objects_copied: i64,
    pub objects_failed: i64,
    pub bytes_copied: i64,
    pub target_bucket: Option<String>,
    pub apply_binding_switch: bool,
    pub failure_message: Option<String>,
    pub created_by: String,
    pub updated_by: String,
}

impl DriveStorageMigration {
    /// Remaining objects for a resumable run.
    ///
    /// Derived rather than stored: `objects_total` is a snapshot taken when the
    /// run was created, and a concurrent upload can add objects to the source
    /// provider after that. The outstanding work is therefore what the item
    /// rows say is still `pending`, which is why progress is persisted per item.
    pub fn outstanding(&self) -> i64 {
        (self.objects_total - self.objects_copied - self.objects_failed).max(0)
    }

    pub fn progress_ratio(&self) -> f64 {
        if self.objects_total <= 0 {
            return 0.0;
        }
        (self.objects_copied + self.objects_failed) as f64 / self.objects_total as f64
    }
}

/// One object queued for a migration run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveStorageMigrationItem {
    pub id: String,
    pub migration_id: String,
    pub tenant_id: String,
    pub storage_object_id: String,
    /// Provider the item is copied *from*. Carried on the item rather than read
    /// from the run so a retry after the run header changed still copies from
    /// the place the queue was built against.
    pub source_provider_id: String,
    pub source_bucket: String,
    pub source_object_key: String,
    /// Provider the item is copied *to*.
    pub target_provider_id: String,
    pub target_bucket: String,
    pub target_object_key: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub status: DriveStorageMigrationItemStatus,
    pub verified_checksum_sha256_hex: Option<String>,
    pub failure_message: Option<String>,
}

/// Fields required to open a new migration run.
#[derive(Debug, Clone)]
pub struct NewDriveStorageMigration {
    pub id: String,
    pub tenant_id: String,
    pub source_provider_id: String,
    pub target_provider_id: String,
    pub name: String,
    pub target_bucket: Option<String>,
    pub apply_binding_switch: bool,
    pub created_by: String,
}

/// A single object as queued for copying.
#[derive(Debug, Clone)]
pub struct NewDriveStorageMigrationItem {
    pub id: String,
    pub storage_object_id: String,
    pub source_bucket: String,
    pub source_object_key: String,
    pub target_bucket: String,
    pub target_object_key: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
}

/// A source object as read from the object registry when a queue is built.
///
/// A projection rather than `DriveStorageObject` because a migration queue does
/// not need the node/version identity — only what is required to copy the bytes
/// and verify them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveStorageMigrationSourceObject {
    pub id: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
}

/// Outcome of one copy attempt, used to advance the item and the run counters.
#[derive(Debug, Clone)]
pub struct DriveStorageMigrationItemOutcome {
    pub copied: bool,
    pub bytes: i64,
    pub verified_checksum_sha256_hex: Option<String>,
    pub failure_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_round_trips_through_its_wire_form() {
        for status in [
            DriveStorageMigrationStatus::Pending,
            DriveStorageMigrationStatus::Running,
            DriveStorageMigrationStatus::Succeeded,
            DriveStorageMigrationStatus::Failed,
            DriveStorageMigrationStatus::Cancelled,
        ] {
            assert_eq!(
                DriveStorageMigrationStatus::parse(status.as_str()).expect("round trip"),
                status
            );
        }
    }

    #[test]
    fn unknown_status_is_rejected_rather_than_defaulted() {
        assert!(DriveStorageMigrationStatus::parse("halfway").is_err());
        assert!(DriveStorageMigrationStatus::parse("").is_err());
    }

    #[test]
    fn only_decided_states_are_terminal() {
        assert!(DriveStorageMigrationStatus::Succeeded.is_terminal());
        assert!(DriveStorageMigrationStatus::Cancelled.is_terminal());
        assert!(!DriveStorageMigrationStatus::Running.is_terminal());
        assert!(!DriveStorageMigrationStatus::Failed.is_terminal());
    }

    #[test]
    fn a_failed_run_stays_resumable() {
        assert!(DriveStorageMigrationStatus::Failed.is_resumable());
        assert!(DriveStorageMigrationStatus::Pending.is_resumable());
        assert!(!DriveStorageMigrationStatus::Succeeded.is_resumable());
        assert!(!DriveStorageMigrationStatus::Cancelled.is_resumable());
    }

    #[test]
    fn outstanding_never_goes_negative_when_counters_overrun() {
        let mut run = sample_migration();
        run.objects_total = 3;
        run.objects_copied = 3;
        run.objects_failed = 0;
        assert_eq!(run.outstanding(), 0);

        // A concurrent re-count that lowered the total must not produce a
        // negative queue depth that a worker would read as "unbounded work".
        run.objects_total = 2;
        run.objects_copied = 5;
        assert_eq!(run.outstanding(), 0);
    }

    #[test]
    fn progress_ratio_is_zero_for_an_empty_run() {
        let mut run = sample_migration();
        run.objects_total = 0;
        assert_eq!(run.progress_ratio(), 0.0);
    }

    #[test]
    fn progress_ratio_counts_failures_as_resolved_work() {
        let mut run = sample_migration();
        run.objects_total = 4;
        run.objects_copied = 3;
        run.objects_failed = 1;
        assert!((run.progress_ratio() - 1.0).abs() < f64::EPSILON);
    }

    fn sample_migration() -> DriveStorageMigration {
        DriveStorageMigration {
            id: "mig-1".to_string(),
            tenant_id: "tenant-1".to_string(),
            source_provider_id: "provider-a".to_string(),
            target_provider_id: "provider-b".to_string(),
            name: "retire minio".to_string(),
            status: DriveStorageMigrationStatus::Pending,
            objects_total: 0,
            objects_copied: 0,
            objects_failed: 0,
            bytes_copied: 0,
            target_bucket: None,
            apply_binding_switch: false,
            failure_message: None,
            created_by: "operator-1".to_string(),
            updated_by: "operator-1".to_string(),
        }
    }
}
