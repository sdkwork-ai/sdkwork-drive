//! Rate limiting middleware with pluggable backends.
//!
//! Provides:
//! - `InMemoryRateLimitBackend`: local process-only, suitable for single-instance deployments.
//! - `redis_rate_limit`: Redis sorted-set sliding window for multi-instance deployments
//!   (requires `redis-rate-limit` feature and `SDKWORK_DRIVE_RATE_LIMIT_BACKEND=redis`).
//! - `RateLimitConfig`: unified configuration loaded once from environment variables.
//!
//! Configuration environment variables (shared across all rate-limit consumers):
//!
//! | Variable | Default | Description |
//! |---|---|---|
//! | `SDKWORK_DRIVE_RATE_LIMIT_ENABLED` | `true` | Master toggle |
//! | `SDKWORK_DRIVE_RATE_LIMIT_BACKEND` | `"memory"` | `"memory"` or `"redis"` |
//! | `SDKWORK_DRIVE_RATE_LIMIT_REDIS_URL` | — | Redis URL (required when backend=redis) |
//! | `SDKWORK_DRIVE_RATE_LIMIT_DEFAULT_WINDOW_SECONDS` | `60` | Default sliding window |
//! | `SDKWORK_DRIVE_RATE_LIMIT_DEFAULT_MAX_REQUESTS` | `600` | Default max requests per window |
//! | `SDKWORK_DRIVE_RATE_LIMIT_TRUST_PROXY` | `false` | Trust `x-forwarded-for` / `x-real-ip` |
//! | `SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED` | `false` | Deny when Redis backend is unavailable |

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::{Duration, Instant};

const MAX_RATE_LIMIT_BUCKETS: usize = 10_000;

/// Unified rate-limit configuration, loaded once from environment variables.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub window: Duration,
    pub max_requests: u32,
    pub trust_proxy: bool,
    pub fail_closed: bool,
}

impl RateLimitConfig {
    /// Load configuration from environment variables with unified prefix.
    ///
    /// Individual surfaces can override `window` and `max_requests` via their own
    /// env vars (e.g. `SDKWORK_DRIVE_APP_API_RATE_LIMIT_WINDOW_SECONDS`), falling
    /// back to the shared defaults.
    pub fn from_env_with_overrides(
        window_env: &str,
        max_env: &str,
        default_window_seconds: u64,
        default_max_requests: u32,
    ) -> Self {
        let enabled = Self::env_bool("SDKWORK_DRIVE_RATE_LIMIT_ENABLED", true);
        let window_seconds = std::env::var(window_env)
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v > 0)
            .or_else(|| {
                std::env::var("SDKWORK_DRIVE_RATE_LIMIT_DEFAULT_WINDOW_SECONDS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|v| *v > 0)
            })
            .unwrap_or(default_window_seconds);
        let max_requests = std::env::var(max_env)
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .or_else(|| {
                std::env::var("SDKWORK_DRIVE_RATE_LIMIT_DEFAULT_MAX_REQUESTS")
                    .ok()
                    .and_then(|v| v.parse::<u32>().ok())
                    .filter(|v| *v > 0)
            })
            .unwrap_or(default_max_requests);
        let trust_proxy = Self::env_bool("SDKWORK_DRIVE_RATE_LIMIT_TRUST_PROXY", false);
        let fail_closed = Self::env_bool("SDKWORK_DRIVE_RATE_LIMIT_FAIL_CLOSED", false);

        Self {
            enabled,
            window: Duration::from_secs(window_seconds),
            max_requests,
            trust_proxy,
            fail_closed,
        }
    }

    pub(crate) fn env_bool(key: &str, default: bool) -> bool {
        std::env::var(key)
            .ok()
            .map(|v| {
                let v = v.trim().to_ascii_lowercase();
                v == "1" || v == "true" || v == "on" || v == "yes"
            })
            .unwrap_or(default)
    }
}

/// In-process sliding-window rate-limit backend.
///
/// **Limitation**: state is per-process. Multi-instance deployments should use
/// a Redis backend via the `redis` feature flag.
struct InMemoryRateLimitBackend {
    config: RateLimitConfig,
    buckets: Mutex<HashMap<String, Vec<Instant>>>,
}

impl InMemoryRateLimitBackend {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    fn allow(&self, key: &str) -> bool {
        if !self.config.enabled {
            return true;
        }

        let now = Instant::now();
        let mut buckets = match self.buckets.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // Evict oldest bucket when at capacity and key is new.
        if buckets.len() >= MAX_RATE_LIMIT_BUCKETS && !buckets.contains_key(key) {
            if let Some(oldest_key) = buckets
                .iter()
                .min_by_key(|(_, entries)| entries.first().copied().unwrap_or_else(Instant::now))
                .map(|(bk, _)| bk.clone())
            {
                buckets.remove(&oldest_key);
            }
        }

