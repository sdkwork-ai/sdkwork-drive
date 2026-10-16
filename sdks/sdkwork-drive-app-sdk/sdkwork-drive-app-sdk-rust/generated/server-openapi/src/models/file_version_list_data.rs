use serde::{Deserialize, Serialize};

use crate::models::{FileVersion, PageInfo};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FileVersionListData {
    pub items: Vec<FileVersion>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
