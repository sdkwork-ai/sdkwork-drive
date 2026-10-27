use serde::{Deserialize, Serialize};

use crate::models::DriveWatchChannel;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StopWatchChannelResponse {
    pub stopped: bool,

    pub channel: DriveWatchChannel,
}
