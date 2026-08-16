use std::time::Instant;

pub const OBSERVABILITY_TARGET: &str = "sdkwork.drive";

pub mod latency_histogram;
pub mod metrics;
pub mod tracing_setup;

pub use tracing_setup::init_tracing;

pub mod events {
    pub const BACKEND_AUDIT_EVENTS_LIST: &str = "drive.audit_events.list";
    pub const BACKEND_MAINTENANCE_JOBS_LIST: &str = "drive.maintenance.jobs.list";
    pub const BACKEND_MAINTENANCE_OBJECT_SWEEP: &str = "drive.maintenance.object_sweep";
    pub const BACKEND_MAINTENANCE_UPLOAD_SESSION_SWEEP: &str =
        "drive.maintenance.upload_session_sweep";
    pub const BACKEND_MAINTENANCE_EXPIRED_UPLOAD_CONTENT_SWEEP: &str =
        "drive.maintenance.expired_upload_content_sweep";
    pub const BACKEND_MAINTENANCE_ABANDONED_UPLOAD_TASK_SWEEP: &str =
        "drive.maintenance.abandoned_upload_task_sweep";
    pub const APP_SPACES_LIST: &str = "drive.app.spaces.list";
    pub const APP_SPACES_CREATE: &str = "drive.app.spaces.create";
    pub const APP_SPACES_GET: &str = "drive.app.spaces.get";
    pub const APP_SPACES_UPDATE: &str = "drive.app.spaces.update";
    pub const APP_SPACES_DELETE: &str = "drive.app.spaces.delete";
    pub const APP_UPLOAD_SESSIONS_CREATE: &str = "drive.app.upload_sessions.create";
    pub const APP_UPLOADER_UPLOADS_PREPARE: &str = "drive.app.uploader.uploads.prepare";
    pub const APP_UPLOADER_PART_MARK_UPLOADED: &str =
        "drive.app.uploader.uploads.parts.mark_uploaded";
    pub const APP_DOWNLOAD_URLS_CREATE: &str = "drive.app.download_urls.create";
    pub const APP_DOWNLOAD_TOKENS_RESOLVE: &str = "drive.app.download_tokens.resolve";
}

pub mod error_kinds {
    pub const VALIDATION: &str = "validation";
    pub const CONFLICT: &str = "conflict";
    pub const NOT_FOUND: &str = "not_found";
    pub const PERMISSION_DENIED: &str = "permission_denied";
    pub const INTERNAL: &str = "internal";
}

pub fn start_timer() -> Instant {
    Instant::now()
}

pub fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

pub fn has_value<T>(value: &Option<T>) -> bool {
    value.is_some()
}

pub fn crate_id() -> &'static str {
    "sdkwork-drive-observability"
}

#[macro_export]
macro_rules! observe_route {
    (
        event = $event:expr,
        result = $result:expr,
        latency_ms = $latency_ms:expr
        $(, $field:ident = $value:expr )* $(,)?
    ) => {
        tracing::info!(
            target: $crate::OBSERVABILITY_TARGET,
            event = $event,
            result = $result,
            latency_ms = $latency_ms
            $(, $field = $value)*
        );
    };
}
