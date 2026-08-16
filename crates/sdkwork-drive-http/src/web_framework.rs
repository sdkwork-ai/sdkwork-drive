use sdkwork_web_core::{SecurityPolicy, WebEnvironment};

const DRIVE_ENVIRONMENT_KEYS: &[&str] = &[
    "SDKWORK_ENVIRONMENT",
    "SDKWORK_DRIVE_ENVIRONMENT",
    "SDKWORK_DRIVE_STANDALONE_GATEWAY_ENVIRONMENT",
    "SDKWORK_ENV",
];
const DRIVE_CORS_ALLOWED_ORIGIN_KEYS: &[&str] = &["SDKWORK_CORS_ALLOWED_ORIGINS"];

/// Resolve the canonical SDKWork web environment for Drive HTTP services.
pub fn resolve_drive_web_environment_from_process_env() -> WebEnvironment {
    sdkwork_web_bootstrap::web_environment_from_env(DRIVE_ENVIRONMENT_KEYS)
}

/// Drive HTTP service security policy derived from the shared Web Framework configuration.
pub fn drive_service_security_policy(environment: &WebEnvironment) -> SecurityPolicy {
    let origins =
        sdkwork_web_bootstrap::cors_allowed_origins_from_env(DRIVE_CORS_ALLOWED_ORIGIN_KEYS);
    drive_service_security_policy_with_origins(environment, origins)
}

fn drive_service_security_policy_with_origins(
    environment: &WebEnvironment,
    origins: impl IntoIterator<Item = String>,
) -> SecurityPolicy {
    sdkwork_web_bootstrap::security_policy_for_environment(environment, origins)
}

#[cfg(test)]
mod tests {
    use super::{
        drive_service_security_policy, drive_service_security_policy_with_origins,
        resolve_drive_web_environment_from_process_env,
    };
    use sdkwork_web_core::WebEnvironment;

    #[test]
    fn dev_security_policy_allows_private_network_browser_origins() {
        let policy = drive_service_security_policy(&WebEnvironment::Dev);
        assert!(!policy.cors.allow_all_origins);
        policy
            .cors
            .validate_origin_value("http://127.0.0.1:5190")
            .expect("loopback development origin");
    }

    #[test]
    fn production_security_policy_rejects_permissive_cors() {
        let policy = drive_service_security_policy(&WebEnvironment::Prod);
        assert!(!policy.cors.allow_all_origins);
    }

    #[test]
    fn production_security_policy_accepts_configured_exact_origin() {
        let policy = drive_service_security_policy_with_origins(
            &WebEnvironment::Prod,
            ["https://manager.sdkwork.com".to_owned()],
        );
        policy
            .cors
            .validate_origin_value("https://manager.sdkwork.com")
            .expect("configured Manager origin");
        policy
            .cors
            .validate_origin_value("https://evil.example.com")
            .expect_err("unconfigured origin");
    }

    #[test]
    fn resolve_environment_from_shared_host_projection() {
        unsafe {
            std::env::set_var("SDKWORK_ENVIRONMENT", "development");
        }
        assert_eq!(
            resolve_drive_web_environment_from_process_env(),
            WebEnvironment::Dev
        );
        unsafe {
            std::env::remove_var("SDKWORK_ENVIRONMENT");
        }
    }
}
