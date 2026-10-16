use serde::{Deserialize, Serialize};

use crate::models::{DriveSandboxEntry};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSandboxFileContent {
    pub entry: DriveSandboxEntry,

    pub encoding: String,

    pub content: String,

    #[serde(rename = "sizeBytes")]
    pub size_bytes: String,

    #[serde(rename = "checksumSha256")]
    pub checksum_sha256: String,
}
