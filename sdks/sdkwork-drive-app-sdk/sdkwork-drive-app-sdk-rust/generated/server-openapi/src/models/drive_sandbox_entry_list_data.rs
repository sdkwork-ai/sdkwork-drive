use serde::{Deserialize, Serialize};

use crate::models::{DriveSandboxEntry, PageInfo};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSandboxEntryListData {
    pub items: Vec<DriveSandboxEntry>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
