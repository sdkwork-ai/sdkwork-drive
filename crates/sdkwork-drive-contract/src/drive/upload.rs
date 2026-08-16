use serde::{Deserialize, Serialize};

/// Upload session states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveUploadSessionState {
    /// Session is reserved but no accepted upload progress is recorded.
    Created,
    /// Upload has started or at least one part/grant has been issued.
    Uploading,
    /// Content is committed and the target node points to the resulting storage object/version.
    Completed,
    /// Client or server canceled the upload.
    Aborted,
    /// Session exceeded its expiry and cannot be completed.
    Expired,
}

impl DriveUploadSessionState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Created => "created",
            Self::Uploading => "uploading",
            Self::Completed => "completed",
            Self::Aborted => "aborted",
            Self::Expired => "expired",
        }
    }
}

/// Drive upload session entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveUploadSession {
    pub id: String,
    pub space_id: String,
    pub node_id: String,
    pub idempotency_key: Option<String>,
    pub state: DriveUploadSessionState,
    pub expires_at_ms: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Upload part information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveUploadPart {
    pub part_number: i32,
    pub etag: Option<String>,
    pub size_bytes: u64,
    pub uploaded: bool,
}

/// Create upload session request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUploadSessionRequest {
    pub space_id: String,
    pub node_id: String,
    pub idempotency_key: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub operator_id: String,
}

/// Complete upload session request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteUploadSessionRequest {
    pub session_id: String,
    pub parts: Vec<DriveUploadPart>,
    pub operator_id: String,
}
