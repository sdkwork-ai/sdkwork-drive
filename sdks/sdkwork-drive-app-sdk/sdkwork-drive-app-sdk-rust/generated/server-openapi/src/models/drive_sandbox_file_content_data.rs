use serde::{Deserialize, Serialize};

use crate::models::{DriveSandboxFileContent};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSandboxFileContentData {
    pub item: DriveSandboxFileContent,
}
