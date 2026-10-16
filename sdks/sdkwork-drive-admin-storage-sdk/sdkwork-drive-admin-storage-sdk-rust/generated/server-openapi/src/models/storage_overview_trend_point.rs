use serde::{Deserialize, Serialize};

/// One monthly ingestion bucket.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageOverviewTrendPoint {
    /// Closed month bucket label in YYYY-MM.
    #[serde(rename = "periodLabel")]
    pub period_label: String,

    /// Objects created in that month.
    #[serde(rename = "objectCount")]
    pub object_count: String,

    /// Sum of content_length created in that month.
    pub bytes: String,
}
