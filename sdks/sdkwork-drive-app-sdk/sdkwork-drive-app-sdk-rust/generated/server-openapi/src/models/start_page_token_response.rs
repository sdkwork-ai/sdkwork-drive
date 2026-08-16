use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StartPageTokenResponse {
    #[serde(rename = "startPageToken")]
    pub start_page_token: String,
}
