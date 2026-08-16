use crate::ports::storage_object_store::{
    DownloadSignCommand, DriveDownloadSigner, DriveStorageObjectStore,
};
use crate::DriveServiceError;
use sdkwork_utils_rust::{hmac_sha256, secure_compare};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

const DOWNLOAD_TOKEN_PREFIX: &str = "dlv2_";
const DEV_DOWNLOAD_TOKEN_SECRET: &str = "sdkwork-drive-dev-download-token-secret";

#[derive(Debug, Clone)]
pub struct CreateDownloadUrlCommand {
    pub tenant_id: String,
    pub node_id: String,
    pub requested_ttl_seconds: u32,
    pub request_base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateDownloadUrlResult {
    pub download_url: String,
    pub expires_at_epoch_ms: i64,
    pub method: String,
    pub signed_source_url: String,
}

#[derive(Debug, Clone)]
pub struct ResolveDownloadTokenCommand {
    pub tenant_id: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveDownloadTokenResult {
    pub node_id: String,
    pub expires_at_epoch_ms: i64,
    pub method: String,
    pub signed_source_url: String,
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DriveDownloadService<S, G>
where
    S: DriveStorageObjectStore,
    G: DriveDownloadSigner,
{
    storage_object_store: S,
    signer: G,
}

impl<S, G> DriveDownloadService<S, G>
where
    S: DriveStorageObjectStore,
    G: DriveDownloadSigner,
{
    pub fn new(storage_object_store: S, signer: G) -> Self {
        Self {
            storage_object_store,
            signer,
        }
    }

    pub async fn create_download_url(
        &self,
        command: CreateDownloadUrlCommand,
    ) -> Result<CreateDownloadUrlResult, DriveServiceError> {
        if command.tenant_id.trim().is_empty() || command.node_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "tenant_id and node_id are required".to_string(),
            ));
        }
        if command.requested_ttl_seconds < 30 || command.requested_ttl_seconds > 300 {
            return Err(DriveServiceError::Validation(
                "requested_ttl_seconds must be between 30 and 300 seconds".to_string(),
            ));
        }

        let object = self
            .storage_object_store
            .find_latest_active_by_node(&command.tenant_id, &command.node_id)
            .await?
            .ok_or_else(|| {
                DriveServiceError::NotFound(
                    "storage object for node is not found or inactive".to_string(),
                )
            })?;

        let now_epoch_ms = now_epoch_ms();
        let expires_at_epoch_ms = now_epoch_ms + i64::from(command.requested_ttl_seconds) * 1000;

        let signed = self
            .signer
            .sign_download(DownloadSignCommand {
                storage_provider_id: object.storage_provider_id,
                bucket: object.bucket,
                object_key: object.object_key,
                expires_at_epoch_ms,
            })
            .await?;

        let token = build_download_token(
            command.tenant_id.trim(),
            command.node_id.trim(),
            expires_at_epoch_ms,
        )?;
        let base = command.request_base_url.trim_end_matches('/');
        let download_url = format!("{base}/download_tokens/{token}");

        Ok(CreateDownloadUrlResult {
            download_url,
            expires_at_epoch_ms: signed.expires_at_epoch_ms,
            method: signed.method,
            signed_source_url: signed.raw_url,
        })
    }

