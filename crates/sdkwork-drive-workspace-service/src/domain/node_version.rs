#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveNodeVersionKind {
    Auto,
    Manual,
    Restore,
    Import,
    AiGenerated,
    System,
}

impl DriveNodeVersionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
            Self::Restore => "restore",
            Self::Import => "import",
            Self::AiGenerated => "ai_generated",
            Self::System => "system",
        }
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        match raw {
            "auto" => Some(Self::Auto),
            "manual" => Some(Self::Manual),
            "restore" => Some(Self::Restore),
            "import" => Some(Self::Import),
            "ai_generated" => Some(Self::AiGenerated),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveNodeVersionChangeSource {
    AppApi,
    BackendApi,
    Uploader,
    Sync,
    Ai,
    Import,
    Restore,
    System,
}

impl DriveNodeVersionChangeSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AppApi => "app_api",
            Self::BackendApi => "backend_api",
            Self::Uploader => "uploader",
            Self::Sync => "sync",
            Self::Ai => "ai",
            Self::Import => "import",
            Self::Restore => "restore",
            Self::System => "system",
        }
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        match raw {
            "app_api" => Some(Self::AppApi),
            "backend_api" => Some(Self::BackendApi),
            "uploader" => Some(Self::Uploader),
            "sync" => Some(Self::Sync),
            "ai" => Some(Self::Ai),
            "import" => Some(Self::Import),
            "restore" => Some(Self::Restore),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveNodeVersion {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub version_no: i64,
    pub storage_object_id: Option<String>,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub version_kind: DriveNodeVersionKind,
    pub version_label: Option<String>,
    pub change_source: DriveNodeVersionChangeSource,
    pub change_summary: Option<String>,
    pub restored_from_version_id: Option<String>,
    pub app_id: Option<String>,
    pub app_resource_type: Option<String>,
    pub app_resource_id: Option<String>,
    pub scene: Option<String>,
    pub source: Option<String>,
    pub lifecycle_status: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateDriveNodeVersionCommand {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub node_id: String,
    pub version_no: i64,
    pub storage_object_id: Option<String>,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub version_kind: DriveNodeVersionKind,
    pub version_label: Option<String>,
    pub change_source: DriveNodeVersionChangeSource,
    pub change_summary: Option<String>,
    pub restored_from_version_id: Option<String>,
    pub app_id: Option<String>,
    pub app_resource_type: Option<String>,
    pub app_resource_id: Option<String>,
    pub scene: Option<String>,
    pub source: Option<String>,
    pub operator_id: String,
}
