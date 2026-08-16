use serde::{Deserialize, Serialize};

use crate::models::{DriveNode, DriveUploadSession};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateFileResponse {
    pub node: DriveNode,

    #[serde(rename = "uploadSession")]
    pub upload_session: DriveUploadSession,
}
