use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteSync {
    pub id: String,

    #[serde(rename = "websiteRootUuid")]
    pub website_root_uuid: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "expectedRootVersion")]
    pub expected_root_version: String,

    #[serde(rename = "expectedGeneration")]
    pub expected_generation: String,

    #[serde(rename = "stagingNodeId")]
    pub staging_node_id: String,

    #[serde(rename = "manifestSha256")]
    pub manifest_sha256: String,

    #[serde(rename = "manifestFileCount")]
    pub manifest_file_count: String,

    #[serde(rename = "manifestTotalBytes")]
    pub manifest_total_bytes: String,

    #[serde(rename = "uploadedFileCount")]
    pub uploaded_file_count: String,

    #[serde(rename = "uploadedTotalBytes")]
    pub uploaded_total_bytes: String,

    pub status: String,

    #[serde(rename = "expiresAt")]
    pub expires_at: String,

    #[serde(rename = "validatedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_at: Option<String>,

    #[serde(rename = "activatedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activated_at: Option<String>,

    #[serde(rename = "completedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,

    #[serde(rename = "errorCode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    #[serde(rename = "errorSummary")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_summary: Option<String>,

    pub version: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
