use serde::{Deserialize, Serialize};

use crate::models::{DriveSandboxEntry};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSandboxEntryData {
    pub item: DriveSandboxEntry,
}
