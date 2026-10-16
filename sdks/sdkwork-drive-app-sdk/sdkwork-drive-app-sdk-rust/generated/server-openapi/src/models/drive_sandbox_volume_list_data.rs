use serde::{Deserialize, Serialize};

use crate::models::{DriveSandboxVolume, PageInfo};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSandboxVolumeListData {
    pub items: Vec<DriveSandboxVolume>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