        let entries = buckets.entry(key.to_string()).or_default();
        entries.retain(|seen| now.duration_since(*seen) <= self.config.window);

        if entries.len() as u32 >= self.config.max_requests {
            return false;
        }
        entries.push(now);
        true
    }
}

fn resolve_rate_limit_backend_type() -> &'static str {
    static BACKEND: OnceLock<&'static str> = OnceLock::new();
    BACKEND.get_or_init(|| {
        #[cfg(feature = "redis-rate-limit")]
        {
            if std::env::var("SDKWORK_DRIVE_RATE_LIMIT_BACKEND")
                .ok()
                .map(|value| value.trim().to_ascii_lowercase())
                .as_deref()
                == Some("redis")
            {
                return "redis";
            }
        }
        "memory"
    })
}

/// Per-API-surface rate-limit state. Each surface keeps its own window and max request budget.
pub struct SharedRateLimitState {
    pub(crate) surface_id: &'static str,
    pub(crate) config: RateLimitConfig,
    backend: InMemoryRateLimitBackend,
    backend_type: &'static str,
}

fn build_shared_rate_limit_state(
    surface_id: &'static str,
    config: RateLimitConfig,
) -> SharedRateLimitState {
    SharedRateLimitState {
        surface_id,
        backend: InMemoryRateLimitBackend::new(config),
        config,
        backend_type: resolve_rate_limit_backend_type(),
    }
}

static RATE_LIMIT_STATES: LazyLock<Mutex<HashMap<&'static str, &'static SharedRateLimitState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Resolve or create a shared rate-limit state for an API surface.
pub fn shared_rate_limit_state(
    surface_id: &'static str,
    init: impl FnOnce() -> RateLimitConfig,
) -> &'static SharedRateLimitState {
    let mut states = RATE_LIMIT_STATES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(state) = states.get(surface_id) {
        return state;
    }
    let state: &'static SharedRateLimitState =
        Box::leak(Box::new(build_shared_rate_limit_state(surface_id, init())));
    states.insert(surface_id, state);
    state
}

/// Extract the client IP/identity from the request for rate-limit key derivation.
pub fn rate_limit_client_key(request: &Request<Body>, fallback: &str, trust_proxy: bool) -> String {
    if trust_proxy {
        if let Some(real_ip) = request
            .headers()
            .get("x-real-ip")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return real_ip.to_string();
        }

        if let Some(forwarded_for) = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.split(',').next().unwrap_or(value).trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return forwarded_for;
        }
    }

    fallback.to_string()
}

/// Sliding-window rate-limit middleware.
pub async fn sliding_window_rate_limit(
    state: &'static SharedRateLimitState,
    request: Request<Body>,
    next: Next,
    message: &'static str,
) -> Response {
    let client_key = rate_limit_client_key(&request, "direct-client", state.config.trust_proxy);
    let key = format!("{}:{client_key}:{}", state.surface_id, request.uri().path());
    let allowed = if state.backend_type == "redis" {
        #[cfg(feature = "redis-rate-limit")]
        {
            super::redis_rate_limit::redis_allow(&key, &state.config).await
        }
        #[cfg(not(feature = "redis-rate-limit"))]
        {
            state.backend.allow(&key)
        }
    } else {
        state.backend.allow(&key)
    };
    if !allowed {
        sdkwork_drive_observability::metrics::record_http_rate_limited();
        return (StatusCode::TOO_MANY_REQUESTS, message).into_response();
    }
    next.run(request).await
}

/// Convenience: build `RateLimitConfig` from env with default overrides.
///
/// Prefer this over manual `RateLimitConfig::from_env_with_overrides()` calls.
pub fn rate_limit_config_from_env(
    window_env: &str,
    max_env: &str,
    default_window_seconds: u64,
    default_max_requests: u32,
) -> RateLimitConfig {
    RateLimitConfig::from_env_with_overrides(
        window_env,
        max_env,
        default_window_seconds,
        default_max_requests,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn rate_limit_client_key_ignores_forwarded_headers_without_trust_proxy() {
        let request = Request::builder()
            .header("x-forwarded-for", "203.0.113.10")
            .body(Body::empty())
            .expect("request");
        assert_eq!(
            rate_limit_client_key(&request, "direct-client", false),
            "direct-client"
        );
        assert_eq!(
            rate_limit_client_key(&request, "direct-client", true),
            "203.0.113.10"
        );
    }
}
