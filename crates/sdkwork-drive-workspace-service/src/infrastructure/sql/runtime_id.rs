use std::sync::OnceLock;

use sdkwork_database_id::SnowflakeIdGenerator;

use crate::DriveServiceError;

const DEFAULT_DRIVE_NODE_ID: u16 = 31;
const DRIVE_NODE_ID_ENV: &str = "SDKWORK_DRIVE_SNOWFLAKE_NODE_ID";

static DRIVE_RUNTIME_ID_GENERATOR: OnceLock<Result<SnowflakeIdGenerator, String>> = OnceLock::new();

pub fn next_drive_runtime_id(context: &str) -> Result<i64, DriveServiceError> {
    let generator = drive_runtime_id_generator()?;
    generator.generate().map_err(|error| {
        DriveServiceError::Internal(format!("failed to generate {context} id: {error:?}"))
    })
}

fn drive_runtime_id_generator() -> Result<&'static SnowflakeIdGenerator, DriveServiceError> {
    match DRIVE_RUNTIME_ID_GENERATOR.get_or_init(build_drive_runtime_id_generator) {
        Ok(generator) => Ok(generator),
        Err(message) => Err(DriveServiceError::Internal(message.clone())),
    }
}

fn build_drive_runtime_id_generator() -> Result<SnowflakeIdGenerator, String> {
    let node_id = match std::env::var(DRIVE_NODE_ID_ENV) {
        Ok(value) if !value.trim().is_empty() => value
            .trim()
            .parse::<u16>()
            .map_err(|_| format!("{DRIVE_NODE_ID_ENV} must be an integer between 0 and 1023"))?,
        Ok(_) => {
            return Err(format!(
                "{DRIVE_NODE_ID_ENV} must be an integer between 0 and 1023"
            ));
        }
        Err(_) => DEFAULT_DRIVE_NODE_ID,
    };

    SnowflakeIdGenerator::new(node_id)
        .map_err(|error| format!("{DRIVE_NODE_ID_ENV} is invalid for Drive runtime IDs: {error:?}"))
}