    pub async fn resolve_download_token(
        &self,
        command: ResolveDownloadTokenCommand,
    ) -> Result<ResolveDownloadTokenResult, DriveServiceError> {
        if command.tenant_id.trim().is_empty() || command.token.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "tenant_id and token are required".to_string(),
            ));
        }

        let parsed =
            parse_download_token_for_tenant(command.token.trim(), command.tenant_id.trim())?;
        let now_epoch_ms = now_epoch_ms();
        if parsed.expires_at_epoch_ms <= now_epoch_ms {
            return Err(DriveServiceError::NotFound(
                "download token has expired".to_string(),
            ));
        }
        if parsed.expires_at_epoch_ms > now_epoch_ms + 300_000 {
            return Err(DriveServiceError::Validation(
                "download token lifetime is invalid".to_string(),
            ));
        }

        let object = self
            .storage_object_store
            .find_latest_active_by_node(command.tenant_id.trim(), &parsed.node_id)
            .await?
            .ok_or_else(|| {
                DriveServiceError::NotFound(
                    "storage object for node is not found or inactive".to_string(),
                )
            })?;

        let signed = self
            .signer
            .sign_download(DownloadSignCommand {
                storage_provider_id: object.storage_provider_id,
                bucket: object.bucket,
                object_key: object.object_key,
                expires_at_epoch_ms: parsed.expires_at_epoch_ms,
            })
            .await?;

        Ok(ResolveDownloadTokenResult {
            node_id: parsed.node_id,
            expires_at_epoch_ms: signed.expires_at_epoch_ms,
            method: signed.method,
            signed_source_url: signed.raw_url,
            headers: signed.headers,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDownloadToken {
    pub tenant_id: String,
    pub node_id: String,
    pub expires_at_epoch_ms: i64,
}

pub fn build_download_token(
    tenant_id: &str,
    node_id: &str,
    expires_at_epoch_ms: i64,
) -> Result<String, DriveServiceError> {
    let signing_key = resolve_download_token_signing_key(tenant_id)?;
    let tenant_hex = encode_hex(tenant_id);
    let node_hex = encode_hex(node_id);
    let signature =
        sign_download_token_payload(&signing_key, tenant_id, node_id, expires_at_epoch_ms)?;
    Ok(format!(
        "{DOWNLOAD_TOKEN_PREFIX}{tenant_hex}_{node_hex}_{expires_at_epoch_ms}_{signature}"
    ))
}

pub fn parse_download_token_for_tenant(
    token: &str,
    tenant_id: &str,
) -> Result<ParsedDownloadToken, DriveServiceError> {
    let token_tenant_id = peek_download_token_tenant_id(token)?;
    let signing_key = resolve_download_token_signing_key(&token_tenant_id)?;
    let parsed = parse_download_token_with_key(token, &signing_key)?;
    if parsed.tenant_id != tenant_id.trim() {
        return Err(DriveServiceError::NotFound(
            "download token is not found".to_string(),
        ));
    }
    Ok(parsed)
}

fn peek_download_token_tenant_id(token: &str) -> Result<String, DriveServiceError> {
    let Some(suffix) = token.strip_prefix(DOWNLOAD_TOKEN_PREFIX) else {
        return Err(DriveServiceError::Validation(
            "download token format is invalid".to_string(),
        ));
    };
    let Some((head, _signature)) = suffix.rsplit_once('_') else {
        return Err(DriveServiceError::Validation(
            "download token format is invalid".to_string(),
        ));
    };
    let Some((head, _expires_raw)) = head.rsplit_once('_') else {
        return Err(DriveServiceError::Validation(
            "download token format is invalid".to_string(),
        ));
    };
    let Some((tenant_hex, _node_hex)) = head.split_once('_') else {
        return Err(DriveServiceError::Validation(
            "download token format is invalid".to_string(),
        ));
    };

    let tenant_id = decode_hex(tenant_hex)?;
    if tenant_id.trim().is_empty() {
        return Err(DriveServiceError::Validation(
            "download token payload is invalid".to_string(),
        ));
    }
    Ok(tenant_id)
}

fn parse_download_token_with_key(
    token: &str,
    signing_key: &str,
) -> Result<ParsedDownloadToken, DriveServiceError> {
    let Some(suffix) = token.strip_prefix(DOWNLOAD_TOKEN_PREFIX) else {
        return Err(DriveServiceError::Validation(
            "download token format is invalid".to_string(),
        ));
    };
    let Some((head, signature)) = suffix.rsplit_once('_') else {
        return Err(DriveServiceError::Validation(
            "download token format is invalid".to_string(),
        ));
    };
    let Some((head, expires_raw)) = head.rsplit_once('_') else {
        return Err(DriveServiceError::Validation(
            "download token format is invalid".to_string(),
        ));
    };
    let Some((tenant_hex, node_hex)) = head.split_once('_') else {
        return Err(DriveServiceError::Validation(
            "download token format is invalid".to_string(),
        ));
    };

    let tenant_id = decode_hex(tenant_hex)?;
    let node_id = decode_hex(node_hex)?;
    if tenant_id.trim().is_empty() || node_id.trim().is_empty() {
        return Err(DriveServiceError::Validation(
            "download token payload is invalid".to_string(),
        ));
    }

    let expires_at_epoch_ms = expires_raw.parse::<i64>().map_err(|_| {
        DriveServiceError::Validation("download token expiration is invalid".to_string())
    })?;

    let expected_signature =
        sign_download_token_payload(signing_key, &tenant_id, &node_id, expires_at_epoch_ms)?;
    if !secure_compare(signature, &expected_signature) {
        return Err(DriveServiceError::Validation(
            "download token signature is invalid".to_string(),
        ));
    }

    Ok(ParsedDownloadToken {
        tenant_id,
        node_id,
        expires_at_epoch_ms,
    })
}

fn sign_download_token_payload(
    signing_key: &str,
    tenant_id: &str,
    node_id: &str,
    expires_at_epoch_ms: i64,
) -> Result<String, DriveServiceError> {
    let payload = format!("{tenant_id}\n{node_id}\n{expires_at_epoch_ms}");
    if signing_key.trim().is_empty() {
        return Err(DriveServiceError::Internal(
            "download token signing key is invalid".to_string(),
        ));
    }
    Ok(hmac_sha256(payload.as_bytes(), signing_key.as_bytes()))
}

fn resolve_download_token_signing_key(tenant_id: &str) -> Result<String, DriveServiceError> {
    let tenant_id = tenant_id.trim();
    if tenant_id.is_empty() {
        return Err(DriveServiceError::Validation(
            "tenant_id is required for download token signing".to_string(),
        ));
    }
    if let Ok(secret) = std::env::var("SDKWORK_DRIVE_DOWNLOAD_TOKEN_HMAC_SECRET") {
        let secret = secret.trim().to_string();
        if !secret.is_empty() {
            return Ok(secret);
        }
    }
    if let Some(secret) = resolve_tenant_secret_from_json_env(
        "SDKWORK_DRIVE_DOWNLOAD_TOKEN_HMAC_SECRETS_JSON",
        tenant_id,
    )? {
        return Ok(secret);
    }
    if let Some(secret) =
        resolve_tenant_secret_from_json_env("SDKWORK_DRIVE_JWT_HMAC_SECRETS_JSON", tenant_id)?
    {
        return Ok(secret);
    }
    if let Ok(secret) = std::env::var("SDKWORK_DRIVE_JWT_HMAC_SECRET") {
        let secret = secret.trim().to_string();
        if !secret.is_empty() {
            return Ok(secret);
        }
    }
    if is_production_runtime_profile() {
        return Err(DriveServiceError::Internal(
            "SDKWORK_DRIVE_DOWNLOAD_TOKEN_HMAC_SECRET is required in production".to_string(),
        ));
    }
    Ok(DEV_DOWNLOAD_TOKEN_SECRET.to_string())
}

fn resolve_tenant_secret_from_json_env(
    env_key: &str,
    tenant_id: &str,
) -> Result<Option<String>, DriveServiceError> {
    let Ok(raw) = std::env::var(env_key) else {
        return Ok(None);
    };
    let secrets: std::collections::BTreeMap<String, String> = serde_json::from_str(raw.trim())
        .map_err(|error| DriveServiceError::Internal(format!("{env_key} is invalid: {error}")))?;
    if let Some(secret) = secrets
        .get(tenant_id)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        return Ok(Some(secret.to_string()));
    }
    if let Some(secret) = secrets
        .get("default")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        return Ok(Some(secret.to_string()));
    }
    Ok(None)
}

