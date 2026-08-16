use crate::jwt::{is_jwt_format, jwt_header_alg, verify_jwt_hmac_claims, verify_jwt_jwks_claims};
use crate::policy::DriveAuthValidationPolicy;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDualTokenContext {
    pub tenant_id: String,
    pub user_id: String,
    pub organization_id: Option<String>,
    pub session_id: Option<String>,
    pub app_id: String,
    pub environment: String,
    pub deployment_mode: String,
    pub auth_level: String,
    pub data_scope: Vec<String>,
    pub permission_scope: Vec<String>,
    pub actor_id: String,
    pub actor_kind: String,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenClaimsError {
    MissingField(String),
    InvalidCredentials(String),
    Forbidden(String),
}

pub fn resolve_dual_token_context(
    raw_auth_token: &str,
    raw_access_token: &str,
    policy: &DriveAuthValidationPolicy,
) -> Result<ResolvedDualTokenContext, TokenClaimsError> {
    let auth = parse_auth_token_claims(raw_auth_token, policy)?;
    let access = parse_access_token_claims(raw_access_token, policy)?;

    require_optional_match(
        "tenant_id",
        auth.tenant_id.as_deref(),
        Some(access.tenant_id.as_str()),
    )?;
    require_optional_match(
        "organization_id",
        auth.organization_id.as_deref(),
        access.organization_id.as_deref(),
    )?;
    require_optional_match(
        "user_id",
        Some(auth.user_id.as_str()),
        access.user_id.as_deref(),
    )?;
    require_optional_match(
        "session_id",
        auth.session_id.as_deref(),
        access.session_id.as_deref(),
    )?;
    require_optional_match(
        "app_id",
        auth.app_id.as_deref(),
        Some(access.app_id.as_str()),
    )?;
    if auth.login_scope != access.login_scope {
        return Err(TokenClaimsError::Forbidden(
            "auth token and access token login_scope contexts do not match".to_string(),
        ));
    }

    let actor_kind = auth
        .subject_type
        .as_deref()
        .or(access.subject_type.as_deref())
        .unwrap_or("user")
        .to_string();

    // actor_id resolves in priority order: access-token subject > auth-token subject > user_id
    let actor_id = access
        .metadata
        .get("subject_id")
        .or_else(|| access.metadata.get("actor_id"))
        .or_else(|| auth.metadata.get("subject_id"))
        .or_else(|| auth.metadata.get("actor_id"))
        .cloned()
        .unwrap_or_else(|| auth.user_id.clone());

    Ok(ResolvedDualTokenContext {
        tenant_id: access.tenant_id,
        user_id: auth.user_id,
        organization_id: access.organization_id.or(auth.organization_id),
        session_id: auth.session_id.or(access.session_id),
        app_id: access.app_id,
        environment: access.environment,
        deployment_mode: access.deployment_mode,
        auth_level: auth.auth_level,
        data_scope: if access.data_scope.is_empty() {
            auth.data_scope
        } else {
            access.data_scope
        },
        permission_scope: if access.permission_scope.is_empty() {
            auth.permission_scope
        } else {
            access.permission_scope
        },
        actor_id,
        actor_kind,
        device_id: optional_claim(&access.metadata, "device_id")
            .or_else(|| optional_claim(&auth.metadata, "device_id")),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedAuthTokenClaims {
    tenant_id: Option<String>,
    organization_id: Option<String>,
    login_scope: String,
    user_id: String,
    session_id: Option<String>,
    app_id: Option<String>,
    auth_level: String,
    data_scope: Vec<String>,
    permission_scope: Vec<String>,
    subject_type: Option<String>,
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedAccessTokenClaims {
    tenant_id: String,
    organization_id: Option<String>,
    login_scope: String,
    user_id: Option<String>,
    session_id: Option<String>,
    app_id: String,
    environment: String,
    deployment_mode: String,
    data_scope: Vec<String>,
    permission_scope: Vec<String>,
    subject_type: Option<String>,
    metadata: BTreeMap<String, String>,
}

fn parse_auth_token_claims(
    raw: &str,
    policy: &DriveAuthValidationPolicy,
) -> Result<ParsedAuthTokenClaims, TokenClaimsError> {
    let claims = parse_claims(raw, policy)?;
    let organization_id = optional_claim(&claims, "organization_id");
    let login_scope = parse_login_scope(
        claims.get("login_scope").map(String::as_str),
        organization_id.as_deref(),
    )?;
    Ok(ParsedAuthTokenClaims {
        tenant_id: optional_claim(&claims, "tenant_id"),
        organization_id,
        login_scope,
        user_id: required_claim(&claims, "user_id")?,
        session_id: optional_claim(&claims, "session_id"),
        app_id: optional_claim(&claims, "app_id"),
        auth_level: optional_claim(&claims, "auth_level").unwrap_or_else(|| "password".to_string()),
        data_scope: split_claim(claims.get("data_scope")),
        permission_scope: split_claim(claims.get("permission_scope")),
        subject_type: optional_claim(&claims, "subject_type"),
        metadata: claims,
    })
}

fn parse_access_token_claims(
    raw: &str,
    policy: &DriveAuthValidationPolicy,
) -> Result<ParsedAccessTokenClaims, TokenClaimsError> {
    let claims = parse_claims(raw, policy)?;
    let organization_id = optional_claim(&claims, "organization_id");
    let login_scope = parse_login_scope(
        claims.get("login_scope").map(String::as_str),
        organization_id.as_deref(),
    )?;
    Ok(ParsedAccessTokenClaims {
        tenant_id: required_claim(&claims, "tenant_id")?,
        organization_id,
        login_scope,
        user_id: optional_claim(&claims, "user_id"),
        session_id: optional_claim(&claims, "session_id"),
        app_id: required_claim(&claims, "app_id")?,
        environment: optional_claim(&claims, "environment").unwrap_or_else(|| "prod".to_string()),
        deployment_mode: optional_claim(&claims, "deployment_mode")
            .unwrap_or_else(|| "saas".to_string()),
        data_scope: split_claim(claims.get("data_scope")),
        permission_scope: split_claim(claims.get("permission_scope")),
        subject_type: optional_claim(&claims, "subject_type"),
        metadata: claims,
    })
}

fn parse_claims(
    raw: &str,
    policy: &DriveAuthValidationPolicy,
) -> Result<BTreeMap<String, String>, TokenClaimsError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(BTreeMap::new());
    }
    if is_jwt_format(raw) {
        let header_b64 = raw.split('.').next().unwrap_or_default();
        if jwt_header_alg(header_b64).as_deref() == Some("RS256") && policy.has_jwks() {
            return verify_jwt_jwks_claims(raw, policy);
        }
        if !policy.jwt_hmac_secrets.is_empty() {
            return verify_jwt_hmac_claims(raw, policy);
        }
        if policy.requires_signed_jwt() {
            return Err(TokenClaimsError::InvalidCredentials(
                "signed JWT credentials are required".to_string(),
            ));
        }
        return parse_jwt_payload_claims(raw).ok_or_else(|| {
            TokenClaimsError::InvalidCredentials("JWT payload could not be decoded".to_string())
        });
    }
    if !policy.allow_inline_claim_tokens {
        return Err(TokenClaimsError::InvalidCredentials(
            "inline claim-string tokens are not allowed; use signed JWT credentials".to_string(),
        ));
    }
    if raw.starts_with('{') {
        return parse_json_claims(raw);
    }
    Ok(parse_key_value_claims(raw))
}

fn parse_key_value_claims(raw: &str) -> BTreeMap<String, String> {
    raw.split(';')
        .filter_map(|part| {
            let (key, value) = part.split_once('=')?;
            let key = key.trim();
            let value = value.trim();
            if key.is_empty() || value.is_empty() {
                return None;
            }
            Some((key.to_owned(), value.to_owned()))
        })
        .collect()
}

pub(crate) fn parse_json_claims(raw: &str) -> Result<BTreeMap<String, String>, TokenClaimsError> {
    let value = serde_json::from_str::<Value>(raw).map_err(|error| {
        TokenClaimsError::InvalidCredentials(format!("invalid JSON claims: {error}"))
    })?;
    let object = value.as_object().ok_or_else(|| {
        TokenClaimsError::InvalidCredentials("token claims must be a JSON object".to_string())
    })?;
    Ok(object
        .iter()
        .filter_map(|(key, value)| claim_value_to_string(value).map(|value| (key.clone(), value)))
        .collect())
}

fn parse_jwt_payload_claims(raw: &str) -> Option<BTreeMap<String, String>> {
    let mut parts = raw.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;
    let _signature = parts.next()?;
    let decoded = decode_base64url(payload)?;
    let decoded = std::str::from_utf8(&decoded).ok()?;
    parse_json_claims(decoded).ok()
}

fn claim_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) => Some(value.trim().to_owned()).filter(|value| !value.is_empty()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Array(items) => {
            let values = items
                .iter()
                .filter_map(claim_value_to_string)
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(values.join(","))
            }
        }
        Value::Object(_) => serde_json::to_string(value).ok(),
    }
}

