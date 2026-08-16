use crate::constants::{
    LAST_APP_SNOWFLAKE_ID, SDKWORK_DRIVE_WORKER_ID, SDKWORK_SNOWFLAKE_EPOCH_MS,
};
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn next_drive_id(prefix: &str) -> String {
    format!("{prefix}_{}", next_snowflake_id())
}

fn next_snowflake_id() -> u64 {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    let timestamp_part = now_ms
        .saturating_sub(SDKWORK_SNOWFLAKE_EPOCH_MS)
        .min((1_u64 << 41) - 1);
    let base = (timestamp_part << 22) | (SDKWORK_DRIVE_WORKER_ID << 12);
    loop {
        let previous = LAST_APP_SNOWFLAKE_ID.load(Ordering::Relaxed);
        let candidate = if base > previous { base } else { previous + 1 };
        if LAST_APP_SNOWFLAKE_ID
            .compare_exchange(previous, candidate, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            return candidate;
        }
    }
}
