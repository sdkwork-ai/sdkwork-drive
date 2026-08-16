use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use crate::latency_histogram;

static HTTP_REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static HTTP_REQUEST_ERRORS_TOTAL: AtomicU64 = AtomicU64::new(0);
static HTTP_RATE_LIMITED_TOTAL: AtomicU64 = AtomicU64::new(0);
static DOMAIN_OUTBOX_PENDING_TOTAL: AtomicU64 = AtomicU64::new(0);
static DOMAIN_OUTBOX_DELIVERED_TOTAL: AtomicU64 = AtomicU64::new(0);
static UPLOAD_CONTENT_POLICY_EVALUATED_TOTAL: AtomicU64 = AtomicU64::new(0);
static UPLOADER_PART_UPLOADED_TOTAL: AtomicU64 = AtomicU64::new(0);
static MULTIPART_COMPENSATION_ATTEMPTED_TOTAL: AtomicU64 = AtomicU64::new(0);
static MULTIPART_COMPENSATION_SUCCEEDED_TOTAL: AtomicU64 = AtomicU64::new(0);
static MULTIPART_COMPENSATION_FAILED_TOTAL: AtomicU64 = AtomicU64::new(0);
static WEBSITE_SYNC_TRANSACTION_RETRIES_TOTAL: AtomicU64 = AtomicU64::new(0);
static WEBSITE_SYNC_TRANSACTION_RETRY_EXHAUSTED_TOTAL: AtomicU64 = AtomicU64::new(0);
static WEBSITE_GENERATIONS_EXPIRED_TOTAL: AtomicU64 = AtomicU64::new(0);
static WEBSITE_SYNCS_EXPIRED_TOTAL: AtomicU64 = AtomicU64::new(0);
static WEBSITE_CLEANUP_CANDIDATES_COMPLETED_TOTAL: AtomicU64 = AtomicU64::new(0);
static WEBSITE_CLEANUP_OBJECTS_DELETED_TOTAL: AtomicU64 = AtomicU64::new(0);
static WEBSITE_CLEANUP_NODES_DELETED_TOTAL: AtomicU64 = AtomicU64::new(0);
static HEALTH_SERVING: AtomicU64 = AtomicU64::new(1);
static HTTP_REQUEST_ROUTE_LABELS: LazyLock<Mutex<HashMap<String, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn record_http_request_route_labels(method: &str, route: &str, status: u16, api_surface: &str) {
    let key = format!(
        "method=\"{method}\",route=\"{route}\",status=\"{status}\",api_surface=\"{api_surface}\""
    );
    if let Ok(mut labels) = HTTP_REQUEST_ROUTE_LABELS.lock() {
        let entry = labels.entry(key).or_insert(0);
        *entry += 1;
    }
}

