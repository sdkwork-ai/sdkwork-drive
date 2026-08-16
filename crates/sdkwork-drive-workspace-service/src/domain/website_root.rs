#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveWebsiteSourceRootMode {
    SpaceRoot,
    Folder,
}

impl DriveWebsiteSourceRootMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SpaceRoot => "space_root",
            Self::Folder => "folder",
        }
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        match raw {
            "space_root" => Some(Self::SpaceRoot),
            "folder" => Some(Self::Folder),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveWebsiteContentMode {
    LiveTree,
    AtomicGeneration,
}

impl DriveWebsiteContentMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LiveTree => "live_tree",
            Self::AtomicGeneration => "atomic_generation",
        }
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        match raw {
            "live_tree" => Some(Self::LiveTree),
            "atomic_generation" => Some(Self::AtomicGeneration),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveWebsiteRoot {
    pub id: String,
    pub uuid: String,
    pub tenant_id: String,
    pub space_id: String,
    pub root_key: String,
    pub display_name: String,
    pub source_root_mode: DriveWebsiteSourceRootMode,
    pub selected_folder_node_id: Option<String>,
    pub content_mode: DriveWebsiteContentMode,
    pub active_node_id: String,
    pub active_generation: i64,
    pub root_status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}