pub(crate) fn decode_base64url(input: &str) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer: u32 = 0;
    let mut bits: u8 = 0;

    for byte in input.bytes() {
        if byte == b'=' {
            break;
        }
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            _ => return None,
        } as u32;
        buffer = (buffer << 6) | value;
        bits += 6;
        while bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 0xff) as u8);
        }
    }

    Some(output)
}

fn parse_login_scope(
    value: Option<&str>,
    organization_id: Option<&str>,
) -> Result<String, TokenClaimsError> {
    let login_scope = match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) if value.eq_ignore_ascii_case("TENANT") => "TENANT".to_string(),
        Some(value) if value.eq_ignore_ascii_case("ORGANIZATION") => "ORGANIZATION".to_string(),
        Some(value) => {
            return Err(TokenClaimsError::InvalidCredentials(format!(
                "unsupported login_scope claim: {value}",
            )));
        }
        None => match organization_id.map(str::trim) {
            Some(value) if !value.is_empty() && value != "0" => "ORGANIZATION".to_string(),
            _ => "TENANT".to_string(),
        },
    };

    match (login_scope.as_str(), organization_id.map(str::trim)) {
        ("TENANT", Some(value)) if !value.is_empty() && value != "0" => {
            Err(TokenClaimsError::InvalidCredentials(
                "login_scope TENANT requires organization_id to be absent or 0".to_string(),
            ))
        }
        ("ORGANIZATION", Some(value)) if !value.is_empty() && value != "0" => Ok(login_scope),
        ("ORGANIZATION", _) => Err(TokenClaimsError::InvalidCredentials(
            "login_scope ORGANIZATION requires a non-zero organization_id".to_string(),
        )),
        _ => Ok(login_scope),
    }
}

