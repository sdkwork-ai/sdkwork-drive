use crate::domain::space::DriveSpaceType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveNodeType {
    File,
    Folder,
    Shortcut,
    VirtualReference,
}

impl DriveNodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Folder => "folder",
            Self::Shortcut => "shortcut",
            Self::VirtualReference => "virtual_reference",
        }
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        match raw {
            "file" => Some(Self::File),
            "folder" => Some(Self::Folder),
            "shortcut" => Some(Self::Shortcut),
            "virtual_reference" => Some(Self::VirtualReference),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveNode {
    pub id: String,
    pub tenant_id: String,
    pub space_id: String,
    pub space_type: DriveSpaceType,
    pub parent_node_id: Option<String>,
    pub shortcut_target_node_id: Option<String>,
    pub node_type: DriveNodeType,
    pub node_name: String,
    pub lifecycle_status: String,
    pub version: i64,
}
