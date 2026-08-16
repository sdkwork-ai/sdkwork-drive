use serde::{Deserialize, Serialize};

use crate::models::{UploadSessionMutationResponse, UploaderUploadItem};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PrepareUploaderUploadResponse {
    #[serde(rename = "uploadItem")]
    pub upload_item: UploaderUploadItem,

    #[serde(rename = "uploadSession")]
    pub upload_session: UploadSessionMutationResponse,
}
