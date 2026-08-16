use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UploaderUploadItem {
    pub id: String,

    #[serde(rename = "taskId")]
    pub task_id: String,

    #[serde(rename = "tenantId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    #[serde(rename = "organizationId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    #[serde(rename = "userId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    #[serde(rename = "actorType")]
    pub actor_type: String,

    #[serde(rename = "actorId")]
    pub actor_id: String,

    #[serde(rename = "appId")]
    pub app_id: String,

    #[serde(rename = "appResourceType")]
    pub app_resource_type: String,

    #[serde(rename = "appResourceId")]
    pub app_resource_id: String,

    #[serde(rename = "uploadProfileCode")]
    pub upload_profile_code: String,

    #[serde(rename = "fileFingerprint")]
    pub file_fingerprint: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "uploadSessionId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_session_id: Option<String>,

    #[serde(rename = "storageProviderId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_provider_id: Option<String>,

    #[serde(rename = "storageUploadId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_upload_id: Option<String>,

    #[serde(rename = "originalFileName")]
    pub original_file_name: String,

    #[serde(rename = "fileExtension")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_extension: Option<String>,

    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "contentTypeGroup")]
    pub content_type_group: String,

    #[serde(rename = "detectedContentType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detected_content_type: Option<String>,

    #[serde(rename = "contentLength")]
    pub content_length: i64,

    #[serde(rename = "checksumSha256Hex")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_sha256_hex: Option<String>,

    #[serde(rename = "chunkSizeBytes")]
    pub chunk_size_bytes: i64,

    #[serde(rename = "totalParts")]
    pub total_parts: i64,

    #[serde(rename = "uploadedPartsCount")]
    pub uploaded_parts_count: i64,

    #[serde(rename = "uploadedBytes")]
    pub uploaded_bytes: i64,

    pub status: String,

    #[serde(rename = "retentionMode")]
    pub retention_mode: String,

    #[serde(rename = "retentionExpiresAtEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_expires_at_epoch_ms: Option<i64>,

    #[serde(rename = "cleanupAction")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cleanup_action: Option<String>,

    #[serde(rename = "hardDeleteAfterEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hard_delete_after_epoch_ms: Option<i64>,

    #[serde(rename = "cleanupStatus")]
    pub cleanup_status: String,

    #[serde(rename = "postProcessStatus")]
    pub post_process_status: String,

    /// Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,

    /// Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}
