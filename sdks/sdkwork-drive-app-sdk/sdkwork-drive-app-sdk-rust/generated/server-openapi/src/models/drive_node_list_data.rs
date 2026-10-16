use serde::{Deserialize, Serialize};

use crate::models::{DriveNode, PageInfo};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveNodeListData {
    pub items: Vec<DriveNode>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,

    /// True when ACL pagination scan budget was exhausted before the requested page could be filled.
    #[serde(rename = "incompletePage")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_page: Option<bool>,
}
