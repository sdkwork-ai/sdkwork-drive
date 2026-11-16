use serde::{Deserialize, Serialize};

use crate::models::{DriveNodeContent};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveNodeContentData {
    pub item: DriveNodeContent,
}
