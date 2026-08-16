#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantQuotaPolicy {
    pub max_bytes: Option<i64>,
}

impl TenantQuotaPolicy {
    pub fn from_env() -> Self {
        let max_bytes = std::env::var("SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| *value > 0);
        Self { max_bytes }
    }
}

/// Returns the canonical deployment profile label for metrics and tracing.
/// Prefers `SDKWORK_DRIVE_DEPLOYMENT_PROFILE`; accepts legacy `SDKWORK_DRIVE_DEPLOYMENT_MODE`.
pub fn resolve_deployment_profile_label() -> String {
    std::env::var("SDKWORK_DRIVE_DEPLOYMENT_PROFILE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("SDKWORK_DRIVE_DEPLOYMENT_MODE")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "standalone".to_string())
}

/// Returns true when production-hardening rules should be enforced.
pub fn is_production_runtime_profile() -> bool {
    matches_runtime_profile(&["production", "prod"])
}

fn matches_runtime_profile(accepted: &[&str]) -> bool {
    let Some(raw) = std::env::var("SDKWORK_DRIVE_RUNTIME_PROFILE")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    accepted.iter().any(|profile| *profile == raw)
}

/// Plaintext credential refs are only allowed outside production profiles.
pub fn allows_plain_credential_refs() -> bool {
    if is_production_runtime_profile() {
        return false;
    }
    std::env::var("SDKWORK_DRIVE_ALLOW_PLAIN_CREDENTIAL_REFS")
        .ok()
        .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
        .unwrap_or(true)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadContentPolicyMode {
    Disabled,
    Audit,
    Enforce,
    Quarantine,
}

impl UploadContentPolicyMode {
    pub fn from_env() -> Self {
        let raw = std::env::var("SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE")
            .or_else(|_| std::env::var("SDKWORK_DRIVE_CONTENT_SCAN_MODE"))
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .unwrap_or_default();
        match raw.as_str() {
            "disabled" | "off" | "none" => Self::Disabled,
            "quarantine" => Self::Quarantine,
            "enforce" | "block" => Self::Enforce,
            _ if is_production_runtime_profile() => Self::Enforce,
            _ => Self::Audit,
        }
    }
}

pub type ContentScanMode = UploadContentPolicyMode;

pub fn is_blocked_upload_content_type(content_type: &str) -> bool {
    let normalized = content_type.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "application/x-msdownload"
            | "application/x-msdos-program"
            | "application/x-dosexec"
            | "application/x-executable"
            | "application/x-sh"
            | "application/x-bat"
            | "application/vnd.microsoft.portable-executable"
            | "application/hta"
    ) || normalized.starts_with("application/x-ms-")
}
