//! Storage provider kind registry: built-in provider brands with an
//! enable/disable switch. One kind owns many provider configurations
//! (`DriveStorageProvider` rows); `custom:<vendor>` kinds are implicit and
//! never stored in the registry.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveStorageProviderKindRegistry {
    pub provider_kind: String,
    pub display_name: String,
    pub enabled: bool,
    pub sort_order: i32,
    pub version: i64,
}
