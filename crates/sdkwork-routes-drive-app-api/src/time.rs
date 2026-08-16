use sdkwork_drive_workspace_service::DriveServiceError;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn current_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

pub(crate) fn signing_ttl_seconds(expires_at_epoch_ms: i64) -> Result<u32, DriveServiceError> {
    let remaining_ms = expires_at_epoch_ms - current_epoch_ms();
    if remaining_ms < 1_000 {
        return Err(DriveServiceError::NotFound(
            "download signing target has expired".to_string(),
        ));
    }
    Ok(((remaining_ms / 1_000).min(3_600)) as u32)
}
