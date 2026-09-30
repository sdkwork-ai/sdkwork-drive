#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveResourceScopeKind {
    WebsiteRoot,
    RootScopeSubscription,
}

impl DriveResourceScopeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebsiteRoot => "WEBSITE_ROOT",
            Self::RootScopeSubscription => "ROOT_SCOPE_SUBSCRIPTION",
        }
    }
}

/// Physical location a resolved Drive resource lives at.
///
/// Deliberately *not* pinned to `dr_drive_storage_provider.version`: that column
/// moves on every operational edit (rename, disable, storage-class change), and
/// pinning it would make a harmless rename unreadable for every object the
/// provider already holds. The fields here are the ones that actually decide
/// where the bytes are, and they are what the read path validates against the
/// live provider row before serving a historical object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveResourceContentLocator {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDriveResource {
    pub scope_kind: DriveResourceScopeKind,
    pub scope_uuid: String,
    pub scope_generation: i64,
    pub relative_path: String,
    pub resource_type: String,
    pub node_id: String,
    pub node_version_id: String,
    pub version_no: i64,
    pub checksum_sha256_hex: String,
    pub content_type: String,
    pub content_length: i64,
    pub last_modified: String,
    pub scope_status: String,
    pub node_status: String,
    pub eligibility: String,
    pub content_locator: DriveResourceContentLocator,
}
