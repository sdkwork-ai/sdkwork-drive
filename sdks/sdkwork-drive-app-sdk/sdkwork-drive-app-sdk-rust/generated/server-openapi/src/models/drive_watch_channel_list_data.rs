use serde::{Deserialize, Serialize};

use crate::models::{DriveWatchChannel, PageInfo};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveWatchChannelListData {
    pub items: Vec<DriveWatchChannel>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
