use jsonwebtoken::DecodingKey;
use std::collections::BTreeMap;

/// Drive auth validation policy for dual-token parsing.
#[derive(Clone)]
pub struct DriveAuthValidationPolicy {
    pub allow_inline_claim_tokens: bool,
    pub jwt_hmac_secrets: BTreeMap<String, String>,
    pub jwt_jwks_keys: BTreeMap<String, DecodingKey>,
}

impl std::fmt::Debug for DriveAuthValidationPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DriveAuthValidationPolicy")
            .field("allow_inline_claim_tokens", &self.allow_inline_claim_tokens)
            .field(
                "jwt_hmac_secret_kids",
                &self.jwt_hmac_secrets.keys().collect::<Vec<_>>(),
            )
            .field(
                "jwt_jwks_key_ids",
                &self.jwt_jwks_keys.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Default for DriveAuthValidationPolicy {
    fn default() -> Self {
        Self::from_env()
    }
}

impl DriveAuthValidationPolicy {
    fn is_production_runtime_profile() -> bool {
        std::env::var("SDKWORK_DRIVE_RUNTIME_PROFILE")
            .ok()
            .map(|value| {
                let value = value.trim().to_ascii_lowercase();
                value == "production" || value == "prod"
            })
            .unwrap_or(false)
    }

    pub fn allow_unsigned_for_development() -> Self {
        Self {
            allow_inline_claim_tokens: true,
            jwt_hmac_secrets: BTreeMap::new(),
            jwt_jwks_keys: BTreeMap::new(),
        }
    }

    pub fn from_env() -> Self {
        let mut jwt_hmac_secrets = BTreeMap::new();
        if let Ok(secret) = std::env::var("SDKWORK_DRIVE_JWT_HMAC_SECRET") {
            let secret = secret.trim().to_string();
            if !secret.is_empty() {
                jwt_hmac_secrets.insert("default".to_string(), secret);
            }
        }
        if let Ok(raw) = std::env::var("SDKWORK_DRIVE_JWT_HMAC_SECRETS_JSON") {
            if let Ok(value) = serde_json::from_str::<BTreeMap<String, String>>(&raw) {
                for (kid, secret) in value {
                    let secret = secret.trim().to_string();
                    if !kid.trim().is_empty() && !secret.is_empty() {
                        jwt_hmac_secrets.insert(kid.trim().to_string(), secret);
                    }
                }
            }
        }
        let jwt_jwks_keys = crate::jwks::load_jwt_jwks_from_env();

        if jwt_hmac_secrets.is_empty() && jwt_jwks_keys.is_empty() {
            if Self::is_production_runtime_profile() {
                Self {
                    allow_inline_claim_tokens: false,
                    jwt_hmac_secrets: BTreeMap::new(),
                    jwt_jwks_keys: BTreeMap::new(),
                }
            } else {
                Self::allow_unsigned_for_development()
            }
        } else {
            Self {
                allow_inline_claim_tokens: false,
                jwt_hmac_secrets,
                jwt_jwks_keys,
            }
        }
    }

    pub fn require_signed_projection(secret: impl Into<String>) -> Self {
        let mut jwt_hmac_secrets = BTreeMap::new();
        jwt_hmac_secrets.insert("default".to_string(), secret.into());
        Self {
            allow_inline_claim_tokens: false,
            jwt_hmac_secrets,
            jwt_jwks_keys: BTreeMap::new(),
        }
    }

    pub fn has_jwks(&self) -> bool {
        !self.jwt_jwks_keys.is_empty()
    }

    pub fn requires_signed_jwt(&self) -> bool {
        !self.jwt_hmac_secrets.is_empty()
            || self.has_jwks()
            || Self::is_production_runtime_profile()
    }

    pub fn ensure_production_ready(&self) -> Result<(), String> {
        if !Self::is_production_runtime_profile() {
            return Ok(());
        }
        if self.allow_inline_claim_tokens {
            return Err(
                "production runtime cannot allow inline claim tokens; configure JWT HMAC or JWKS"
                    .to_string(),
            );
        }
        if self.jwt_hmac_secrets.is_empty() && self.jwt_jwks_keys.is_empty() {
            return Err(
                "production runtime requires SDKWORK_DRIVE_JWT_HMAC_SECRET, \
                 SDKWORK_DRIVE_JWT_HMAC_SECRETS_JSON, or SDKWORK_DRIVE_JWT_JWKS_URL"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn resolve_jwt_hmac_secret(&self, kid: Option<&str>) -> Option<&str> {
        if let Some(kid) = kid.map(str::trim).filter(|value| !value.is_empty()) {
            if let Some(secret) = self.jwt_hmac_secrets.get(kid) {
                return Some(secret.as_str());
            }
        }
        self.jwt_hmac_secrets
            .get("default")
            .map(String::as_str)
            .or_else(|| self.jwt_hmac_secrets.values().next().map(String::as_str))
    }

    pub fn resolve_jwt_jwks_key(&self, kid: Option<&str>) -> Option<&DecodingKey> {
        if let Some(kid) = kid.map(str::trim).filter(|value| !value.is_empty()) {
            if let Some(key) = self.jwt_jwks_keys.get(kid) {
                return Some(key);
            }
        }
        self.jwt_jwks_keys
            .get("default")
            .or_else(|| self.jwt_jwks_keys.values().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwks::parse_jwt_jwks_json;
    use std::sync::{Mutex, MutexGuard};

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn lock_env_tests() -> MutexGuard<'static, ()> {
        ENV_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct EnvVarRestore {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarRestore {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarRestore {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn production_auth_policy_requires_signing_material() {
        let _guard = lock_env_tests();
        let _profile = EnvVarRestore::set("SDKWORK_DRIVE_RUNTIME_PROFILE", "production");
        let _hmac = EnvVarRestore::remove("SDKWORK_DRIVE_JWT_HMAC_SECRET");
        let _hmac_json = EnvVarRestore::remove("SDKWORK_DRIVE_JWT_HMAC_SECRETS_JSON");
        let _jwks_json = EnvVarRestore::remove("SDKWORK_DRIVE_JWT_JWKS_JSON");

        let policy = DriveAuthValidationPolicy::from_env();
        assert!(policy.requires_signed_jwt());
        assert!(!policy.allow_inline_claim_tokens);
        let error = policy
            .ensure_production_ready()
            .expect_err("production without secrets must fail readiness");
        assert!(error.contains("production runtime requires"));
    }

    #[test]
    fn resolves_kid_specific_secret_before_default() {
        let policy = DriveAuthValidationPolicy {
            allow_inline_claim_tokens: false,
            jwt_hmac_secrets: BTreeMap::from([
                ("default".to_string(), "default-secret".to_string()),
                ("tenant-a".to_string(), "tenant-a-secret".to_string()),
            ]),
            jwt_jwks_keys: BTreeMap::new(),
        };

        assert_eq!(
            policy.resolve_jwt_hmac_secret(Some("tenant-a")),
            Some("tenant-a-secret")
        );
        assert_eq!(policy.resolve_jwt_hmac_secret(None), Some("default-secret"));
    }

    #[test]
    fn rejects_empty_jwks_document() {
        assert!(matches!(
            parse_jwt_jwks_json(r#"{"keys":[]}"#),
            Err(message) if message.contains("did not include any RSA keys")
        ));
    }

    #[test]
    fn resolves_configured_jwks_key_by_kid() {
        let public_pem = concat!(
            "-----BEGIN PUBLIC KEY-----\n",
            "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAlVdGoYQZS2mN42WV6gfI\n",
            "B6v0DHLwmuilOM9dclUNamtBJHoRQgHfprVw60A+vbuPv6ODRKF+0m8rY5MSumCo\n",
            "fqSbsAkF/q+pY22YQcS8lXsutv2qK1lSgoeNn/Q/r0eWju76QosNybV1/fouV0q9\n",
            "kErd+MKpX01uAHNxUsLR7naMvRE0yAlUtewVau3yqNR0MhNlf50kfoI0D/VwISvW\n",
            "wzUjjZRCDzCzuMwhzbjB3WT2fKluwkVdN3JQey3sOMkz5E3BOQy77++WrTTslkoj\n",
            "UzA84rjUPNzSKrGtgBcuqY8nCAgUgQWSxIQPN11RWu/t0gTHMTRnp+AgItuCzzFo\n",
            "9wIDAQAB\n",
            "-----END PUBLIC KEY-----\n",
        );
        let decoding_key =
            DecodingKey::from_rsa_pem(public_pem.as_bytes()).expect("test rsa public key");
        let policy = DriveAuthValidationPolicy {
            allow_inline_claim_tokens: false,
            jwt_hmac_secrets: BTreeMap::new(),
            jwt_jwks_keys: BTreeMap::from([("iam-default".to_string(), decoding_key)]),
        };

        assert!(policy.has_jwks());
        assert!(policy.resolve_jwt_jwks_key(Some("iam-default")).is_some());
    }
}
