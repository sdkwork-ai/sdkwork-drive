use serde::{Deserialize, Serialize};

use crate::models::{DriveResourceResolution};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveResourceResolutionResourceData {
    pub item: DriveResourceResolution,
}