fn split_claim(value: Option<&String>) -> Vec<String> {
    value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn required_claim(
    claims: &BTreeMap<String, String>,
    key: &str,
) -> Result<String, TokenClaimsError> {
    claims
        .get(key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| TokenClaimsError::MissingField(key.to_string()))
}

fn optional_claim(claims: &BTreeMap<String, String>, key: &str) -> Option<String> {
    claims
        .get(key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn require_optional_match(
    field: &str,
    left: Option<&str>,
    right: Option<&str>,
) -> Result<(), TokenClaimsError> {
    match (left, right) {
        (Some(left), Some(right)) if left != right => Err(TokenClaimsError::Forbidden(format!(
            "auth token and access token {field} contexts do not match"
        ))),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::DriveAuthValidationPolicy;

    fn dev_policy() -> DriveAuthValidationPolicy {
        DriveAuthValidationPolicy::allow_unsigned_for_development()
    }

    fn auth_token(tenant: &str, user: &str) -> String {
        format!(
            "tenant_id={tenant};user_id={user};session_id=session-1;app_id=appbase;auth_level=password"
        )
    }

    fn access_token(tenant: &str, user: &str) -> String {
        format!(
            "tenant_id={tenant};user_id={user};session_id=session-1;app_id=appbase;environment=prod;deployment_mode=saas"
        )
    }

    #[test]
    fn rejects_inline_tokens_when_signed_policy_is_required() {
        let policy = DriveAuthValidationPolicy::require_signed_projection("test-secret");
        let error = resolve_dual_token_context(
            "tenant_id=tenant-a;user_id=user-001",
            "tenant_id=tenant-a;user_id=user-001;app_id=appbase",
            &policy,
        )
        .expect_err("inline token should be rejected");
        assert!(matches!(error, TokenClaimsError::InvalidCredentials(_)));
    }

    #[test]
    fn resolves_dual_token_context_from_claim_strings() {
        let context = resolve_dual_token_context(
            &auth_token("tenant-a", "user-001"),
            &access_token("tenant-a", "user-001"),
            &dev_policy(),
        )
        .expect("context");

        assert_eq!(context.tenant_id, "tenant-a");
        assert_eq!(context.user_id, "user-001");
        assert_eq!(context.actor_id, "user-001");
        assert_eq!(context.actor_kind, "user");
    }

    #[test]
    fn rejects_mismatched_tenant_claims() {
        let error = resolve_dual_token_context(
            &auth_token("tenant-a", "user-001"),
            &access_token("tenant-b", "user-001"),
            &dev_policy(),
        )
        .expect_err("mismatch");

        assert_eq!(
            error,
            TokenClaimsError::Forbidden(
                "auth token and access token tenant_id contexts do not match".to_string()
            )
        );
    }
}
