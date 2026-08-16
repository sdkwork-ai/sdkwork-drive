use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SweepObjectStoreRequest {
    #[serde(rename = "dryRun")]
    pub dry_run: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}
