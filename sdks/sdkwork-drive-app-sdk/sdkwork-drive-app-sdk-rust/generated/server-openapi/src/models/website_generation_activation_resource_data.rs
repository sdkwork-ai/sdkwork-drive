use serde::{Deserialize, Serialize};

use crate::models::{WebsiteGenerationActivation};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebsiteGenerationActivationResourceData {
    pub item: WebsiteGenerationActivation,
}
