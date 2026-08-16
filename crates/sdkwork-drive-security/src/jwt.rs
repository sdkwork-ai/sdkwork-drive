use crate::policy::DriveAuthValidationPolicy;
use crate::token_claims::{decode_base64url, parse_json_claims, TokenClaimsError};
use jsonwebtoken::{decode, Algorithm, Validation};
#[cfg(test)]
use sdkwork_utils_rust::hmac_sha256_base64url;
use sdkwork_utils_rust::verify_hmac_sha256_base64url;
use serde_json::Value;
use std::collections::BTreeMap;

pub fn is_jwt_format(raw: &str) -> bool {
    if raw.contains('=') || raw.starts_with('{') {
        return false;
    }
    let mut parts = raw.split('.');
    let header = parts.next().unwrap_or_default();
    let payload = parts.next().unwrap_or_default();
    let signature = parts.next().unwrap_or_default();
    !header.is_empty() && !payload.is_empty() && !signature.is_empty() && parts.next().is_none()
}

pub fn verify_jwt_hmac_claims(
    raw: &str,
    policy: &DriveAuthValidationPolicy,
) -> Result<BTreeMap<String, String>, TokenClaimsError> {
    let raw = raw.trim();
    let mut parts = raw.split('.');
    let header_b64 = parts.next().ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT header segment is required".to_string())
    })?;
    let payload_b64 = parts.next().ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT payload segment is required".to_string())
    })?;
    let signature_b64 = parts.next().ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT signature segment is required".to_string())
    })?;
    if parts.next().is_some() {
        return Err(TokenClaimsError::InvalidCredentials(
            "JWT must contain exactly three segments".to_string(),
        ));
    }

    let kid = jwt_header_kid(header_b64);
    // Use a generic error message that does not reveal whether the kid is configured.
    let secret = policy
        .resolve_jwt_hmac_secret(kid.as_deref())
        .ok_or_else(|| {
            TokenClaimsError::InvalidCredentials("JWT signature verification failed".to_string())
        })?;

    let signing_input = format!("{header_b64}.{payload_b64}");
    let actual = decode_base64url(signature_b64).ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT signature is not valid base64url".to_string())
    })?;
    if !verify_hmac_sha256_base64url(signing_input.as_bytes(), secret.as_bytes(), &actual) {
        return Err(TokenClaimsError::InvalidCredentials(
            "JWT signature verification failed".to_string(),
        ));
    }

    let payload_bytes = decode_base64url(payload_b64).ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT payload is not valid base64url".to_string())
    })?;
    let payload = std::str::from_utf8(&payload_bytes).map_err(|error| {
        TokenClaimsError::InvalidCredentials(format!("JWT payload must be UTF-8: {error}"))
    })?;
    let claims = parse_json_claims(payload)?;
    validate_jwt_expiry(&claims)?;
    validate_jwt_tenant_kid_binding(kid.as_deref(), &claims)?;
    Ok(claims)
}

pub fn verify_jwt_jwks_claims(
    raw: &str,
    policy: &DriveAuthValidationPolicy,
) -> Result<BTreeMap<String, String>, TokenClaimsError> {
    let raw = raw.trim();
    let mut parts = raw.split('.');
    let header_b64 = parts.next().ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT header segment is required".to_string())
    })?;
    let _payload_b64 = parts.next().ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT payload segment is required".to_string())
    })?;
    if parts.next().is_some() {
        return Err(TokenClaimsError::InvalidCredentials(
            "JWT must contain exactly three segments".to_string(),
        ));
    }

    let alg = jwt_header_alg(header_b64).ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT alg header is required".to_string())
    })?;
    if alg != "RS256" {
        return Err(TokenClaimsError::InvalidCredentials(format!(
            "unsupported JWT alg for JWKS verification: {alg}"
        )));
    }

    let kid = jwt_header_kid(header_b64);
    // Use a generic error message that does not reveal whether the kid is configured.
    let decoding_key = policy.resolve_jwt_jwks_key(kid.as_deref()).ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT signature verification failed".to_string())
    })?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;
    let token_data = decode::<Value>(raw, decoding_key, &validation).map_err(|error| {
        TokenClaimsError::InvalidCredentials(format!("JWT signature verification failed: {error}"))
    })?;
    let claims = json_value_to_claim_map(&token_data.claims)?;
    validate_jwt_expiry(&claims)?;
    validate_jwt_tenant_kid_binding(kid.as_deref(), &claims)?;
    Ok(claims)
}

