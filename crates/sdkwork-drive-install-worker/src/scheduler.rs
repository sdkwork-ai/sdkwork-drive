use std::time::Duration;

use tokio::task::JoinHandle;

/// Scheduler configuration.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Interval for upload session cleanup.
    pub upload_cleanup_interval: Duration,
    /// Interval for aborting vendor multipart uploads of expired sessions.
    pub expired_multipart_abort_interval: Duration,
    /// Interval for orphan object cleanup.
    pub orphan_cleanup_interval: Duration,
    /// Interval for expired uploader content moving to the recycle bin / purge.
    pub expired_upload_cleanup_interval: Duration,
    /// Interval for failing upload items abandoned by expired sessions.
    pub abandoned_upload_sweep_interval: Duration,
    /// Interval for physically deleting retired provider objects.
    pub storage_object_purge_interval: Duration,
    /// Interval for deleting expired download-package archives.
    pub download_package_cleanup_interval: Duration,
    /// Interval for quota recalculation.
    pub quota_recalculation_interval: Duration,
    /// Interval for domain outbox dispatch.
    pub domain_outbox_dispatch_interval: Duration,
    /// Interval for WebsiteSync and expired generation cleanup.
    pub website_publishing_cleanup_interval: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            upload_cleanup_interval: Duration::from_secs(3600),
            expired_multipart_abort_interval: Duration::from_secs(1800),
            orphan_cleanup_interval: Duration::from_secs(86400),
            expired_upload_cleanup_interval: Duration::from_secs(900),
            abandoned_upload_sweep_interval: Duration::from_secs(900),
            storage_object_purge_interval: Duration::from_secs(900),
            download_package_cleanup_interval: Duration::from_secs(900),
            quota_recalculation_interval: Duration::from_secs(3600),
            domain_outbox_dispatch_interval: Duration::from_secs(30),
            website_publishing_cleanup_interval: Duration::from_secs(300),
        }
    }
}

/// Background task scheduler.
pub struct Scheduler {
    config: SchedulerConfig,
    handles: Vec<JoinHandle<()>>,
}

