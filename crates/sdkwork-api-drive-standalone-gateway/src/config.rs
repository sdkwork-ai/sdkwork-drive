use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayFileConfig {
    pub service: ServiceSection,
    pub server: ServerSection,
    pub cors: CorsSection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceSection {
    pub name: String,
    pub environment: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSection {
    pub bind: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorsSection {
    #[serde(default)]
    pub allow_any_origin: bool,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedGatewayConfig {
    pub service_name: String,
    pub environment: String,
    pub bind: String,
    pub allowed_origins: Vec<String>,
}

pub fn load_gateway_config(config_path: &Path) -> Result<GatewayFileConfig, String> {
    let raw = fs::read_to_string(config_path).map_err(|error| {
        format!(
            "failed to read standalone gateway config {}: {error}",
            config_path.display()
        )
    })?;
    toml::from_str(&raw).map_err(|error| {
        format!(
            "failed to parse standalone gateway config {}: {error}",
            config_path.display()
        )
    })
}

pub fn resolve_gateway_config(
    file_config: GatewayFileConfig,
) -> Result<ResolvedGatewayConfig, String> {
    if file_config
        .service
        .environment
        .eq_ignore_ascii_case("production")
        && file_config.cors.allow_any_origin
    {
        return Err(
            "standalone gateway production profile must not enable cors.allowAnyOrigin".to_string(),
        );
    }

    if is_production_runtime_profile() && file_config.cors.allow_any_origin {
        return Err(
            "SDKWORK_DRIVE_RUNTIME_PROFILE=production rejects cors.allowAnyOrigin on standalone gateway"
                .to_string(),
        );
    }

    Ok(ResolvedGatewayConfig {
        service_name: file_config.service.name,
        environment: file_config.service.environment,
        bind: read_env_override("SDKWORK_DRIVE_STANDALONE_GATEWAY_BIND")
            .unwrap_or(file_config.server.bind),
        allowed_origins: file_config.cors.allowed_origins,
    })
}

pub fn web_framework_env_projection(config: &ResolvedGatewayConfig) -> [(&'static str, String); 2] {
    [
        ("SDKWORK_ENVIRONMENT", config.environment.clone()),
        (
            "SDKWORK_CORS_ALLOWED_ORIGINS",
            config.allowed_origins.join(","),
        ),
    ]
}

fn read_env_override(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_end_matches('/').to_string())
}

fn is_production_runtime_profile() -> bool {
    std::env::var("SDKWORK_DRIVE_RUNTIME_PROFILE")
        .ok()
        .map(|value| value.trim().eq_ignore_ascii_case("production"))
        .unwrap_or(false)
}

pub fn resolve_config_path(args: &[String]) -> Result<String, String> {
    if let Ok(path) = std::env::var("SDKWORK_DRIVE_STANDALONE_GATEWAY_CONFIG") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    for (index, arg) in args.iter().enumerate() {
        if arg == "--config" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| "--config requires a path".to_string())?;
            return Ok(value.clone());
        }
    }

    let environment = std::env::var("SDKWORK_DRIVE_STANDALONE_GATEWAY_ENVIRONMENT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "development".to_string());

    Ok(format!(
        "etc/sdkwork-api-drive-standalone-gateway.{environment}.toml"
    ))
}

#[cfg(test)]
mod tests {
    use super::{web_framework_env_projection, ResolvedGatewayConfig};

    #[test]
    fn projects_source_config_to_shared_web_framework_keys() {
        let config = ResolvedGatewayConfig {
            service_name: "sdkwork-api-drive-standalone-gateway".to_owned(),
            environment: "production".to_owned(),
            bind: "127.0.0.1:3900".to_owned(),
            allowed_origins: vec![
                "https://manager.sdkwork.com".to_owned(),
                "https://apps.sdkwork.com".to_owned(),
            ],
        };

        assert_eq!(
            web_framework_env_projection(&config),
            [
                ("SDKWORK_ENVIRONMENT", "production".to_owned()),
                (
                    "SDKWORK_CORS_ALLOWED_ORIGINS",
                    "https://manager.sdkwork.com,https://apps.sdkwork.com".to_owned(),
                ),
            ],
        );
    }
}
