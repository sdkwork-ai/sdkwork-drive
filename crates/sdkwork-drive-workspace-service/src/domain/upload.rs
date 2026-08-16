#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveUploadSessionState {
    Created,
    Uploading,
    Completing,
    Completed,
    Aborted,
    Expired,
}

impl DriveUploadSessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Uploading => "uploading",
            Self::Completing => "completing",
            Self::Completed => "completed",
            Self::Aborted => "aborted",
            Self::Expired => "expired",
        }
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        match raw {
            "created" => Some(Self::Created),
            "uploading" => Some(Self::Uploading),
            "completing" => Some(Self::Completing),
            "completed" => Some(Self::Completed),
            "aborted" => Some(Self::Aborted),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveUploadSession {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub bucket: String,
    pub object_key: String,
    pub idempotency_key: String,
    pub storage_provider_id: String,
    pub storage_upload_id: String,
    pub state: DriveUploadSessionState,
    pub expires_at_epoch_ms: i64,
    pub version: i64,
}