impl Scheduler {
    /// Create a new scheduler with the given configuration.
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            handles: Vec::new(),
        }
    }

    /// Start all scheduled tasks.
    pub fn start(&mut self, pool: sqlx::PgPool) {
        let pool_clone = pool.clone();
        let interval = self.config.upload_cleanup_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "upload_session_cleanup",
                    || async move {
                        match crate::maintenance::upload_session_cleanup::cleanup_expired_sessions(
                            &pool_for_task,
                        )
                        .await
                        {
                            Ok(result) => {
                                tracing::info!(
                                    target: "sdkwork.drive",
                                    event = "drive.install_worker.upload_session_cleanup",
                                    expired_sessions = result.expired_sessions,
                                    cleaned_parts = result.cleaned_parts,
                                    retired_uploading_nodes = result.retired_uploading_nodes,
                                    retired_storage_objects = result.retired_storage_objects,
                                    "upload session cleanup completed"
                                );
                            }
                            Err(error) => {
                                tracing::error!("Upload session cleanup failed: {}", error);
                            }
                        }
                    },
                )
                .await;
            }
        }));

        let pool_clone = pool.clone();
        let interval = self.config.expired_multipart_abort_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "expired_multipart_abort",
                    || async move {
                        match crate::maintenance::expired_multipart_abort::abort_expired_multipart_uploads(
                            &pool_for_task,
                            200,
                        )
                        .await
                        {
                            Ok(result) => {
                                tracing::info!(
                                    target: "sdkwork.drive",
                                    event = "drive.install_worker.expired_multipart_abort",
                                    scanned_sessions = result.scanned_sessions,
                                    aborted_uploads = result.aborted_uploads,
                                    failed_uploads = result.failed_uploads,
                                    "expired multipart abort completed"
                                );
                            }
                            Err(error) => {
                                tracing::error!(
                                    event = "drive.install_worker.expired_multipart_abort_failed",
                                    error = %error,
                                    "expired multipart abort failed"
                                );
                            }
                        }
                    },
                )
                .await;
            }
        }));

        let pool_clone = pool.clone();
        let interval = self.config.orphan_cleanup_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "orphan_object_cleanup",
                    || async move {
                        match crate::maintenance::orphan_object_cleanup::cleanup_orphan_objects(
                            &pool_for_task,
                        )
                        .await
                        {
                            Ok(result) => {
                                tracing::info!(
                                    target: "sdkwork.drive",
                                    event = "drive.install_worker.orphan_object_cleanup",
                                    orphaned_nodes = result.orphaned_nodes,
                                    cleaned_objects = result.cleaned_objects,
                                    "orphan object cleanup completed"
                                );
                            }
                            Err(error) => {
                                tracing::error!("Orphan cleanup failed: {}", error);
                            }
                        }
                    },
                )
                .await;
            }
        }));

        let pool_clone = pool.clone();
        let interval = self.config.expired_upload_cleanup_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "expired_upload_content_cleanup",
                    || async move {
                        let service =
                            sdkwork_drive_workspace_service::application::maintenance_service::DriveMaintenanceService::new(
                                sdkwork_drive_workspace_service::infrastructure::sql::maintenance_store::SqlMaintenanceStore::new(
                                    pool_for_task,
                                ),
                            );
                        match service
                            .sweep_expired_upload_content(
                                sdkwork_drive_workspace_service::application::maintenance_service::SweepExpiredUploadContentCommand {
                                    now_epoch_ms: chrono::Utc::now().timestamp_millis(),
                                    dry_run: false,
                                    limit: Some(500),
                                    operator_id: "install-worker".to_string(),
                                    request_id: None,
                                    trace_id: None,
                                },
                            )
                            .await
                        {
                            Ok(result) => {
                                tracing::info!(
                                    target: "sdkwork.drive",
                                    event = "drive.install_worker.expired_upload_content_cleanup",
                                    scanned = result.scanned_count,
                                    affected = result.affected_count,
                                    "expired upload content cleanup completed"
                                );
                            }
                            Err(error) => {
                                tracing::error!(
                                    event = "drive.install_worker.expired_upload_content_cleanup_failed",
                                    error = ?error,
                                    "expired upload content cleanup failed"
                                );
                            }
                        }
                    },
                )
                .await;
            }
        }));

        let pool_clone = pool.clone();
        let interval = self.config.abandoned_upload_sweep_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "abandoned_upload_task_sweep",
                    || async move {
                        let service =
                            sdkwork_drive_workspace_service::application::maintenance_service::DriveMaintenanceService::new(
                                sdkwork_drive_workspace_service::infrastructure::sql::maintenance_store::SqlMaintenanceStore::new(
                                    pool_for_task,
                                ),
                            );
                        match service
                            .sweep_abandoned_upload_tasks(
                                sdkwork_drive_workspace_service::application::maintenance_service::SweepAbandonedUploadTasksCommand {
                                    now_epoch_ms: chrono::Utc::now().timestamp_millis(),
                                    dry_run: false,
                                    limit: Some(500),
                                    operator_id: "install-worker".to_string(),
                                    request_id: None,
                                    trace_id: None,
                                },
                            )
                            .await
                        {
                            Ok(result) => {
                                tracing::info!(
                                    target: "sdkwork.drive",
                                    event = "drive.install_worker.abandoned_upload_task_sweep",
                                    scanned = result.scanned_count,
                                    affected = result.affected_count,
                                    "abandoned upload task sweep completed"
                                );
                            }
                            Err(error) => {
                                tracing::error!(
                                    event = "drive.install_worker.abandoned_upload_task_sweep_failed",
                                    error = ?error,
                                    "abandoned upload task sweep failed"
                                );
                            }
                        }
                    },
                )
                .await;
            }
        }));

        let pool_clone = pool.clone();
        let interval = self.config.storage_object_purge_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "storage_object_purge",
                    || async move {
                        match crate::maintenance::storage_object_purge::purge_deleted_storage_objects(
                            &pool_for_task,
                            500,
                        )
                        .await
                        {
                            Ok(result) => {
                                tracing::info!(
                                    target: "sdkwork.drive",
                                    event = "drive.install_worker.storage_object_purge",
                                    scanned_objects = result.scanned_objects,
                                    purged_objects = result.purged_objects,
                                    failed_objects = result.failed_objects,
                                    "storage object purge completed"
                                );
                            }
                            Err(error) => {
                                tracing::error!(
                                    event = "drive.install_worker.storage_object_purge_failed",
                                    error = %error,
                                    "storage object purge failed"
                                );
                            }
                        }
                    },
                )
                .await;
            }
        }));

        let pool_clone = pool.clone();
        let interval = self.config.download_package_cleanup_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "download_package_cleanup",
                    || async move {
                        match crate::maintenance::expired_download_package_cleanup::cleanup_expired_download_packages(
                            &pool_for_task,
                            200,
                        )
                        .await
                        {
                            Ok(result) => {
                                tracing::info!(
                                    target: "sdkwork.drive",
                                    event = "drive.install_worker.download_package_cleanup",
                                    scanned_packages = result.scanned_packages,
                                    cleaned_packages = result.cleaned_packages,
                                    failed_packages = result.failed_packages,
                                    "expired download package cleanup completed"
                                );
                            }
                            Err(error) => {
                                tracing::error!(
                                    event = "drive.install_worker.download_package_cleanup_failed",
                                    error = %error,
                                    "expired download package cleanup failed"
                                );
                            }
                        }
                    },
                )
                .await;
            }
        }));

        let pool_clone = pool.clone();
        let interval = self.config.quota_recalculation_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "quota_recalculation",
                    || async move {
                        match crate::maintenance::quota_recalculation::recalculate_quotas(
                            &pool_for_task,
                        )
                        .await
                        {
                            Ok(result) => {
                                tracing::info!(
                                    target: "sdkwork.drive",
                                    event = "drive.install_worker.quota_recalculation",
                                    tenants_scanned = result.tenants_scanned,
                                    storage_objects_retired = result.storage_objects_retired,
                                    tenants_over_quota = result.tenants_over_quota,
                                    "quota reconciliation completed"
                                );
                                if result.tenants_over_quota > 0 {
                                    tracing::warn!(
                                        target: "sdkwork.drive",
                                        event = "drive.install_worker.quota_over_limit",
                                        tenants_over_quota = result.tenants_over_quota,
                                        "tenants exceed configured storage quota caps"
                                    );
                                }
                            }
                            Err(error) => {
                                tracing::error!("Quota recalculation failed: {}", error);
                            }
                        }
                    },
                )
                .await;
            }
        }));

        let pool_clone = pool.clone();
        let interval = self.config.domain_outbox_dispatch_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "domain_outbox_dispatch",
                    || async move {
                        match crate::maintenance::domain_outbox_dispatch::dispatch_pending_outbox_events(
                            &pool_for_task,
                        )
                        .await
                        {
                            Ok(result) => {
                                if result.processed > 0 {
                                    tracing::info!(
                                        "Domain outbox dispatch: processed={}, delivered={}, failed={}",
                                        result.processed,
                                        result.delivered,
                                        result.failed
                                    );
                                }
                            }
                            Err(error) => {
                                tracing::error!("Domain outbox dispatch failed: {}", error);
                            }
                        }
                    },
                )
                .await;
            }
        }));

        let pool_clone = pool.clone();
        let interval = self.config.website_publishing_cleanup_interval;
        self.handles.push(tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let pool_for_task = pool_clone.clone();
                crate::maintenance::leader::run_if_maintenance_leader(
                    &pool_clone,
                    "website_publishing_cleanup",
                    || async move {
                        match crate::maintenance::website_publishing_cleanup::
                            cleanup_website_publishing(&pool_for_task, 1_000)
                            .await
                        {
                            Ok(result) => {
                                if result.expired_syncs > 0
                                    || result.completed_candidates > 0
                                    || result.deleted_objects > 0
                                {
                                    tracing::info!(
                                        target: "sdkwork.drive",
                                        event = "drive.install_worker.website_publishing_cleanup",
                                        expired_syncs = result.expired_syncs,
                                        completed_candidates = result.completed_candidates,
                                        deleted_objects = result.deleted_objects,
                                        deleted_nodes = result.deleted_nodes,
                                        "website publishing cleanup completed"
                                    );
                                }
                            }
                            Err(error) => {
                                tracing::error!(
                                    target: "sdkwork.drive",
                                    event = "drive.install_worker.website_publishing_cleanup_failed",
                                    error = %error,
                                    "website publishing cleanup failed"
                                );
                            }
                        }
                    },
                )
                .await;
            }
        }));
    }

    /// Stop all scheduled tasks.
    pub fn stop(&mut self) {
        for handle in self.handles.drain(..) {
            handle.abort();
        }
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.stop();
    }
}
