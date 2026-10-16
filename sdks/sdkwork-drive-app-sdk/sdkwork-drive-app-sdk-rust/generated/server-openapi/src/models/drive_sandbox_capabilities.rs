use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveSandboxCapabilities {
    pub browse: bool,

    #[serde(rename = "createDirectory")]
    pub create_directory: bool,

    #[serde(rename = "selectDirectory")]
    pub select_directory: bool,

    #[serde(rename = "readFile")]
    pub read_file: bool,

    #[serde(rename = "createFile")]
    pub create_file: bool,

    #[serde(rename = "writeFile")]
    pub write_file: bool,

    #[serde(rename = "moveEntry")]
    pub move_entry: bool,

    #[serde(rename = "deleteEntry")]
    pub delete_entry: bool,
}
