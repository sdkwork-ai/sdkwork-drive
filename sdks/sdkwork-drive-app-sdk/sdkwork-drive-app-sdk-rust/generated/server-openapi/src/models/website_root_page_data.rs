use serde::{Deserialize, Serialize};

use crate::models::{PageInfo, WebsiteRoot};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteRootPageData {
    pub items: Vec<WebsiteRoot>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
