use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderBucketListItem {
    pub bucket: String,

    pub configured: bool,

    #[serde(rename = "creationDateEpochMs")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creation_date_epoch_ms: Option<i64>,
}
