use std::sync::atomic::AtomicU64;

pub const DEFAULT_DOWNLOAD_PUBLIC_BASE_URL: &str = "http://127.0.0.1:18080/app/v3/api/drive";
/// Default list page size per PAGINATION_SPEC (§3); sourced from sdkwork-utils-rust.
pub const DEFAULT_LIST_PAGE_SIZE: i64 = sdkwork_utils_rust::DEFAULT_LIST_PAGE_SIZE as i64;
/// Maximum list page size per PAGINATION_SPEC (§3); sourced from sdkwork-utils-rust.
pub const MAX_LIST_PAGE_SIZE: i64 = sdkwork_utils_rust::MAX_LIST_PAGE_SIZE as i64;
/// Maximum node IDs accepted by favorites.check per request.
pub const MAX_FAVORITE_CHECK_NODE_IDS: usize = 200;
/// Maximum folder nodes returned by move_destinations.list for one space.
pub const MAX_MOVE_DESTINATION_FOLDERS: usize = 2_000;
/// Maximum nodes collected for lifecycle mutations (delete/retire subtree).
pub const MAX_LIFECYCLE_SUBTREE_NODES: usize = 10_000;
pub const DOWNLOAD_PACKAGE_MAX_FILES: usize = 500;
pub const DOWNLOAD_PACKAGE_MAX_TOTAL_BYTES: i64 = 1_073_741_824;
pub const ARCHIVE_MAX_ENTRIES: usize = 500;
/// Maximum total uncompressed bytes reported by archive metadata for listing.
pub const ARCHIVE_MAX_TOTAL_UNCOMPRESSED_BYTES: i64 = 1_073_741_824;
/// Maximum selected uncompressed bytes handled by the synchronous extract API.
pub const ARCHIVE_EXTRACT_MAX_TOTAL_UNCOMPRESSED_BYTES: i64 = 64 * 1024 * 1024;
/// Maximum single archive entry bytes held in memory during synchronous extract.
pub const ARCHIVE_EXTRACT_MAX_FILE_BYTES: i64 = 16 * 1024 * 1024;
/// Maximum compressed archive bytes loaded into memory before ZIP inspection.
pub const ARCHIVE_MAX_COMPRESSED_BYTES: i64 = 64 * 1024 * 1024;
pub const SDKWORK_SNOWFLAKE_EPOCH_MS: u64 = 1_609_459_200_000;
pub const SDKWORK_DRIVE_WORKER_ID: u64 = 17;
pub static LAST_APP_SNOWFLAKE_ID: AtomicU64 = AtomicU64::new(0);
