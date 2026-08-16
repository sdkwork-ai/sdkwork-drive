//! Redis-backed rate-limit backend for multi-instance deployments.
//!
//! Uses a sliding-window counter stored as a Redis sorted set with a Lua script
//! for atomic prune/count/add semantics across gateway instances.
//!
//! This module is gated behind the `redis-rate-limit` feature flag.
//! Build with: `cargo build --features redis-rate-limit`

use std::sync::OnceLock;

use redis::aio::ConnectionManager;

use crate::middleware::rate_limit::RateLimitConfig;

const REDIS_SLIDING_WINDOW_LUA: &str = r#"
local key = KEYS[1]
local now = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local member = ARGV[3]
local cutoff = now - window
redis.call('ZREMRANGEBYSCORE', key, 0, cutoff)
redis.call('ZADD', key, now, member)
redis.call('EXPIRE', key, math.ceil(window / 1000) + 1)
return redis.call('ZCOUNT', key, cutoff, now)
"#;

/// Shared Redis connection manager (singleton per process).
fn redis_manager() -> Option<&'static ConnectionManager> {
    static MANAGER: OnceLock<Option<ConnectionManager>> = OnceLock::new();
    MANAGER
        .get_or_init(|| {
            let url = std::env::var("SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL")
                .or_else(|_| std::env::var("REDIS_URL"))
                .ok()?;
            let client = redis::Client::open(url.as_str()).ok()?;
            let manager = tokio::runtime::Handle::try_current()
                .ok()
                .and_then(|handle| {
                    tokio::task::block_in_place(|| {
                        handle.block_on(async { ConnectionManager::new(client.clone()).await.ok() })
                    })
                })
                .or_else(|| {
                    tokio::runtime::Runtime::new().ok().and_then(|rt| {
                        rt.block_on(async { ConnectionManager::new(client).await.ok() })
                    })
                });
            manager
        })
        .as_ref()
}

fn redis_fail_open(config: &RateLimitConfig) -> bool {
    !config.fail_closed
}

/// Allow check using Redis sorted set sliding window.
///
/// Returns `true` if the key is within its rate limit, `false` otherwise.
pub async fn redis_allow(key: &str, config: &RateLimitConfig) -> bool {
    if !config.enabled {
        return true;
    }

    let manager = match redis_manager() {
        Some(m) => m.clone(),
        None => {
            tracing::warn!(
                target: "sdkwork.drive",
                event = "drive.rate_limit.redis_unavailable",
                key = key,
                fail_closed = config.fail_closed,
                "redis rate limit backend unavailable"
            );
            return redis_fail_open(config);
        }
    };

    let window_ms = config.window.as_millis() as i64;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let member = sdkwork_utils_rust::uuid();

    let mut conn = manager.clone();
    let result: Result<i64, _> = redis::Script::new(REDIS_SLIDING_WINDOW_LUA)
        .key(key)
        .arg(now_ms)
        .arg(window_ms)
        .arg(member)
        .invoke_async(&mut conn)
        .await;

    match result {
        Ok(count) => (count as u32) <= config.max_requests,
        Err(error) => {
            tracing::warn!(
                target: "sdkwork.drive",
                event = "drive.rate_limit.redis_error",
                key = key,
                error = %error,
                fail_closed = config.fail_closed,
                "redis rate limit script failed"
            );
            redis_fail_open(config)
        }
    }
}
