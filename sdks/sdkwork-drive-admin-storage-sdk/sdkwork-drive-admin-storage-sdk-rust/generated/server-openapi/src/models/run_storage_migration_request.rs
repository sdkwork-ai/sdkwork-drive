use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RunStorageMigrationRequest {
    /// Objects to copy in this call. Clamped into the supported range rather than rejected, because it is a throughput knob and not a contract.
    #[serde(rename = "batchSize")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<i64>,
}
