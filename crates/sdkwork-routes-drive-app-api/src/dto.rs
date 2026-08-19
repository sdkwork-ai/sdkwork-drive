use sdkwork_drive_workspace_service::domain::uploader::{DriveUploadItem, DriveUploadPart};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListSandboxesQuery {
    pub page: Option<i64>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxCapabilitiesResponse {
    pub browse: bool,
    pub create_directory: bool,
    pub select_directory: bool,
    pub read_file: bool,
    pub create_file: bool,
    pub write_file: bool,
    pub move_entry: bool,
    pub delete_entry: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxVolumeResponse {
    pub id: String,
    pub display_name: String,
    pub root_entry_id: String,
    pub effective_access: String,
    pub lifecycle_status: String,
    pub capabilities: SandboxCapabilitiesResponse,
    pub revision: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListSandboxEntriesQuery {
    #[serde(default, rename = "parent_path")]
    pub parent_path: Option<String>,
    pub cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSandboxDirectoryRequest {
    pub parent_path: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadSandboxFileQuery {
    #[serde(rename = "logical_path")]
    pub logical_path: String,
    pub encoding: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSandboxFileRequest {
    pub parent_path: String,
    pub name: String,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateSandboxFileContentRequest {
    pub logical_path: String,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MoveSandboxEntryRequest {
    pub logical_path: String,
    pub destination_parent_path: String,
    pub destination_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PurgeSandboxEntryRequest {
    pub logical_path: String,
    pub recursive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxEntryResponse {
    pub id: String,
    pub sandbox_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub logical_path: String,
    pub revision: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxFileContentResponse {
    pub entry: SandboxEntryResponse,
    pub encoding: String,
    pub content: String,
    pub size_bytes: String,
    pub checksum_sha256: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxMutationCommandResponse {
    pub accepted: bool,
    pub resource_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSpaceRequest {
    pub id: String,
    pub owner_subject_type: String,
    pub owner_subject_id: String,
    pub display_name: String,
    pub space_type: String,
    pub presentation_icon: Option<String>,
    pub presentation_color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpaceResponse {
    pub id: String,
    pub tenant_id: String,
    pub owner_subject_type: String,
    pub owner_subject_id: String,
    pub display_name: String,
    pub space_type: String,
    pub presentation_icon: Option<String>,
    pub presentation_color: Option<String>,
    pub description: Option<String>,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateSpaceRequest {
    pub display_name: Option<String>,
    pub presentation_icon: Option<String>,
    pub presentation_color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateUploadSessionRequest {
    pub session_id: String,
    pub space_id: String,
    pub node_id: String,
    pub bucket: Option<String>,
    #[serde(rename = "objectKey")]
    pub object_key: Option<String>,
    pub idempotency_key: String,
    pub expires_at_epoch_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUploadSessionResponse {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub bucket: String,
    pub object_key: String,
    pub idempotency_key: String,
    pub storage_provider_id: String,
    pub storage_upload_id: String,
    pub state: String,
    pub expires_at_epoch_ms: i64,
    pub version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDownloadGrantRequest {
    pub requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDownloadUrlRequest {
    pub node_id: String,
    pub requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDownloadUrlResponse {
    pub download_url: String,
    pub signed_source_url: String,
    pub expires_at_epoch_ms: i64,
    pub method: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateDownloadPackageRequest {
    pub node_ids: Vec<String>,
    pub package_name: Option<String>,
    pub requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum FlexibleI64 {
    Number(i64),
    String(StrictI64String),
}

impl FlexibleI64 {
    pub fn into_i64(self) -> i64 {
        match self {
            FlexibleI64::Number(value) => value,
            FlexibleI64::String(value) => value.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StrictI64String(i64);

impl<'de> Deserialize<'de> for StrictI64String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let trimmed = value.trim();
        if trimmed != value || trimmed.is_empty() {
            return Err(serde::de::Error::custom(
                "expected an integer string without surrounding whitespace",
            ));
        }
        let digits = trimmed
            .strip_prefix('-')
            .filter(|remaining| !remaining.is_empty())
            .unwrap_or(trimmed);
        if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(serde::de::Error::custom("expected an integer string"));
        }
        trimmed
            .parse::<i64>()
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploaderRetentionRequest {
    pub mode: String,
    pub ttl_seconds: Option<FlexibleI64>,
    pub cleanup_action: Option<String>,
    pub hard_delete_after_seconds: Option<FlexibleI64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrepareUploaderUploadRequest {
    pub id: String,
    pub task_id: String,
    pub app_resource_type: String,
    pub app_resource_id: String,
    pub scene: Option<String>,
    pub source: Option<String>,
    pub upload_profile_code: Option<String>,
    pub file_fingerprint: String,
    pub original_file_name: String,
    pub content_type: String,
    pub content_length: FlexibleI64,
    pub chunk_size_bytes: FlexibleI64,
    pub space_id: Option<String>,
    pub parent_node_id: Option<String>,
    pub share_token: Option<String>,
    pub retention: Option<UploaderRetentionRequest>,
    pub now_epoch_ms: Option<FlexibleI64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploaderUploadItemResponse {
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareUploaderUploadResponse {
    pub upload_item: UploaderUploadItemResponse,
    pub upload_session: UploadSessionMutationResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkUploaderPartUploadedRequest {
    pub upload_session_id: String,
    pub offset_bytes: FlexibleI64,
    pub size_bytes: FlexibleI64,
    pub etag: String,
    pub checksum_sha256_hex: Option<String>,
    pub uploaded_at_epoch_ms: Option<FlexibleI64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploaderUploadPartResponse {
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadPackageResponse {
    pub id: String,
    pub tenant_id: String,
    pub package_name: String,
    pub state: String,
    pub storage_provider_id: String,
    pub bucket: String,
    pub archive_object_key: String,
    pub content_type: String,
    pub file_count: i64,
    pub total_bytes: i64,
    pub archive_size_bytes: i64,
    pub expires_at_epoch_ms: i64,
    pub download_url: String,
    pub signed_source_url: String,
    pub method: String,
    pub items: Vec<DownloadPackageItemResponse>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadPackageItemResponse {
    pub node_id: String,
    pub node_name: String,
    pub archive_path: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEntryResponse {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub uncompressed_size_bytes: i64,
    pub compressed_size_bytes: i64,
    pub content_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtractArchiveEntriesRequest {
    pub entry_paths: Option<Vec<String>>,
    pub target_parent_node_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractArchiveEntriesResponse {
    pub items: Vec<DriveNodeResponse>,
    pub extracted_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSpacesQuery {
    pub owner_subject_type: Option<String>,
    pub owner_subject_id: Option<String>,
    pub space_type: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateWebsiteRootRequest {
    pub root_key: String,
    pub display_name: String,
    pub source_root: WebsiteRootSelectorRequest,
    pub content_mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebsiteRootSelectorRequest {
    SpaceRoot,
    Folder {
        #[serde(rename = "folderNodeId")]
        folder_node_id: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListWebsiteRootsQuery {
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsiteRootResponse {
    pub uuid: String,
    pub space_id: String,
    pub root_key: String,
    pub display_name: String,
    pub source_root_mode: String,
    pub selected_folder_node_id: Option<String>,
    pub content_mode: String,
    pub active_node_id: String,
    pub active_generation: String,
    pub root_status: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateWebsiteSyncRequest {
    pub expected_root_version: String,
    pub expected_generation: String,
    pub manifest_sha256: String,
    pub manifest_file_count: String,
    pub manifest_total_bytes: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebsiteSyncVersionRequest {
    pub expected_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActivateWebsiteGenerationRequest {
    pub expected_root_version: String,
    pub expected_generation: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsiteSyncResponse {
    pub id: String,
    pub website_root_uuid: String,
    pub space_id: String,
    pub expected_root_version: String,
    pub expected_generation: String,
    pub staging_node_id: String,
    pub manifest_sha256: String,
    pub manifest_file_count: String,
    pub manifest_total_bytes: String,
    pub uploaded_file_count: String,
    pub uploaded_total_bytes: String,
    pub status: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_summary: Option<String>,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsiteSyncActivationResponse {
    pub sync: WebsiteSyncResponse,
    pub website_root: WebsiteRootResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsiteGenerationResponse {
    pub generation: String,
    pub root_node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_sha256: Option<String>,
    pub file_count: String,
    pub total_bytes: String,
    pub status: String,
    pub activated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsiteGenerationActivationResponse {
    pub source_generation: WebsiteGenerationResponse,
    pub website_root: WebsiteRootResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListNodesQuery {
    pub parent_node_id: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateFolderRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub space_id: String,
    pub parent_node_id: Option<String>,
    pub node_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateFileRequest {
    pub id: String,
    pub space_id: String,
    pub parent_node_id: Option<String>,
    pub node_name: String,
    pub upload_session_id: String,
    pub idempotency_key: String,
    pub expires_at_epoch_ms: i64,
    pub bucket: Option<String>,
    #[serde(rename = "objectKey")]
    pub object_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateShortcutRequest {
    pub id: String,
    pub space_id: String,
    pub parent_node_id: Option<String>,
    pub node_name: String,
    pub target_node_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileResponse {
    pub node: DriveNodeResponse,
    pub upload_session: CreateUploadSessionResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateNodeRequest {
    pub node_name: Option<String>,
    pub parent_node_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MoveNodeRequest {
    pub target_parent_node_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CopyNodeRequest {
    pub id: String,
    pub target_space_id: Option<String>,
    pub target_parent_node_id: Option<String>,
    pub node_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NodeCommandRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmptyTrashRequest {
    pub space_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmptyTrashResponse {
    pub deleted_count: i64,
    pub skipped_count: i64,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeViewQuery {
    pub space_id: Option<String>,
    pub parent_node_id: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubjectNodeViewQuery {
    pub space_id: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FavoriteNodeRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FavoriteNodeQuery {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NodeCapabilitiesQuery {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteNodeResponse {
    pub favorited: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckFavoriteNodesRequest {
    pub node_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteNodeCheckItem {
    pub node_id: String,
    pub favorited: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMoveDestinationsQuery {
    /// Comma-separated node IDs to exclude from results and subtree traversal.
    pub exclude_node_ids: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveNodeResponse {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub space_type: String,
    pub parent_node_id: Option<String>,
    pub shortcut_target_node_id: Option<String>,
    pub node_type: String,
    pub node_name: String,
    pub scene: Option<String>,
    pub source: Option<String>,
    pub content_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_extension: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_color: Option<String>,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

pub fn is_false_bool(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePathResponse {
    pub items: Vec<DriveNodeResponse>,
    pub path_segments: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeCapabilitiesResponse {
    pub tenant_id: String,
    pub node_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub role: String,
    pub source: String,
    pub permission_id: Option<String>,
    pub inherited: bool,
    pub inherited_from_node_id: Option<String>,
    pub can_read: bool,
    pub can_comment: bool,
    pub can_write: bool,
    pub can_download: bool,
    pub can_copy: bool,
    pub can_move: bool,
    pub can_trash: bool,
    pub can_restore: bool,
    pub can_delete: bool,
    pub can_share: bool,
    pub can_manage_permissions: bool,
    pub can_manage_versions: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePropertyListQuery {
    pub visibility: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyNodeListQuery {
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetNodePropertyRequest {
    pub value: String,
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteNodePropertyQuery {
    pub visibility: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePropertyResponse {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub property_key: String,
    pub property_value: String,
    pub visibility: String,
    pub lifecycle_status: String,
    pub version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeLabelListQuery {
    pub label_key: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyNodeLabelRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoveNodeLabelQuery {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelSummaryResponse {
    pub id: String,
    pub tenant_id: String,
    pub label_key: String,
    pub display_name: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub lifecycle_status: String,
    pub version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeLabelResponse {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub label_id: String,
    pub lifecycle_status: String,
    pub version: i64,
    pub label: LabelSummaryResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchChannelListQuery {
    pub resource_type: Option<String>,
    pub lifecycle_status: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateWatchChannelRequest {
    pub id: String,
    pub space_id: Option<String>,
    pub address: String,
    pub token: Option<String>,
    pub channel_type: Option<String>,
    pub expiration_epoch_ms: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StopWatchChannelRequest {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveWatchChannelResponse {
    pub id: String,
    pub tenant_id: String,
    pub space_id: Option<String>,
    pub node_id: Option<String>,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub channel_type: String,
    pub address: String,
    pub expiration_epoch_ms: i64,
    pub lifecycle_status: String,
    pub version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopWatchChannelResponse {
    pub stopped: bool,
    pub channel: DriveWatchChannelResponse,
}

pub struct InsertWatchChannel<'a> {
    pub id: &'a str,
    pub tenant_id: &'a str,
    pub space_id: Option<&'a str>,
    pub node_id: Option<&'a str>,
    pub resource_type: &'a str,
    pub resource_id: Option<&'a str>,
    pub channel_type: &'a str,
    pub address: &'a str,
    pub token_hash: Option<String>,
    pub expiration_epoch_ms: i64,
    pub operator_id: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NodeMutationQuery {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDownloadUrlQuery {
    pub requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileVersionResponse {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_object_id: Option<String>,
    pub version_no: i64,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub lifecycle_status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatePermissionRequest {
    pub id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdatePermissionRequest {
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionResponse {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub role: String,
    pub inherited: bool,
    pub lifecycle_status: String,
    pub version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePermissionResponse {
    pub id: String,
    pub tenant_id: String,
    pub target_node_id: String,
    pub node_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub role: String,
    pub inherited: bool,
    pub inherited_from_node_id: Option<String>,
    pub lifecycle_status: String,
    pub version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateShareLinkRequest {
    pub id: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub access_code: Option<String>,
    pub role: Option<String>,
    pub expires_at_epoch_ms: Option<i64>,
    pub download_limit: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareLinkResponse {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub role: String,
    pub expires_at_epoch_ms: Option<i64>,
    pub download_limit: Option<i64>,
    pub download_count: i64,
    pub access_code_required: bool,
    pub lifecycle_status: String,
    pub version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShareLinkResponse {
    #[serde(flatten)]
    pub link: ShareLinkResponse,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimShareLinkResponse {
    pub share_link_id: String,
    pub node_id: String,
    pub space_id: String,
    pub role: String,
    pub permission_id: String,
    pub already_claimed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateShareLinkRequest {
    pub role: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_patch")]
    pub expires_at_epoch_ms: OptionalI64Patch,
    #[serde(default, deserialize_with = "deserialize_optional_i64_patch")]
    pub download_limit: OptionalI64Patch,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCommentRequest {
    pub id: String,
    pub content: String,
    pub anchor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateCommentRequest {
    pub content: Option<String>,
    pub anchor: Option<String>,
    pub resolved: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentResponse {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub content: String,
    pub anchor: Option<String>,
    pub resolved: bool,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCommentReplyRequest {
    pub id: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateCommentReplyRequest {
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentReplyResponse {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub comment_id: String,
    pub content: String,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum OptionalI64Patch {
    #[default]
    Missing,
    Null,
    Value(i64),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub q: Option<String>,
    pub space_id: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangesQuery {
    pub space_id: Option<String>,
    pub cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPageTokenQuery {
    pub space_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageQuery {
    #[serde(rename = "page_size")]
    pub page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub page_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaSummaryResponse {
    pub tenant_id: String,
    pub used_bytes: i64,
    pub object_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_bytes: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignUploadPartRequest {
    pub upload_id: Option<String>,
    pub requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignedUploadPartResponse {
    pub upload_url: String,
    pub expires_at_epoch_ms: i64,
    pub method: String,
    pub headers: BTreeMap<String, String>,
    pub part_no: u16,
    pub upload_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedUploadPartRequest {
    pub part_no: u16,
    pub etag: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteUploadSessionRequest {
    pub upload_id: Option<String>,
    pub content_type: String,
    pub content_length: FlexibleI64,
    pub checksum_sha256_hex: String,
    pub parts: Vec<CompletedUploadPartRequest>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadSessionMutationResponse {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub bucket: String,
    pub object_key: String,
    pub state: String,
    pub storage_provider_id: String,
    pub storage_upload_id: String,
    pub expires_at_epoch_ms: i64,
    pub version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPageTokenResponse {
    pub start_page_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeResponse {
    pub sequence_no: i64,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: Option<String>,
    pub event_type: String,
    pub actor_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct UploadSessionRecord {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub bucket: String,
    pub object_key: String,
    pub idempotency_key: String,
    pub storage_provider_id: String,
    pub storage_upload_id: String,
    pub state: String,
    pub expires_at_epoch_ms: i64,
    pub version: i64,
}

#[derive(Debug, Clone)]
pub struct StorageTarget {
    pub provider_id: String,
    pub bucket: String,
    pub object_key: String,
}

#[derive(Debug, Clone)]
pub struct DefaultStorageProviderTarget {
    pub provider_id: String,
    pub bucket: String,
    pub storage_root_prefix: String,
}

#[derive(Debug, Clone)]
pub struct CreatedStorageMultipartUpload {
    pub upload_id: String,
}

#[derive(Debug, Clone, Copy)]
pub struct PageRequest {
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct ShareLinkRecord {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub role: String,
    pub expires_at_epoch_ms: Option<i64>,
    pub download_limit: Option<i64>,
    pub download_count: i64,
    pub access_code_hash: Option<String>,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_by: String,
}

#[derive(Debug, Clone)]
pub struct CommentRecord {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub content: String,
    pub anchor: Option<String>,
    pub resolved: bool,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct CommentReplyRecord {
    pub id: String,
    pub tenant_id: String,
    pub node_id: String,
    pub comment_id: String,
    pub content: String,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadPackageFileItem {
    pub node_id: String,
    pub node_name: String,
    pub archive_path: String,
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
}

#[derive(Debug, Clone)]
pub struct DownloadPackageRecordView {
    pub id: String,
    pub tenant_id: String,
    pub package_name: String,
    pub state: String,
    pub storage_provider_id: String,
    pub bucket: String,
    pub archive_object_key: String,
    pub content_type: String,
    pub file_count: i64,
    pub total_bytes: i64,
    pub archive_size_bytes: i64,
    pub expires_at_epoch_ms: i64,
    pub items: Vec<DownloadPackageFileItem>,
}

pub struct InsertDownloadPackageRecord<'a> {
    pub id: &'a str,
    pub tenant_id: &'a str,
    pub package_name: &'a str,
    pub state: &'a str,
    pub storage_provider_id: &'a str,
    pub bucket: &'a str,
    pub archive_object_key: &'a str,
    pub file_count: i64,
    pub total_bytes: i64,
    pub archive_size_bytes: i64,
    pub requested_node_ids_json: &'a str,
    pub item_manifest_json: &'a str,
    pub expires_at_epoch_ms: i64,
    pub operator_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct ActiveStorageObjectRef {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub content_length: i64,
}

#[derive(Debug, Clone)]
pub struct CompletedStorageObjectInsertPlan {
    pub id: String,
    pub version_no: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeArchivePath {
    pub path: String,
    pub segments: Vec<String>,
}

impl From<UploadSessionRecord> for UploadSessionMutationResponse {
    fn from(value: UploadSessionRecord) -> Self {
        Self {
            id: value.id,
            tenant_id: value.tenant_id,
            space_id: value.space_id,
            node_id: value.node_id,
            bucket: value.bucket,
            object_key: value.object_key,
            storage_provider_id: value.storage_provider_id,
            storage_upload_id: value.storage_upload_id,
            state: value.state,
            expires_at_epoch_ms: value.expires_at_epoch_ms,
            version: value.version,
        }
    }
}

impl From<UploadSessionRecord> for CreateUploadSessionResponse {
    fn from(value: UploadSessionRecord) -> Self {
        Self {
            id: value.id,
            tenant_id: value.tenant_id,
            space_id: value.space_id,
            node_id: value.node_id,
            bucket: value.bucket,
            object_key: value.object_key,
            idempotency_key: value.idempotency_key,
            storage_provider_id: value.storage_provider_id,
            storage_upload_id: value.storage_upload_id,
            state: value.state,
            expires_at_epoch_ms: value.expires_at_epoch_ms,
            version: value.version,
        }
    }
}

impl From<DriveUploadItem> for UploaderUploadItemResponse {
    fn from(value: DriveUploadItem) -> Self {
        Self {
            id: value.id,
            task_id: value.task_id,
            tenant_id: value.tenant_id,
            organization_id: value.organization_id,
            user_id: value.user_id,
            actor_type: value.actor_type,
            actor_id: value.actor_id,
            app_id: value.app_id,
            app_resource_type: value.app_resource_type,
            app_resource_id: value.app_resource_id,
            scene: value.scene,
            source: value.source,
            upload_profile_code: value.upload_profile_code,
            file_fingerprint: value.file_fingerprint,
            space_id: value.space_id,
            node_id: value.node_id,
            upload_session_id: value.upload_session_id,
            storage_provider_id: value.storage_provider_id,
            storage_upload_id: value.storage_upload_id,
            original_file_name: value.original_file_name,
            file_extension: value.file_extension,
            content_type: value.content_type,
            content_type_group: value.content_type_group,
            detected_content_type: value.detected_content_type,
            content_length: value.content_length,
            checksum_sha256_hex: value.checksum_sha256_hex,
            chunk_size_bytes: value.chunk_size_bytes,
            total_parts: value.total_parts,
            uploaded_parts_count: value.uploaded_parts_count,
            uploaded_bytes: value.uploaded_bytes,
            status: value.status,
            retention_mode: value.retention_mode,
            retention_expires_at_epoch_ms: value.retention_expires_at_epoch_ms,
            cleanup_action: value.cleanup_action,
            hard_delete_after_epoch_ms: value.hard_delete_after_epoch_ms,
            cleanup_status: value.cleanup_status,
            post_process_status: value.post_process_status,
        }
    }
}

impl From<DriveUploadPart> for UploaderUploadPartResponse {
    fn from(value: DriveUploadPart) -> Self {
        Self {
            id: value.id,
            tenant_id: value.tenant_id,
            upload_item_id: value.upload_item_id,
            upload_session_id: value.upload_session_id,
            part_no: value.part_no,
            offset_bytes: value.offset_bytes,
            size_bytes: value.size_bytes,
            etag: value.etag,
            checksum_sha256_hex: value.checksum_sha256_hex,
            status: value.status,
            retry_count: value.retry_count,
            uploaded_at_epoch_ms: value.uploaded_at_epoch_ms,
        }
    }
}

impl From<ShareLinkRecord> for ShareLinkResponse {
    fn from(value: ShareLinkRecord) -> Self {
        Self {
            id: value.id,
            tenant_id: value.tenant_id,
            node_id: value.node_id,
            role: value.role,
            expires_at_epoch_ms: value.expires_at_epoch_ms,
            download_limit: value.download_limit,
            download_count: value.download_count,
            access_code_required: value
                .access_code_hash
                .as_deref()
                .map(str::trim)
                .is_some_and(|hash| !hash.is_empty()),
            lifecycle_status: value.lifecycle_status,
            version: value.version,
        }
    }
}

impl From<CommentRecord> for CommentResponse {
    fn from(value: CommentRecord) -> Self {
        Self {
            id: value.id,
            tenant_id: value.tenant_id,
            node_id: value.node_id,
            content: value.content,
            anchor: value.anchor,
            resolved: value.resolved,
            lifecycle_status: value.lifecycle_status,
            version: value.version,
            created_by: value.created_by,
            updated_by: value.updated_by,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<CommentReplyRecord> for CommentReplyResponse {
    fn from(value: CommentReplyRecord) -> Self {
        Self {
            id: value.id,
            tenant_id: value.tenant_id,
            node_id: value.node_id,
            comment_id: value.comment_id,
            content: value.content,
            lifecycle_status: value.lifecycle_status,
            version: value.version,
            created_by: value.created_by,
            updated_by: value.updated_by,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

pub fn apply_optional_i64_patch(
    value: OptionalI64Patch,
    current: Option<i64>,
) -> Option<i64> {
    match value {
        OptionalI64Patch::Missing => current,
        OptionalI64Patch::Null => None,
        OptionalI64Patch::Value(value) => Some(value),
    }
}

fn deserialize_optional_i64_patch<'de, D>(deserializer: D) -> Result<OptionalI64Patch, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<i64>::deserialize(deserializer)?;
    Ok(match value {
        Some(value) => OptionalI64Patch::Value(value),
        None => OptionalI64Patch::Null,
    })
}


#[cfg(test)]
mod auth_projection_request_tests {
    use super::PrepareUploaderUploadRequest;

    #[test]
    fn prepare_uploader_upload_request_uses_only_business_fields() {
        serde_json::from_str::<PrepareUploaderUploadRequest>(
            r#"{
                "id":"upload-item-001",
                "taskId":"task-001",
                "appResourceType":"desktop-file-browser",
                "appResourceId":"root",
                "fileFingerprint":"fp-001",
                "originalFileName":"a.pdf",
                "contentType":"application/pdf",
                "contentLength":5,
                "chunkSizeBytes":5242880
            }"#,
        )
        .expect("prepare request should deserialize without auth projection fields");
    }

    #[test]
    fn prepare_uploader_upload_request_rejects_auth_projection_fields() {
        let error = serde_json::from_str::<PrepareUploaderUploadRequest>(
            r#"{
                "id":"upload-item-001",
                "taskId":"task-001",
                "appResourceType":"desktop-file-browser",
                "appResourceId":"root",
                "fileFingerprint":"fp-001",
                "originalFileName":"a.pdf",
                "contentType":"application/pdf",
                "contentLength":5,
                "chunkSizeBytes":5242880,
                "operatorId":"user-002"
            }"#,
        )
        .expect_err("auth projection fields must not be accepted by app request DTOs");

        assert!(error.to_string().contains("operatorId"));
    }
}

#[cfg(test)]
mod node_view_query_tests {
    use super::NodeViewQuery;

    #[test]
    fn node_view_query_accepts_parent_node_id_for_trash_drilldown() {
        let query: NodeViewQuery = serde_json::from_value(serde_json::json!({
            "spaceId": "space-001",
            "parentNodeId": "folder-trashed-001",
            "page_size": 50,
            "cursor": "100",
        }))
        .expect("trash list query should deserialize parentNodeId");

        assert_eq!(query.space_id.as_deref(), Some("space-001"));
        assert_eq!(query.parent_node_id.as_deref(), Some("folder-trashed-001"));
        assert_eq!(query.page_size, Some(50));
        assert_eq!(query.page_token.as_deref(), Some("100"));
    }
}
