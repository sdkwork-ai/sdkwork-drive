use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveResourceResolution {
    #[serde(rename = "scopeType")]
    pub scope_type: String,

    #[serde(rename = "scopeUuid")]
    pub scope_uuid: String,

    #[serde(rename = "scopeGeneration")]
    pub scope_generation: String,

    #[serde(rename = "normalizedRelativePath")]
    pub normalized_relative_path: String,

    #[serde(rename = "resourceType")]
    pub resource_type: String,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "logicalNodeVersionId")]
    pub logical_node_version_id: String,

    #[serde(rename = "versionNo")]
    pub version_no: String,

    #[serde(rename = "checksumSha256Hex")]
    pub checksum_sha256_hex: String,

    pub etag: String,

    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "contentLength")]
    pub content_length: String,

    #[serde(rename = "lastModified")]
    pub last_modified: String,

    #[serde(rename = "scopeStatus")]
    pub scope_status: String,

    #[serde(rename = "nodeStatus")]
    pub node_status: String,

    pub eligibility: String,
}