fn is_production_runtime_profile() -> bool {
    std::env::var("SDKWORK_DRIVE_RUNTIME_PROFILE")
        .ok()
        .map(|value| {
            let value = value.trim().to_ascii_lowercase();
            value == "production" || value == "prod"
        })
        .unwrap_or(false)
}

/// Fail fast when production is configured without download-token signing secrets.
pub fn ensure_production_download_token_signing_configured() -> Result<(), String> {
    if !is_production_runtime_profile() {
        return Ok(());
    }

    if std::env::var("SDKWORK_DRIVE_DOWNLOAD_TOKEN_HMAC_SECRET")
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return Ok(());
    }

    for env_key in [
        "SDKWORK_DRIVE_DOWNLOAD_TOKEN_HMAC_SECRETS_JSON",
        "SDKWORK_DRIVE_JWT_HMAC_SECRETS_JSON",
    ] {
        if let Ok(raw) = std::env::var(env_key) {
            let secrets: std::collections::BTreeMap<String, String> =
                serde_json::from_str(raw.trim())
                    .map_err(|error| format!("{env_key} is invalid in production: {error}"))?;
            if secrets.values().any(|value| !value.trim().is_empty()) {
                return Ok(());
            }
        }
    }

    if std::env::var("SDKWORK_DRIVE_JWT_HMAC_SECRET")
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return Ok(());
    }

    Err(
        "production runtime requires SDKWORK_DRIVE_DOWNLOAD_TOKEN_HMAC_SECRET or tenant-scoped download/JWT signing secrets"
            .to_string(),
    )
}