pub fn jwt_header_alg(header_b64: &str) -> Option<String> {
    let decoded = decode_base64url(header_b64)?;
    let decoded = std::str::from_utf8(&decoded).ok()?;
    let value = serde_json::from_str::<Value>(decoded).ok()?;
    value
        .get("alg")
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn json_value_to_claim_map(value: &Value) -> Result<BTreeMap<String, String>, TokenClaimsError> {
    let object = value.as_object().ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("JWT payload must be a JSON object".to_string())
    })?;
    let mut claims = BTreeMap::new();
    for (key, value) in object {
        let normalized = match value {
            Value::String(text) => text.clone(),
            Value::Number(number) => number.to_string(),
            Value::Bool(flag) => flag.to_string(),
            Value::Null => continue,
            _ => serde_json::to_string(value).map_err(|error| {
                TokenClaimsError::InvalidCredentials(format!(
                    "JWT claim {key} could not be normalized: {error}"
                ))
            })?,
        };
        if !normalized.is_empty() {
            claims.insert(key.clone(), normalized);
        }
    }
    Ok(claims)
}

fn jwt_header_kid(header_b64: &str) -> Option<String> {
    let decoded = decode_base64url(header_b64)?;
    let decoded = std::str::from_utf8(&decoded).ok()?;
    let value = serde_json::from_str::<Value>(decoded).ok()?;
    value
        .get("kid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

/// Maximum allowed clock skew (in seconds) between servers.
/// Tokens with `exp` within this range from `now` may still be accepted.
const JWT_CLOCK_SKEW_LEEWAY_SECONDS: i64 = 30;

fn validate_jwt_expiry(claims: &BTreeMap<String, String>) -> Result<(), TokenClaimsError> {
    let Some(exp) = claims.get("exp") else {
        return Err(TokenClaimsError::InvalidCredentials(
            "JWT exp claim is required".to_string(),
        ));
    };
    let exp = exp.trim().parse::<i64>().map_err(|error| {
        TokenClaimsError::InvalidCredentials(format!("invalid exp claim: {error}"))
    })?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| {
            TokenClaimsError::InvalidCredentials(format!("system clock error: {error}"))
        })?
        .as_secs() as i64;
    // Allow clock skew up to JWT_CLOCK_SKEW_LEEWAY_SECONDS so that
    // tokens issued by a server with a slightly faster clock are not
    // rejected prematurely.
    if exp <= now - JWT_CLOCK_SKEW_LEEWAY_SECONDS {
        return Err(TokenClaimsError::InvalidCredentials(
            "JWT exp claim is in the past".to_string(),
        ));
    }
    Ok(())
}

fn validate_jwt_tenant_kid_binding(
    kid: Option<&str>,
    claims: &BTreeMap<String, String>,
) -> Result<(), TokenClaimsError> {
    let Some(kid) = kid.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    if kid == "default" {
        return Ok(());
    }
    let Some(tenant_id) = claims
        .get("tenant_id")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return Err(TokenClaimsError::InvalidCredentials(
            "JWT tenant_id claim is required for tenant-bound kid".to_string(),
        ));
    };
    if tenant_id != kid {
        return Err(TokenClaimsError::InvalidCredentials(
            "JWT kid does not match tenant_id claim".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
pub fn encode_test_jwt_hmac(payload_json: &str, secret: &str) -> String {
    encode_test_jwt_hmac_with_kid(payload_json, secret, None)
}

#[cfg(test)]
pub fn encode_test_jwt_hmac_with_kid(
    payload_json: &str,
    secret: &str,
    kid: Option<&str>,
) -> String {
    let header = match kid {
        Some(kid) => encode_base64url(&format!(r#"{{"alg":"HS256","typ":"JWT","kid":"{kid}"}}"#)),
        None => encode_base64url(r#"{"alg":"HS256","typ":"JWT"}"#),
    };
    let payload = encode_base64url(payload_json);
    let signing_input = format!("{header}.{payload}");
    let signature = hmac_sha256_base64url(signing_input.as_bytes(), secret.as_bytes());
    format!("{signing_input}.{signature}")
}

#[cfg(test)]
fn encode_base64url(input: &str) -> String {
    encode_base64url_bytes(input.as_bytes())
}

#[cfg(test)]
fn encode_base64url_bytes(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::new();
    for chunk in input.chunks(3) {
        let mut buf = [0u8; 3];
        for (target, source) in buf.iter_mut().zip(chunk.iter()) {
            *target = *source;
        }
        let triple = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        output.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        }
        if chunk.len() > 2 {
            output.push(TABLE[(triple & 0x3f) as usize] as char);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_treat_dotted_inline_permission_scopes_as_jwt() {
        assert!(!is_jwt_format(
            "tenant_id=tenant-a;user_id=admin-001;permission_scope=drive.storage.admin"
        ));
    }

    #[test]
    fn verifies_hmac_signed_jwt() {
        let payload =
            r#"{"tenant_id":"tenant-a","user_id":"user-001","app_id":"appbase","exp":4102444800}"#;
        let token = encode_test_jwt_hmac(payload, "test-secret");
        let policy = DriveAuthValidationPolicy::require_signed_projection("test-secret");
        let claims = verify_jwt_hmac_claims(&token, &policy).expect("claims");
        assert_eq!(claims.get("tenant_id"), Some(&"tenant-a".to_string()));
        assert_eq!(claims.get("user_id"), Some(&"user-001".to_string()));
    }

    #[test]
    fn verifies_hmac_signed_jwt_with_kid_rotation_secret() {
        let payload =
            r#"{"tenant_id":"tenant-a","user_id":"user-001","app_id":"appbase","exp":4102444800}"#;
        let token = encode_test_jwt_hmac_with_kid(payload, "tenant-a-secret", Some("tenant-a"));
        let policy = DriveAuthValidationPolicy {
            allow_inline_claim_tokens: false,
            jwt_hmac_secrets: BTreeMap::from([
                ("default".to_string(), "default-secret".to_string()),
                ("tenant-a".to_string(), "tenant-a-secret".to_string()),
            ]),
            jwt_jwks_keys: BTreeMap::new(),
        };
        let claims = verify_jwt_hmac_claims(&token, &policy).expect("claims");
        assert_eq!(claims.get("tenant_id"), Some(&"tenant-a".to_string()));
    }

    #[test]
    fn rejects_invalid_signature() {
        let payload = r#"{"tenant_id":"tenant-a","user_id":"user-001","exp":4102444800}"#;
        let token = encode_test_jwt_hmac(payload, "test-secret");
        let policy = DriveAuthValidationPolicy::require_signed_projection("wrong-secret");
        let error = verify_jwt_hmac_claims(&token, &policy).expect_err("mismatch");
        assert!(matches!(error, TokenClaimsError::InvalidCredentials(_)));
    }

    #[test]
    fn rejects_jwt_without_exp_claim() {
        let payload = r#"{"tenant_id":"tenant-a","user_id":"user-001","app_id":"appbase"}"#;
        let token = encode_test_jwt_hmac(payload, "test-secret");
        let policy = DriveAuthValidationPolicy::require_signed_projection("test-secret");
        let error = verify_jwt_hmac_claims(&token, &policy).expect_err("missing exp");
        assert!(matches!(error, TokenClaimsError::InvalidCredentials(_)));
    }

    #[test]
    fn rejects_jwt_when_kid_does_not_match_tenant_id_claim() {
        let payload =
            r#"{"tenant_id":"tenant-b","user_id":"user-001","app_id":"appbase","exp":4102444800}"#;
        let token = encode_test_jwt_hmac_with_kid(payload, "tenant-a-secret", Some("tenant-a"));
        let policy = DriveAuthValidationPolicy {
            allow_inline_claim_tokens: false,
            jwt_hmac_secrets: BTreeMap::from([(
                "tenant-a".to_string(),
                "tenant-a-secret".to_string(),
            )]),
            jwt_jwks_keys: BTreeMap::new(),
        };
        let error = verify_jwt_hmac_claims(&token, &policy).expect_err("kid mismatch");
        assert!(matches!(error, TokenClaimsError::InvalidCredentials(_)));
    }
}
