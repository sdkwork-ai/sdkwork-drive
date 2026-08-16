use crate::policy::DriveAuthValidationPolicy;
use std::sync::{Arc, OnceLock, RwLock};

static SHARED_POLICY: OnceLock<DriveAuthPolicyHandle> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct DriveAuthPolicyHandle {
    inner: Arc<RwLock<DriveAuthValidationPolicy>>,
}

impl DriveAuthPolicyHandle {
    pub fn shared_from_env() -> Self {
        SHARED_POLICY.get_or_init(Self::from_env).clone()
    }

    pub fn from_env() -> Self {
        Self::from_policy(DriveAuthValidationPolicy::from_env())
    }

    pub fn from_policy(policy: DriveAuthValidationPolicy) -> Self {
        Self {
            inner: Arc::new(RwLock::new(policy)),
        }
    }

    pub fn read<R>(&self, inspect: impl FnOnce(&DriveAuthValidationPolicy) -> R) -> R {
        let guard = self
            .inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        inspect(&guard)
    }

    pub async fn refresh_jwks_if_configured(&self) {
        let Some(url) = std::env::var("SDKWORK_DRIVE_JWT_JWKS_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
        else {
            return;
        };

        match crate::jwks::fetch_jwks_keys_from_url(&url).await {
            Ok(keys) if !keys.is_empty() => {
                let mut guard = self
                    .inner
                    .write()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.jwt_jwks_keys = keys;
                guard.allow_inline_claim_tokens = false;
            }
            Ok(_) => tracing::warn!("JWKS refresh from {url} returned no RSA keys"),
            Err(error) => tracing::warn!("JWKS refresh from {url} failed: {error}"),
        }
    }
}

pub fn spawn_drive_auth_jwks_refresh(handle: DriveAuthPolicyHandle) {
    let Some(url) = std::env::var("SDKWORK_DRIVE_JWT_JWKS_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return;
    };
    let refresh_secs = std::env::var("SDKWORK_DRIVE_JWT_JWKS_REFRESH_SECONDS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(300);
    tracing::info!(
        jwks_url = %url,
        refresh_seconds = refresh_secs,
        "starting Drive auth JWKS refresh task"
    );
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(refresh_secs));
        loop {
            interval.tick().await;
            handle.refresh_jwks_if_configured().await;
        }
    });
}

pub fn ensure_drive_auth_policy_refresh_task() {
    let handle = DriveAuthPolicyHandle::shared_from_env();
    match handle.read(|policy| policy.ensure_production_ready()) {
        Ok(()) => spawn_drive_auth_jwks_refresh(handle),
        Err(error) => {
            tracing::error!(
                event = "drive.auth.policy_not_production_ready",
                error = %error,
                "Drive auth policy is not production ready; JWKS refresh task was not started"
            );
        }
    }
}