fn encode_hex(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn decode_hex(encoded: &str) -> Result<String, DriveServiceError> {
    if encoded.is_empty() || !encoded.len().is_multiple_of(2) {
        return Err(DriveServiceError::Validation(
            "download token payload is invalid".to_string(),
        ));
    }

    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    let mut index = 0;
    while index < encoded.len() {
        let pair = &encoded[index..index + 2];
        let byte = u8::from_str_radix(pair, 16).map_err(|_| {
            DriveServiceError::Validation("download token payload is invalid".to_string())
        })?;
        bytes.push(byte);
        index += 2;
    }

    String::from_utf8(bytes)
        .map_err(|_| DriveServiceError::Validation("download token payload is invalid".to_string()))
}

fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_download_token_round_trips_for_tenant() {
        let token =
            build_download_token("tenant-001", "node-001", 1_700_000_000_000).expect("token");
        assert!(token.starts_with(DOWNLOAD_TOKEN_PREFIX));
        let parsed = parse_download_token_for_tenant(&token, "tenant-001").expect("parsed token");
        assert_eq!(parsed.node_id, "node-001");
        assert_eq!(parsed.tenant_id, "tenant-001");
        assert_eq!(parsed.expires_at_epoch_ms, 1_700_000_000_000);
    }

    #[test]
    fn forged_download_token_is_rejected() {
        let token =
            build_download_token("tenant-001", "node-001", 1_700_000_000_000).expect("token");
        let mut forged = token;
        forged.pop();
        forged.push('0');
        let error = parse_download_token_for_tenant(&forged, "tenant-001").expect_err("forged");
        assert!(matches!(error, DriveServiceError::Validation(_)));
    }

    #[test]
    fn download_token_rejects_cross_tenant_resolution() {
        let token =
            build_download_token("tenant-001", "node-001", 1_700_000_000_000).expect("token");
        let error = parse_download_token_for_tenant(&token, "tenant-002").expect_err("tenant");
        assert!(matches!(error, DriveServiceError::NotFound(_)));
    }

    #[test]
    fn download_token_uses_tenant_specific_signing_secret_when_configured() {
        let download_secret_env = "SDKWORK_DRIVE_DOWNLOAD_TOKEN_HMAC_SECRET";
        let download_secrets_json_env = "SDKWORK_DRIVE_DOWNLOAD_TOKEN_HMAC_SECRETS_JSON";
        let previous_download_secret = std::env::var(download_secret_env).ok();
        let previous_download_secrets_json = std::env::var(download_secrets_json_env).ok();
        std::env::remove_var(download_secret_env);
        std::env::set_var(
            download_secrets_json_env,
            r#"{"tenant-a":"tenant-a-download-secret"}"#,
        );

        let token = build_download_token("tenant-a", "node-001", 1_700_000_000_000).expect("token");
        assert!(parse_download_token_for_tenant(&token, "tenant-a").is_ok());
        let error = parse_download_token_for_tenant(&token, "tenant-b").expect_err("tenant");
        assert!(matches!(error, DriveServiceError::NotFound(_)));

        match previous_download_secret {
            Some(value) => std::env::set_var(download_secret_env, value),
            None => std::env::remove_var(download_secret_env),
        }
        match previous_download_secrets_json {
            Some(value) => std::env::set_var(download_secrets_json_env, value),
            None => std::env::remove_var(download_secrets_json_env),
        }
    }
}
