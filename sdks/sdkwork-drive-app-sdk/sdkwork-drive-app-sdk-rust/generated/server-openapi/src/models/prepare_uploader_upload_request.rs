use serde::{Deserialize, Serialize};

use crate::models::{UploaderRetentionRequest};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PrepareUploaderUploadRequest {
    pub id: String,

    #[serde(rename = "taskId")]
    pub task_id: String,

    #[serde(rename = "appResourceType")]
    pub app_resource_type: String,

    #[serde(rename = "appResourceId")]
    pub app_resource_id: String,

    #[serde(rename = "uploadProfileCode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_profile_code: Option<String>,

    #[serde(rename = "fileFingerprint")]
    pub file_fingerprint: String,

    #[serde(rename = "originalFileName")]
    pub original_file_name: String,

    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "contentLength")]
    pub content_length: i64,

    #[serde(rename = "chunkSizeBytes")]
    pub chunk_size_bytes: i64,

    #[serde(rename = "spaceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,

    #[serde(rename = "parentNodeId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_node_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<UploaderRetentionRequest>,

    #[serde(rename = "nowEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub now_epoch_ms: Option<i64>,

    /// Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,

    /// Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Optional Drive share token authorizing anonymous or external uploads into an explicit target folder. The raw token is accepted only on prepare requests and is never returned.
    #[serde(rename = "shareToken")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_token: Option<String>,
}