pub fn record_uploader_part_uploaded() {
    UPLOADER_PART_UPLOADED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_multipart_compensation_attempted() {
    MULTIPART_COMPENSATION_ATTEMPTED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_multipart_compensation_succeeded() {
    MULTIPART_COMPENSATION_SUCCEEDED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_multipart_compensation_failed() {
    MULTIPART_COMPENSATION_FAILED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_website_sync_transaction_retry() {
    WEBSITE_SYNC_TRANSACTION_RETRIES_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_website_sync_transaction_retry_exhausted() {
    WEBSITE_SYNC_TRANSACTION_RETRY_EXHAUSTED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_website_generations_expired(count: u64) {
    WEBSITE_GENERATIONS_EXPIRED_TOTAL.fetch_add(count, Ordering::Relaxed);
}

pub fn record_website_publishing_cleanup(
    expired_syncs: u64,
    completed_candidates: u64,
    deleted_objects: u64,
    deleted_nodes: u64,
) {
    WEBSITE_SYNCS_EXPIRED_TOTAL.fetch_add(expired_syncs, Ordering::Relaxed);
    WEBSITE_CLEANUP_CANDIDATES_COMPLETED_TOTAL.fetch_add(completed_candidates, Ordering::Relaxed);
    WEBSITE_CLEANUP_OBJECTS_DELETED_TOTAL.fetch_add(deleted_objects, Ordering::Relaxed);
    WEBSITE_CLEANUP_NODES_DELETED_TOTAL.fetch_add(deleted_nodes, Ordering::Relaxed);
}

pub fn record_http_request() {
    HTTP_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_http_request_error() {
    HTTP_REQUEST_ERRORS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_http_request_duration(duration: Duration) {
    latency_histogram::record_duration(duration);
}

pub fn record_http_rate_limited() {
    HTTP_RATE_LIMITED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_outbox_pending() {
    DOMAIN_OUTBOX_PENDING_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_outbox_delivered() {
    DOMAIN_OUTBOX_DELIVERED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_upload_content_policy_evaluated() {
    UPLOAD_CONTENT_POLICY_EVALUATED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn set_health_serving(serving: bool) {
    HEALTH_SERVING.store(if serving { 1 } else { 0 }, Ordering::Relaxed);
}

pub fn render_prometheus(service: &str) -> String {
    let environment = std::env::var("SDKWORK_DRIVE_RUNTIME_PROFILE")
        .unwrap_or_else(|_| "development".to_string());
    let deployment_profile = sdkwork_drive_config::resolve_deployment_profile_label();

    let mut output = format!(
        "# HELP drive_http_requests_total Total HTTP requests handled by Drive routers.\n\
         # TYPE drive_http_requests_total counter\n\
         drive_http_requests_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_http_request_errors_total Total HTTP request errors handled by Drive routers.\n\
         # TYPE drive_http_request_errors_total counter\n\
         drive_http_request_errors_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_http_rate_limited_total Total HTTP requests rejected by in-process rate limiters.\n\
         # TYPE drive_http_rate_limited_total counter\n\
         drive_http_rate_limited_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_domain_outbox_pending_total Domain outbox events accepted for delivery.\n\
         # TYPE drive_domain_outbox_pending_total counter\n\
         drive_domain_outbox_pending_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_domain_outbox_delivered_total Domain outbox events delivered successfully.\n\
         # TYPE drive_domain_outbox_delivered_total counter\n\
         drive_domain_outbox_delivered_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_upload_content_policy_evaluated_total Uploads evaluated by MIME upload content policy.\n\
         # TYPE drive_upload_content_policy_evaluated_total counter\n\
         drive_upload_content_policy_evaluated_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_uploader_part_uploaded_total Uploader multipart parts marked uploaded.\n\
         # TYPE drive_uploader_part_uploaded_total counter\n\
         drive_uploader_part_uploaded_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_multipart_compensation_attempted_total Multipart abort compensations attempted after publication failure.\n\
         # TYPE drive_multipart_compensation_attempted_total counter\n\
         drive_multipart_compensation_attempted_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_multipart_compensation_succeeded_total Multipart abort compensations completed after publication failure.\n\
         # TYPE drive_multipart_compensation_succeeded_total counter\n\
         drive_multipart_compensation_succeeded_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_multipart_compensation_failed_total Multipart abort compensations that failed after publication failure.\n\
         # TYPE drive_multipart_compensation_failed_total counter\n\
         drive_multipart_compensation_failed_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_website_sync_transaction_retries_total WebsiteSync serializable transactions retried after PostgreSQL serialization failures.\n\
         # TYPE drive_website_sync_transaction_retries_total counter\n\
         drive_website_sync_transaction_retries_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_website_sync_transaction_retry_exhausted_total WebsiteSync serializable transactions that exhausted bounded retries.\n\
         # TYPE drive_website_sync_transaction_retry_exhausted_total counter\n\
         drive_website_sync_transaction_retry_exhausted_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_website_generations_expired_total WebsiteRoot generations expired by retained-generation policy.\n\
         # TYPE drive_website_generations_expired_total counter\n\
         drive_website_generations_expired_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_website_syncs_expired_total WebsiteSync rows expired by background publishing maintenance.\n\
         # TYPE drive_website_syncs_expired_total counter\n\
         drive_website_syncs_expired_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_website_cleanup_candidates_completed_total Website publishing cleanup candidates completed.\n\
         # TYPE drive_website_cleanup_candidates_completed_total counter\n\
         drive_website_cleanup_candidates_completed_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_website_cleanup_objects_deleted_total Website publishing storage objects deleted from providers.\n\
         # TYPE drive_website_cleanup_objects_deleted_total counter\n\
         drive_website_cleanup_objects_deleted_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_website_cleanup_nodes_deleted_total Website publishing nodes retired after reference checks.\n\
         # TYPE drive_website_cleanup_nodes_deleted_total counter\n\
         drive_website_cleanup_nodes_deleted_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n\
         # HELP drive_health_status Service health serving status (1=serving, 0=not serving).\n\
         # TYPE drive_health_status gauge\n\
         drive_health_status{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\"}} {}\n",
        HTTP_REQUESTS_TOTAL.load(Ordering::Relaxed),
        HTTP_REQUEST_ERRORS_TOTAL.load(Ordering::Relaxed),
        HTTP_RATE_LIMITED_TOTAL.load(Ordering::Relaxed),
        DOMAIN_OUTBOX_PENDING_TOTAL.load(Ordering::Relaxed),
        DOMAIN_OUTBOX_DELIVERED_TOTAL.load(Ordering::Relaxed),
        UPLOAD_CONTENT_POLICY_EVALUATED_TOTAL.load(Ordering::Relaxed),
        UPLOADER_PART_UPLOADED_TOTAL.load(Ordering::Relaxed),
        MULTIPART_COMPENSATION_ATTEMPTED_TOTAL.load(Ordering::Relaxed),
        MULTIPART_COMPENSATION_SUCCEEDED_TOTAL.load(Ordering::Relaxed),
        MULTIPART_COMPENSATION_FAILED_TOTAL.load(Ordering::Relaxed),
        WEBSITE_SYNC_TRANSACTION_RETRIES_TOTAL.load(Ordering::Relaxed),
        WEBSITE_SYNC_TRANSACTION_RETRY_EXHAUSTED_TOTAL.load(Ordering::Relaxed),
        WEBSITE_GENERATIONS_EXPIRED_TOTAL.load(Ordering::Relaxed),
        WEBSITE_SYNCS_EXPIRED_TOTAL.load(Ordering::Relaxed),
        WEBSITE_CLEANUP_CANDIDATES_COMPLETED_TOTAL.load(Ordering::Relaxed),
        WEBSITE_CLEANUP_OBJECTS_DELETED_TOTAL.load(Ordering::Relaxed),
        WEBSITE_CLEANUP_NODES_DELETED_TOTAL.load(Ordering::Relaxed),
        HEALTH_SERVING.load(Ordering::Relaxed),
    );
    output.push_str(&latency_histogram::render_prometheus_histogram(
        "drive_http_request_duration_seconds",
        service,
        &environment,
        &deployment_profile,
    ));
    if let Ok(labels) = HTTP_REQUEST_ROUTE_LABELS.lock() {
        if !labels.is_empty() {
            output.push_str(
                "# HELP drive_http_requests_by_route_total HTTP requests grouped by route template, method, status, and API surface.\n\
                 # TYPE drive_http_requests_by_route_total counter\n",
            );
            for (labels, count) in labels.iter() {
                output.push_str(&format!(
                    "drive_http_requests_by_route_total{{service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\",{labels}}} {count}\n",
                ));
            }
        }
    }
    output
}
