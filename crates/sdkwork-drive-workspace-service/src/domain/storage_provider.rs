pub use sdkwork_drive_storage_contract::DriveStorageProviderKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveStorageProvider {
    pub id: String,
    pub provider_kind: DriveStorageProviderKind,
    pub name: String,
    pub endpoint_url: String,
    pub region: Option<String>,
    pub bucket: String,
    pub path_style: bool,
    pub strict_tls: bool,
    pub credential_ref: Option<String>,
    pub server_side_encryption_mode: Option<String>,
    pub default_storage_class: Option<String>,
    pub status: String,
    pub version: i64,
}
