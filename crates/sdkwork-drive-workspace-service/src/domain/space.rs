#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveSpaceType {
    Personal,
    Team,
    KnowledgeBase,
    AiGenerated,
    GitRepository,
    Deployment,
    AppUpload,
    Im,
    Rtc,
    Notary,
    Website,
}

impl DriveSpaceType {
    pub fn as_str(&self) -> &'static str {
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

    pub fn requires_user_owner(&self) -> bool {
        matches!(self, Self::GitRepository | Self::Rtc)
    }

    pub fn is_non_deletable(&self) -> bool {
        matches!(self, Self::GitRepository)
    }

    pub fn accepts_only_root_folders(&self) -> bool {
        matches!(self, Self::GitRepository)
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            Self::Personal => "personal space",
            Self::Team => "team space",
            Self::KnowledgeBase => "knowledge base space",
            Self::AiGenerated => "AI generated space",
            Self::GitRepository => "git repository space",
            Self::Deployment => "deployment space",
            Self::AppUpload => "app upload space",
            Self::Im => "IM space",
            Self::Rtc => "rtc space",
            Self::Notary => "notary space",
            Self::Website => "website space",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveSpace {
    pub id: String,
    pub tenant_id: String,
    pub owner_subject_type: String,
    pub owner_subject_id: String,
    pub display_name: String,
    pub presentation_icon: Option<String>,
    pub presentation_color: Option<String>,
    pub description: Option<String>,
    pub space_type: DriveSpaceType,
    pub lifecycle_status: String,
    pub version: i64,
    pub created_by: String,
}
