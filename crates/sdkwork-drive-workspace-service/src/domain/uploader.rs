#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveUploadItem {
    pub id: String,
    pub task_id: String,
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub user_id: Option<String>,
    pub actor_type: String,
    pub actor_id: String,
    pub app_id: String,
    pub app_resource_type: String,
    pub app_resource_id: String,
    pub scene: Option<String>,
    pub source: Option<String>,
    pub upload_profile_code: String,
    pub file_fingerprint: String,
    pub space_id: String,
    pub node_id: String,
    pub upload_session_id: Option<String>,
    pub storage_provider_id: Option<String>,
    pub storage_upload_id: Option<String>,
    pub object_bucket: Option<String>,
    pub object_key: Option<String>,
    pub original_file_name: String,
    pub file_extension: Option<String>,
    pub content_type: String,
    pub content_type_group: String,
    pub detected_content_type: Option<String>,
    pub content_length: i64,
    pub checksum_sha256_hex: Option<String>,
    pub chunk_size_bytes: i64,
    pub total_parts: i64,
    pub uploaded_parts_count: i64,
    pub uploaded_bytes: i64,
    pub status: String,
    pub retention_mode: String,
    pub retention_expires_at_epoch_ms: Option<i64>,
    pub cleanup_action: Option<String>,
    pub hard_delete_after_epoch_ms: Option<i64>,
    pub cleanup_status: String,
    pub post_process_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveUploadPart {
    pub id: String,
    pub tenant_id: String,
    pub upload_item_id: String,
    pub upload_session_id: String,
    pub part_no: i64,
    pub offset_bytes: i64,
    pub size_bytes: i64,
    pub etag: String,
    pub checksum_sha256_hex: Option<String>,
    pub status: String,
    pub retry_count: i64,
    pub uploaded_at_epoch_ms: Option<i64>,
}

pub fn content_type_group_for(content_type: &str) -> &'static str {
    let normalized = content_type.trim().to_ascii_lowercase();
    if normalized.starts_with("image/") {
        "image"
    } else if normalized.starts_with("video/") {
        "video"
    } else if normalized.starts_with("audio/") {
        "audio"
    } else if normalized.starts_with("text/") {
        "text"
    } else if matches!(
        normalized.as_str(),
        "application/pdf"
            | "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.ms-excel"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-powerpoint"
            | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
    ) {
        "document"
    } else if matches!(
        normalized.as_str(),
        "application/zip"
            | "application/x-zip-compressed"
            | "application/gzip"
            | "application/x-tar"
    ) {
        "archive"
    } else {
        "binary"
    }
}
