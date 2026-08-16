/// Domain-facing columns from `dr_drive_upload_item`.
/// Excludes timestamps that are not needed by upload projections.
pub const DRIVE_UPLOAD_ITEM_SELECT_COLUMNS: &str = "\
id, task_id, tenant_id, organization_id, user_id, \
actor_type, actor_id, app_id, app_resource_type, app_resource_id, \
scene, source, upload_profile_code, file_fingerprint, \
space_id, node_id, upload_session_id, storage_provider_id, storage_upload_id, \
original_file_name, file_extension, content_type, content_type_group, detected_content_type, \
content_length, checksum_sha256_hex, chunk_size_bytes, total_parts, uploaded_parts_count, \
uploaded_bytes, status, retention_mode, retention_expires_at_epoch_ms, \
cleanup_action, hard_delete_after_epoch_ms, cleanup_status, post_process_status";

/// Same columns as [`DRIVE_UPLOAD_ITEM_SELECT_COLUMNS`] with the `ui.` table alias.
pub const DRIVE_UPLOAD_ITEM_UI_SELECT_COLUMNS: &str = "\
ui.id, ui.task_id, ui.tenant_id, ui.organization_id, ui.user_id, \
ui.actor_type, ui.actor_id, ui.app_id, ui.app_resource_type, ui.app_resource_id, \
ui.scene, ui.source, ui.upload_profile_code, ui.file_fingerprint, \
ui.space_id, ui.node_id, ui.upload_session_id, ui.storage_provider_id, ui.storage_upload_id, \
ui.original_file_name, ui.file_extension, ui.content_type, ui.content_type_group, ui.detected_content_type, \
ui.content_length, ui.checksum_sha256_hex, ui.chunk_size_bytes, ui.total_parts, ui.uploaded_parts_count, \
ui.uploaded_bytes, ui.status, ui.retention_mode, ui.retention_expires_at_epoch_ms, \
ui.cleanup_action, ui.hard_delete_after_epoch_ms, ui.cleanup_status, ui.post_process_status";

/// Domain-facing columns from `dr_drive_upload_part`.
pub const DRIVE_UPLOAD_PART_SELECT_COLUMNS: &str = "\
id, tenant_id, upload_item_id, upload_session_id, part_no, \
offset_bytes, size_bytes, etag, checksum_sha256_hex, \
status, retry_count, uploaded_at_epoch_ms";
