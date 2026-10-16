use serde::{Deserialize, Serialize};

use crate::models::{Change, PageInfo};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ChangeListData {
    pub items: Vec<Change>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
