use sdkwork_drive_workspace_service::domain::uploader::{DriveUploadItem, DriveUploadPart};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ListSandboxesQuery {
    pub(crate) page: Option<i64>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SandboxCapabilitiesResponse {
    pub(crate) browse: bool,
    pub(crate) create_directory: bool,
    pub(crate) select_directory: bool,
    pub(crate) read_file: bool,
    pub(crate) create_file: bool,
    pub(crate) write_file: bool,
    pub(crate) move_entry: bool,
    pub(crate) delete_entry: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SandboxVolumeResponse {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) root_entry_id: String,
    pub(crate) effective_access: String,
    pub(crate) lifecycle_status: String,
    pub(crate) capabilities: SandboxCapabilitiesResponse,
    pub(crate) revision: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ListSandboxEntriesQuery {
    #[serde(default, rename = "parent_path")]
    pub(crate) parent_path: Option<String>,
    pub(crate) cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateSandboxDirectoryRequest {
    pub(crate) parent_path: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReadSandboxFileQuery {
    #[serde(rename = "logical_path")]
    pub(crate) logical_path: String,
    pub(crate) encoding: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateSandboxFileRequest {
    pub(crate) parent_path: String,
    pub(crate) name: String,
    pub(crate) content: String,
    pub(crate) encoding: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateSandboxFileContentRequest {
    pub(crate) logical_path: String,
    pub(crate) content: String,
    pub(crate) encoding: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct MoveSandboxEntryRequest {
    pub(crate) logical_path: String,
    pub(crate) destination_parent_path: String,
    pub(crate) destination_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PurgeSandboxEntryRequest {
    pub(crate) logical_path: String,
    pub(crate) recursive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SandboxEntryResponse {
    pub(crate) id: String,
    pub(crate) sandbox_id: String,
    pub(crate) parent_id: Option<String>,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) logical_path: String,
    pub(crate) revision: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SandboxFileContentResponse {
    pub(crate) entry: SandboxEntryResponse,
    pub(crate) encoding: String,
    pub(crate) content: String,
    pub(crate) size_bytes: String,
    pub(crate) checksum_sha256: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SandboxMutationCommandResponse {
    pub(crate) accepted: bool,
    pub(crate) resource_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateSpaceRequest {
    pub(crate) id: String,
    pub(crate) owner_subject_type: String,
    pub(crate) owner_subject_id: String,
    pub(crate) display_name: String,
    pub(crate) space_type: String,
    pub(crate) presentation_icon: Option<String>,
    pub(crate) presentation_color: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateSpaceResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) owner_subject_type: String,
    pub(crate) owner_subject_id: String,
    pub(crate) display_name: String,
    pub(crate) space_type: String,
    pub(crate) presentation_icon: Option<String>,
    pub(crate) presentation_color: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) created_by: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateSpaceRequest {
    pub(crate) display_name: Option<String>,
    pub(crate) presentation_icon: Option<String>,
    pub(crate) presentation_color: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateUploadSessionRequest {
    pub(crate) session_id: String,
    pub(crate) space_id: String,
    pub(crate) node_id: String,
    pub(crate) bucket: Option<String>,
    #[serde(rename = "objectKey")]
    pub(crate) object_key: Option<String>,
    pub(crate) idempotency_key: String,
    pub(crate) expires_at_epoch_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateUploadSessionResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) space_id: String,
    pub(crate) node_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) idempotency_key: String,
    pub(crate) storage_provider_id: String,
    pub(crate) storage_upload_id: String,
    pub(crate) state: String,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateDownloadGrantRequest {
    pub(crate) requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateDownloadUrlRequest {
    pub(crate) node_id: String,
    pub(crate) requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateDownloadUrlResponse {
    pub(crate) download_url: String,
    pub(crate) signed_source_url: String,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) method: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateDownloadPackageRequest {
    pub(crate) node_ids: Vec<String>,
    pub(crate) package_name: Option<String>,
    pub(crate) requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub(crate) enum FlexibleI64 {
    Number(i64),
    String(StrictI64String),
}

impl FlexibleI64 {
    pub(crate) fn into_i64(self) -> i64 {
        match self {
            FlexibleI64::Number(value) => value,
            FlexibleI64::String(value) => value.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StrictI64String(i64);

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
pub(crate) struct UploaderRetentionRequest {
    pub(crate) mode: String,
    pub(crate) ttl_seconds: Option<FlexibleI64>,
    pub(crate) cleanup_action: Option<String>,
    pub(crate) hard_delete_after_seconds: Option<FlexibleI64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PrepareUploaderUploadRequest {
    pub(crate) id: String,
    pub(crate) task_id: String,
    pub(crate) app_resource_type: String,
    pub(crate) app_resource_id: String,
    pub(crate) scene: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) upload_profile_code: Option<String>,
    pub(crate) file_fingerprint: String,
    pub(crate) original_file_name: String,
    pub(crate) content_type: String,
    pub(crate) content_length: FlexibleI64,
    pub(crate) chunk_size_bytes: FlexibleI64,
    pub(crate) space_id: Option<String>,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) share_token: Option<String>,
    pub(crate) retention: Option<UploaderRetentionRequest>,
    pub(crate) now_epoch_ms: Option<FlexibleI64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UploaderUploadItemResponse {
    pub(crate) id: String,
    pub(crate) task_id: String,
    pub(crate) tenant_id: String,
    pub(crate) organization_id: Option<String>,
    pub(crate) user_id: Option<String>,
    pub(crate) actor_type: String,
    pub(crate) actor_id: String,
    pub(crate) app_id: String,
    pub(crate) app_resource_type: String,
    pub(crate) app_resource_id: String,
    pub(crate) scene: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) upload_profile_code: String,
    pub(crate) file_fingerprint: String,
    pub(crate) space_id: String,
    pub(crate) node_id: String,
    pub(crate) upload_session_id: Option<String>,
    pub(crate) storage_provider_id: Option<String>,
    pub(crate) storage_upload_id: Option<String>,
    pub(crate) original_file_name: String,
    pub(crate) file_extension: Option<String>,
    pub(crate) content_type: String,
    pub(crate) content_type_group: String,
    pub(crate) detected_content_type: Option<String>,
    pub(crate) content_length: i64,
    pub(crate) checksum_sha256_hex: Option<String>,
    pub(crate) chunk_size_bytes: i64,
    pub(crate) total_parts: i64,
    pub(crate) uploaded_parts_count: i64,
    pub(crate) uploaded_bytes: i64,
    pub(crate) status: String,
    pub(crate) retention_mode: String,
    pub(crate) retention_expires_at_epoch_ms: Option<i64>,
    pub(crate) cleanup_action: Option<String>,
    pub(crate) hard_delete_after_epoch_ms: Option<i64>,
    pub(crate) cleanup_status: String,
    pub(crate) post_process_status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrepareUploaderUploadResponse {
    pub(crate) upload_item: UploaderUploadItemResponse,
    pub(crate) upload_session: UploadSessionMutationResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MarkUploaderPartUploadedRequest {
    pub(crate) upload_session_id: String,
    pub(crate) offset_bytes: FlexibleI64,
    pub(crate) size_bytes: FlexibleI64,
    pub(crate) etag: String,
    pub(crate) checksum_sha256_hex: Option<String>,
    pub(crate) uploaded_at_epoch_ms: Option<FlexibleI64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UploaderUploadPartResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) upload_item_id: String,
    pub(crate) upload_session_id: String,
    pub(crate) part_no: i64,
    pub(crate) offset_bytes: i64,
    pub(crate) size_bytes: i64,
    pub(crate) etag: String,
    pub(crate) checksum_sha256_hex: Option<String>,
    pub(crate) status: String,
    pub(crate) retry_count: i64,
    pub(crate) uploaded_at_epoch_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DownloadPackageResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) package_name: String,
    pub(crate) state: String,
    pub(crate) storage_provider_id: String,
    pub(crate) bucket: String,
    pub(crate) archive_object_key: String,
    pub(crate) content_type: String,
    pub(crate) file_count: i64,
    pub(crate) total_bytes: i64,
    pub(crate) archive_size_bytes: i64,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) download_url: String,
    pub(crate) signed_source_url: String,
    pub(crate) method: String,
    pub(crate) items: Vec<DownloadPackageItemResponse>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DownloadPackageItemResponse {
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) archive_path: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) content_type: String,
    pub(crate) content_length: i64,
    pub(crate) checksum_sha256_hex: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchiveEntryResponse {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) is_directory: bool,
    pub(crate) uncompressed_size_bytes: i64,
    pub(crate) compressed_size_bytes: i64,
    pub(crate) content_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ExtractArchiveEntriesRequest {
    pub(crate) entry_paths: Option<Vec<String>>,
    pub(crate) target_parent_node_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExtractArchiveEntriesResponse {
    pub(crate) items: Vec<DriveNodeResponse>,
    pub(crate) extracted_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListSpacesQuery {
    pub(crate) owner_subject_type: Option<String>,
    pub(crate) owner_subject_id: Option<String>,
    pub(crate) space_type: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateWebsiteRootRequest {
    pub(crate) root_key: String,
    pub(crate) display_name: String,
    pub(crate) source_root: WebsiteRootSelectorRequest,
    pub(crate) content_mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum WebsiteRootSelectorRequest {
    SpaceRoot,
    Folder {
        #[serde(rename = "folderNodeId")]
        folder_node_id: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ListWebsiteRootsQuery {
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    pub(crate) cursor: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebsiteRootResponse {
    pub(crate) uuid: String,
    pub(crate) space_id: String,
    pub(crate) root_key: String,
    pub(crate) display_name: String,
    pub(crate) source_root_mode: String,
    pub(crate) selected_folder_node_id: Option<String>,
    pub(crate) content_mode: String,
    pub(crate) active_node_id: String,
    pub(crate) active_generation: String,
    pub(crate) root_status: String,
    pub(crate) version: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateWebsiteSyncRequest {
    pub(crate) expected_root_version: String,
    pub(crate) expected_generation: String,
    pub(crate) manifest_sha256: String,
    pub(crate) manifest_file_count: String,
    pub(crate) manifest_total_bytes: String,
    pub(crate) expires_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct WebsiteSyncVersionRequest {
    pub(crate) expected_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ActivateWebsiteGenerationRequest {
    pub(crate) expected_root_version: String,
    pub(crate) expected_generation: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebsiteSyncResponse {
    pub(crate) id: String,
    pub(crate) website_root_uuid: String,
    pub(crate) space_id: String,
    pub(crate) expected_root_version: String,
    pub(crate) expected_generation: String,
    pub(crate) staging_node_id: String,
    pub(crate) manifest_sha256: String,
    pub(crate) manifest_file_count: String,
    pub(crate) manifest_total_bytes: String,
    pub(crate) uploaded_file_count: String,
    pub(crate) uploaded_total_bytes: String,
    pub(crate) status: String,
    pub(crate) expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) validated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) activated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_summary: Option<String>,
    pub(crate) version: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebsiteSyncActivationResponse {
    pub(crate) sync: WebsiteSyncResponse,
    pub(crate) website_root: WebsiteRootResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebsiteGenerationResponse {
    pub(crate) generation: String,
    pub(crate) root_node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) manifest_sha256: Option<String>,
    pub(crate) file_count: String,
    pub(crate) total_bytes: String,
    pub(crate) status: String,
    pub(crate) activated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebsiteGenerationActivationResponse {
    pub(crate) source_generation: WebsiteGenerationResponse,
    pub(crate) website_root: WebsiteRootResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListNodesQuery {
    pub(crate) parent_node_id: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
    pub(crate) sort_by: Option<String>,
    pub(crate) sort_order: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateFolderRequest {
    #[serde(default)]
    pub(crate) id: Option<String>,
    pub(crate) space_id: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) node_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateFileRequest {
    pub(crate) id: String,
    pub(crate) space_id: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) node_name: String,
    pub(crate) upload_session_id: String,
    pub(crate) idempotency_key: String,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) bucket: Option<String>,
    #[serde(rename = "objectKey")]
    pub(crate) object_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateShortcutRequest {
    pub(crate) id: String,
    pub(crate) space_id: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) node_name: String,
    pub(crate) target_node_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateFileResponse {
    pub(crate) node: DriveNodeResponse,
    pub(crate) upload_session: CreateUploadSessionResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateNodeRequest {
    pub(crate) node_name: Option<String>,
    pub(crate) parent_node_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct MoveNodeRequest {
    pub(crate) target_parent_node_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CopyNodeRequest {
    pub(crate) id: String,
    pub(crate) target_space_id: Option<String>,
    pub(crate) target_parent_node_id: Option<String>,
    pub(crate) node_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NodeCommandRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct EmptyTrashRequest {
    pub(crate) space_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EmptyTrashResponse {
    pub(crate) deleted_count: i64,
    pub(crate) skipped_count: i64,
    pub(crate) has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeViewQuery {
    pub(crate) space_id: Option<String>,
    pub(crate) parent_node_id: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
    pub(crate) sort_by: Option<String>,
    pub(crate) sort_order: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SubjectNodeViewQuery {
    pub(crate) space_id: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
    pub(crate) sort_by: Option<String>,
    pub(crate) sort_order: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FavoriteNodeRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FavoriteNodeQuery {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NodeCapabilitiesQuery {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FavoriteNodeResponse {
    pub(crate) favorited: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CheckFavoriteNodesRequest {
    pub(crate) node_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FavoriteNodeCheckItem {
    pub(crate) node_id: String,
    pub(crate) favorited: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListMoveDestinationsQuery {
    /// Comma-separated node IDs to exclude from results and subtree traversal.
    pub(crate) exclude_node_ids: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DriveNodeResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) space_id: String,
    pub(crate) space_type: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) shortcut_target_node_id: Option<String>,
    pub(crate) node_type: String,
    pub(crate) node_name: String,
    pub(crate) scene: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) content_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) file_extension: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content_type_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) folder_color: Option<String>,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

pub(crate) fn is_false_bool(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodePathResponse {
    pub(crate) items: Vec<DriveNodeResponse>,
    pub(crate) path_segments: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeCapabilitiesResponse {
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) subject_type: String,
    pub(crate) subject_id: String,
    pub(crate) role: String,
    pub(crate) source: String,
    pub(crate) permission_id: Option<String>,
    pub(crate) inherited: bool,
    pub(crate) inherited_from_node_id: Option<String>,
    pub(crate) can_read: bool,
    pub(crate) can_comment: bool,
    pub(crate) can_write: bool,
    pub(crate) can_download: bool,
    pub(crate) can_copy: bool,
    pub(crate) can_move: bool,
    pub(crate) can_trash: bool,
    pub(crate) can_restore: bool,
    pub(crate) can_delete: bool,
    pub(crate) can_share: bool,
    pub(crate) can_manage_permissions: bool,
    pub(crate) can_manage_versions: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodePropertyListQuery {
    pub(crate) visibility: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PropertyNodeListQuery {
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetNodePropertyRequest {
    pub(crate) value: String,
    pub(crate) visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DeleteNodePropertyQuery {
    pub(crate) visibility: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodePropertyResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) property_key: String,
    pub(crate) property_value: String,
    pub(crate) visibility: String,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeLabelListQuery {
    pub(crate) label_key: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ApplyNodeLabelRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RemoveNodeLabelQuery {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LabelSummaryResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) label_key: String,
    pub(crate) display_name: String,
    pub(crate) color: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeLabelResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) label_id: String,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) label: LabelSummaryResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WatchChannelListQuery {
    pub(crate) resource_type: Option<String>,
    pub(crate) lifecycle_status: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateWatchChannelRequest {
    pub(crate) id: String,
    pub(crate) space_id: Option<String>,
    pub(crate) address: String,
    pub(crate) token: Option<String>,
    pub(crate) channel_type: Option<String>,
    pub(crate) expiration_epoch_ms: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StopWatchChannelRequest {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DriveWatchChannelResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) space_id: Option<String>,
    pub(crate) node_id: Option<String>,
    pub(crate) resource_type: String,
    pub(crate) resource_id: Option<String>,
    pub(crate) channel_type: String,
    pub(crate) address: String,
    pub(crate) expiration_epoch_ms: i64,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StopWatchChannelResponse {
    pub(crate) stopped: bool,
    pub(crate) channel: DriveWatchChannelResponse,
}

pub(crate) struct InsertWatchChannel<'a> {
    pub(crate) id: &'a str,
    pub(crate) tenant_id: &'a str,
    pub(crate) space_id: Option<&'a str>,
    pub(crate) node_id: Option<&'a str>,
    pub(crate) resource_type: &'a str,
    pub(crate) resource_id: Option<&'a str>,
    pub(crate) channel_type: &'a str,
    pub(crate) address: &'a str,
    pub(crate) token_hash: Option<String>,
    pub(crate) expiration_epoch_ms: i64,
    pub(crate) operator_id: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NodeMutationQuery {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeDownloadUrlQuery {
    pub(crate) requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileVersionResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) storage_object_id: Option<String>,
    pub(crate) version_no: i64,
    pub(crate) content_type: String,
    pub(crate) content_length: i64,
    pub(crate) checksum_sha256_hex: String,
    pub(crate) lifecycle_status: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreatePermissionRequest {
    pub(crate) id: String,
    pub(crate) subject_type: String,
    pub(crate) subject_id: String,
    pub(crate) role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdatePermissionRequest {
    pub(crate) role: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PermissionResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) subject_type: String,
    pub(crate) subject_id: String,
    pub(crate) role: String,
    pub(crate) inherited: bool,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EffectivePermissionResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) target_node_id: String,
    pub(crate) node_id: String,
    pub(crate) subject_type: String,
    pub(crate) subject_id: String,
    pub(crate) role: String,
    pub(crate) inherited: bool,
    pub(crate) inherited_from_node_id: Option<String>,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateShareLinkRequest {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) token: String,
    #[serde(default)]
    pub(crate) access_code: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) expires_at_epoch_ms: Option<i64>,
    pub(crate) download_limit: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShareLinkResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) role: String,
    pub(crate) expires_at_epoch_ms: Option<i64>,
    pub(crate) download_limit: Option<i64>,
    pub(crate) download_count: i64,
    pub(crate) access_code_required: bool,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateShareLinkResponse {
    #[serde(flatten)]
    pub(crate) link: ShareLinkResponse,
    pub(crate) token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) access_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaimShareLinkResponse {
    pub(crate) share_link_id: String,
    pub(crate) node_id: String,
    pub(crate) space_id: String,
    pub(crate) role: String,
    pub(crate) permission_id: String,
    pub(crate) already_claimed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateShareLinkRequest {
    pub(crate) role: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_i64_patch")]
    pub(crate) expires_at_epoch_ms: OptionalI64Patch,
    #[serde(default, deserialize_with = "deserialize_optional_i64_patch")]
    pub(crate) download_limit: OptionalI64Patch,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateCommentRequest {
    pub(crate) id: String,
    pub(crate) content: String,
    pub(crate) anchor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateCommentRequest {
    pub(crate) content: Option<String>,
    pub(crate) anchor: Option<String>,
    pub(crate) resolved: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommentResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) content: String,
    pub(crate) anchor: Option<String>,
    pub(crate) resolved: bool,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) created_by: String,
    pub(crate) updated_by: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateCommentReplyRequest {
    pub(crate) id: String,
    pub(crate) content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateCommentReplyRequest {
    pub(crate) content: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommentReplyResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) comment_id: String,
    pub(crate) content: String,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) created_by: String,
    pub(crate) updated_by: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum OptionalI64Patch {
    #[default]
    Missing,
    Null,
    Value(i64),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchQuery {
    pub(crate) q: Option<String>,
    pub(crate) space_id: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChangesQuery {
    pub(crate) space_id: Option<String>,
    pub(crate) cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartPageTokenQuery {
    pub(crate) space_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PageQuery {
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuotaSummaryResponse {
    pub(crate) tenant_id: String,
    pub(crate) used_bytes: i64,
    pub(crate) object_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) quota_bytes: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PresignUploadPartRequest {
    pub(crate) upload_id: Option<String>,
    pub(crate) requested_ttl_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PresignedUploadPartResponse {
    pub(crate) upload_url: String,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) method: String,
    pub(crate) headers: BTreeMap<String, String>,
    pub(crate) part_no: u16,
    pub(crate) upload_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CompletedUploadPartRequest {
    pub(crate) part_no: u16,
    pub(crate) etag: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CompleteUploadSessionRequest {
    pub(crate) upload_id: Option<String>,
    pub(crate) content_type: String,
    pub(crate) content_length: FlexibleI64,
    pub(crate) checksum_sha256_hex: String,
    pub(crate) parts: Vec<CompletedUploadPartRequest>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UploadSessionMutationResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) space_id: String,
    pub(crate) node_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) state: String,
    pub(crate) storage_provider_id: String,
    pub(crate) storage_upload_id: String,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) version: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartPageTokenResponse {
    pub(crate) start_page_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChangeResponse {
    pub(crate) sequence_no: i64,
    pub(crate) tenant_id: String,
    pub(crate) space_id: String,
    pub(crate) node_id: Option<String>,
    pub(crate) event_type: String,
    pub(crate) actor_id: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Clone)]
pub(crate) struct UploadSessionRecord {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) space_id: String,
    pub(crate) node_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) idempotency_key: String,
    pub(crate) storage_provider_id: String,
    pub(crate) storage_upload_id: String,
    pub(crate) state: String,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) version: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct StorageTarget {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
}

#[derive(Debug, Clone)]
pub(crate) struct DefaultStorageProviderTarget {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) storage_root_prefix: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CreatedStorageMultipartUpload {
    pub(crate) upload_id: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PageRequest {
    pub(crate) limit: i64,
    pub(crate) offset: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct ShareLinkRecord {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) role: String,
    pub(crate) expires_at_epoch_ms: Option<i64>,
    pub(crate) download_limit: Option<i64>,
    pub(crate) download_count: i64,
    pub(crate) access_code_hash: Option<String>,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) created_by: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CommentRecord {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) content: String,
    pub(crate) anchor: Option<String>,
    pub(crate) resolved: bool,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) created_by: String,
    pub(crate) updated_by: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CommentReplyRecord {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) node_id: String,
    pub(crate) comment_id: String,
    pub(crate) content: String,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) created_by: String,
    pub(crate) updated_by: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DownloadPackageFileItem {
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) archive_path: String,
    pub(crate) storage_provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) content_type: String,
    pub(crate) content_length: i64,
    pub(crate) checksum_sha256_hex: String,
}

#[derive(Debug, Clone)]
pub(crate) struct DownloadPackageRecordView {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) package_name: String,
    pub(crate) state: String,
    pub(crate) storage_provider_id: String,
    pub(crate) bucket: String,
    pub(crate) archive_object_key: String,
    pub(crate) content_type: String,
    pub(crate) file_count: i64,
    pub(crate) total_bytes: i64,
    pub(crate) archive_size_bytes: i64,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) items: Vec<DownloadPackageFileItem>,
}

pub(crate) struct InsertDownloadPackageRecord<'a> {
    pub(crate) id: &'a str,
    pub(crate) tenant_id: &'a str,
    pub(crate) package_name: &'a str,
    pub(crate) state: &'a str,
    pub(crate) storage_provider_id: &'a str,
    pub(crate) bucket: &'a str,
    pub(crate) archive_object_key: &'a str,
    pub(crate) file_count: i64,
    pub(crate) total_bytes: i64,
    pub(crate) archive_size_bytes: i64,
    pub(crate) requested_node_ids_json: &'a str,
    pub(crate) item_manifest_json: &'a str,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) operator_id: &'a str,
}

#[derive(Debug, Clone)]
pub(crate) struct ActiveStorageObjectRef {
    pub(crate) storage_provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) content_type: String,
    pub(crate) content_length: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct CompletedStorageObjectInsertPlan {
    pub(crate) id: String,
    pub(crate) version_no: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SafeArchivePath {
    pub(crate) path: String,
    pub(crate) segments: Vec<String>,
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

pub(crate) fn apply_optional_i64_patch(
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

pub(crate) const ASSET_NODE_SELECT_COLUMNS: &str = "\
    id, tenant_id, space_id, space_type, parent_node_id, shortcut_target_node_id, \
    node_type, node_name, scene, source, content_state, file_extension, \
    head_content_type, head_content_type_group, head_content_length, \
    lifecycle_status, version, CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListAssetsQuery {
    pub(crate) cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    pub(crate) kind: Option<String>,
    pub(crate) source_type: Option<String>,
    pub(crate) q: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateAssetRequest {
    pub(crate) drive_node_id: Option<String>,
    pub(crate) virtual_reference: Option<serde_json::Value>,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) scene: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateAssetRequest {
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) scene: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetActionRequest {
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MediaResourceResponse {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) size_bytes: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetItemResponse {
    pub(crate) asset_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    pub(crate) tenant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user_id: Option<String>,
    pub(crate) drive_space_id: String,
    pub(crate) drive_node_id: String,
    pub(crate) drive_uri: String,
    pub(crate) node_type: String,
    pub(crate) asset_kind: String,
    pub(crate) title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) scene: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) visibility: Option<String>,
    pub(crate) lifecycle_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resource_snapshot: Option<MediaResourceResponse>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListAssetCollectionsQuery {
    pub(crate) cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateAssetCollectionRequest {
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) collection_type: Option<String>,
    pub(crate) visibility: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetCollectionResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) organization_id: Option<String>,
    pub(crate) user_id: String,
    pub(crate) title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) collection_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) visibility: Option<String>,
    pub(crate) lifecycle_status: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateAssetCollectionItemRequest {
    pub(crate) asset_id: String,
    pub(crate) sort_order: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetCollectionItemResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) collection_id: String,
    pub(crate) asset_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateAssetRelationRequest {
    pub(crate) related_asset_id: Option<String>,
    pub(crate) relation_type: String,
    pub(crate) source_domain: Option<String>,
    pub(crate) source_resource_type: Option<String>,
    pub(crate) source_resource_id: Option<String>,
    pub(crate) metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetRelationResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) asset_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) related_asset_id: Option<String>,
    pub(crate) relation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_resource_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metadata: Option<serde_json::Value>,
    pub(crate) lifecycle_status: String,
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
