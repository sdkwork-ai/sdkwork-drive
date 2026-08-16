use serde::{Deserialize, Serialize};

/// Drive space types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveSpaceType {
    /// User-owned files and folders.
    Personal,
    /// Organization/team shared files.
    Team,
    /// Files managed for knowledge ingestion, indexing, and retrieval.
    KnowledgeBase,
    /// AI-generated artifacts, model outputs, edited media.
    AiGenerated,
    /// Git repository files.
    GitRepository,
    /// Deployment artifacts.
    Deployment,
    /// Application-owned uploads (product media, avatars, etc.).
    AppUpload,
    /// Instant-messaging files.
    Im,
    /// Real-time communication files.
    Rtc,
    /// Notary/verification files.
    Notary,
    /// Directory-faithful static website projects.
    Website,
}

impl DriveSpaceType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Personal => "personal",
            Self::Team => "team",
            Self::KnowledgeBase => "knowledge_base",
            Self::AiGenerated => "ai_generated",
            Self::GitRepository => "git_repository",
            Self::Deployment => "deployment",
            Self::AppUpload => "app_upload",
            Self::Im => "im",
            Self::Rtc => "rtc",
            Self::Notary => "notary",
            Self::Website => "website",
        }
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        match raw {
            "personal" => Some(Self::Personal),
            "team" => Some(Self::Team),
            "knowledge_base" => Some(Self::KnowledgeBase),
            "ai_generated" => Some(Self::AiGenerated),
            "git_repository" => Some(Self::GitRepository),
            "deployment" => Some(Self::Deployment),
            "app_upload" => Some(Self::AppUpload),
            "im" => Some(Self::Im),
            "rtc" => Some(Self::Rtc),
            "notary" => Some(Self::Notary),
            "website" => Some(Self::Website),
            _ => None,
        }
    }
}

/// Drive space entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveSpace {
    pub id: String,
    pub tenant_id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub space_type: DriveSpaceType,
    pub name: String,
    pub version: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}
