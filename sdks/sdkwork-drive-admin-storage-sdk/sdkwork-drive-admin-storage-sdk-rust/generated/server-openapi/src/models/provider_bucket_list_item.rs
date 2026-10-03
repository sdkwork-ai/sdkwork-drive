use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderBucketListItem {
    pub bucket: String,

    pub configured: bool,

    #[serde(rename = "creationDateEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creation_date_epoch_ms: Option<String>,

    /// Region the vendor reports for this bucket on its account bucket inventory (`Location`). The inventory is an account-level read that spans regions, so the region belongs to the row rather than to the provider configuration; omitted for vendors that report none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}
