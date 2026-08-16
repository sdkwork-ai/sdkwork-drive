use http::{HeaderMap, Uri};
use sdkwork_utils_rust::{uuid, SdkWorkResultCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::policy::DriveAuthValidationPolicy;
use crate::token_claims::{resolve_dual_token_context, TokenClaimsError};

pub const AUTHORIZATION_HEADER: &str = "authorization";
pub const ACCESS_TOKEN_HEADER: &str = "access-token";
pub const TRACE_ID_HEADER: &str = sdkwork_utils_rust::SDKWORK_TRACE_ID_HEADER;

pub fn crate_id() -> &'static str {
    "sdkwork-drive-security"
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveAppContext {
    pub tenant_id: String,
    pub user_id: String,
    pub organization_id: Option<String>,
    pub session_id: Option<String>,
    pub app_id: Option<String>,
    pub environment: Option<String>,
    pub deployment_mode: Option<String>,
    pub auth_level: Option<String>,
    pub data_scope: Vec<String>,
    pub permission_scope: Vec<String>,
    pub actor_id: String,
    pub actor_kind: String,
    pub device_id: Option<String>,
    pub request_id: String,
    pub trace_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveAuthError {
    pub status: u16,
    pub title: &'static str,
    pub detail: String,
    pub code: i32,
    pub request_id: String,
    pub trace_id: String,
}

pub fn validate_drive_app_context(
    headers: &HeaderMap,
    uri: &Uri,
    policy: &DriveAuthValidationPolicy,
) -> Result<DriveAppContext, DriveAuthError> {
    let request_id = uuid();
    let trace_id = optional_header(headers, TRACE_ID_HEADER).unwrap_or_else(uuid);

    let auth_token = bearer_token(headers, &request_id, &trace_id)?;
    if auth_token.is_empty() {
        return Err(auth_error(
            401,
            "unauthorized",
            "Authorization bearer token is required",
            SdkWorkResultCode::AuthenticationRequired,
            &request_id,
            &trace_id,
        ));
    }

    let access_token = required_header(
        headers,
        ACCESS_TOKEN_HEADER,
        "Access-Token header is required",
        SdkWorkResultCode::AuthenticationRequired,
        &request_id,
        &trace_id,
    )?;
    if access_token.trim().is_empty() {
        return Err(auth_error(
            401,
            "unauthorized",
            "Access-Token header is required",
            SdkWorkResultCode::AuthenticationRequired,
            &request_id,
            &trace_id,
        ));
    }

    let resolved = resolve_dual_token_context(&auth_token, &access_token, policy)
        .map_err(|error| map_token_claims_error(error, &request_id, &trace_id))?;

    let context = DriveAppContext {
        tenant_id: resolved.tenant_id,
        user_id: resolved.user_id.clone(),
        organization_id: resolved.organization_id,
        session_id: resolved.session_id,
        app_id: Some(resolved.app_id),
        environment: Some(resolved.environment),
        deployment_mode: Some(resolved.deployment_mode),
        auth_level: Some(resolved.auth_level),
        data_scope: resolved.data_scope,
        permission_scope: resolved.permission_scope,
        // actor_id should come from the resolved token's actor_subject or user_id
        // for proper impersonation audit trail support.
        actor_id: resolved.actor_id,
        actor_kind: resolved.actor_kind,
        device_id: resolved.device_id,
        request_id,
        trace_id,
    };

    validate_uri_context(uri, &context)?;
    Ok(context)
}

pub fn validate_json_context_projection(
    body: &Value,
    context: &DriveAppContext,
) -> Result<(), DriveAuthError> {
    let Some(object) = body.as_object() else {
        return Ok(());
    };
    if let Some(value) = first_string_field(object, &["tenantId", "tenant_id"]) {
        validate_tenant_match(value, context)?;
    }
    if let Some(value) = first_string_field(object, &["operatorId", "operator_id"]) {
        validate_operator_match(value, context)?;
    }
    if let Some(value) = first_string_field(object, &["subjectType", "subject_type"]) {
        validate_subject_type_match(value, context)?;
    }
    if let Some(value) = first_string_field(object, &["subjectId", "subject_id"]) {
        validate_subject_id_match(value, context)?;
    }
    if let Some(value) = first_string_field(object, &["userId", "user_id"]) {
        validate_user_match(value, context)?;
    }
    if let Some(value) = first_string_field(object, &["appId", "app_id"]) {
        validate_app_match(value, context)?;
    }
    if let Some(value) = first_string_field(object, &["organizationId", "organization_id"]) {
        validate_organization_match(value, context)?;
    }
    Ok(())
}

fn map_token_claims_error(
    error: TokenClaimsError,
    request_id: &str,
    trace_id: &str,
) -> DriveAuthError {
    match error {
        TokenClaimsError::MissingField(field) => auth_error(
            401,
            "unauthorized",
            format!("{field} claim is required in SDKWork dual-token credentials"),
            SdkWorkResultCode::InvalidToken,
            request_id,
            trace_id,
        ),
        TokenClaimsError::InvalidCredentials(detail) => auth_error(
            401,
            "unauthorized",
            detail,
            SdkWorkResultCode::InvalidToken,
            request_id,
            trace_id,
        ),
        TokenClaimsError::Forbidden(detail) => auth_error(
            403,
            "forbidden",
            detail,
            SdkWorkResultCode::TenantAccessDenied,
            request_id,
            trace_id,
        ),
    }
}

pub fn validate_uri_context(uri: &Uri, context: &DriveAppContext) -> Result<(), DriveAuthError> {
    let Some(query) = uri.query() else {
        return Ok(());
    };
    let query = parse_query(query);
    if let Some(value) = first_query_value(&query, &["tenantId", "tenant_id"]) {
        validate_tenant_match(value, context)?;
    }
    if let Some(value) = first_query_value(&query, &["operatorId", "operator_id"]) {
        validate_operator_match(value, context)?;
    }
    if let Some(value) = first_query_value(&query, &["subjectType", "subject_type"]) {
        validate_subject_type_match(value, context)?;
    }
    if let Some(value) = first_query_value(&query, &["subjectId", "subject_id"]) {
        validate_subject_id_match(value, context)?;
    }
    if let Some(value) = first_query_value(&query, &["userId", "user_id"]) {
        validate_user_match(value, context)?;
    }
    if let Some(value) = first_query_value(&query, &["appId", "app_id"]) {
        validate_app_match(value, context)?;
    }
    if let Some(value) = first_query_value(&query, &["organizationId", "organization_id"]) {
        validate_organization_match(value, context)?;
    }
    Ok(())
}

fn validate_tenant_match(value: &str, context: &DriveAppContext) -> Result<(), DriveAuthError> {
    if value.trim() == context.tenant_id {
        return Ok(());
    }
    Err(context_conflict(
        "request tenant does not match verified SDKWork AppContext tenant",
        context,
    ))
}

fn validate_operator_match(value: &str, context: &DriveAppContext) -> Result<(), DriveAuthError> {
    if value.trim() == context.actor_id {
        return Ok(());
    }
    Err(context_conflict(
        "request operator does not match verified SDKWork AppContext actor",
        context,
    ))
}

fn validate_subject_type_match(
    value: &str,
    context: &DriveAppContext,
) -> Result<(), DriveAuthError> {
    if value.trim() == context.actor_kind {
        return Ok(());
    }
    Err(context_conflict(
        "request subjectType does not match verified SDKWork AppContext subject type",
        context,
    ))
}

fn validate_subject_id_match(value: &str, context: &DriveAppContext) -> Result<(), DriveAuthError> {
    if value.trim() == context.user_id {
        return Ok(());
    }
    Err(context_conflict(
        "request subjectId does not match verified SDKWork AppContext subject",
        context,
    ))
}

fn validate_user_match(value: &str, context: &DriveAppContext) -> Result<(), DriveAuthError> {
    if value.trim() == context.user_id {
        return Ok(());
    }
    Err(context_conflict(
        "request userId does not match verified SDKWork AppContext user",
        context,
    ))
}

fn validate_app_match(value: &str, context: &DriveAppContext) -> Result<(), DriveAuthError> {
    let Some(token_app_id) = context.app_id.as_deref() else {
        return Ok(());
    };
    if value.trim() == token_app_id {
        return Ok(());
    }
    Err(context_conflict(
        "request appId does not match verified SDKWork AppContext app",
        context,
    ))
}

fn validate_organization_match(
    value: &str,
    context: &DriveAppContext,
) -> Result<(), DriveAuthError> {
    let Some(token_organization_id) = context.organization_id.as_deref() else {
        return Ok(());
    };
    if value.trim() == token_organization_id {
        return Ok(());
    }
    Err(context_conflict(
        "request organizationId does not match verified SDKWork AppContext organization",
        context,
    ))
}

fn context_conflict(detail: impl Into<String>, context: &DriveAppContext) -> DriveAuthError {
    DriveAuthError {
        status: 403,
        title: "forbidden",
        detail: detail.into(),
        code: SdkWorkResultCode::TenantAccessDenied.as_i32(),
        request_id: context.request_id.clone(),
        trace_id: context.trace_id.clone(),
    }
}

fn bearer_token(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
) -> Result<String, DriveAuthError> {
    let Some(value) = optional_header(headers, AUTHORIZATION_HEADER) else {
        return Err(auth_error(
            401,
            "unauthorized",
            "Authorization bearer token is required",
            SdkWorkResultCode::AuthenticationRequired,
            request_id,
            trace_id,
        ));
    };
    let Some(token) = value.strip_prefix("Bearer ") else {
        return Err(auth_error(
            401,
            "unauthorized",
            "Authorization header must use Bearer token transport",
            SdkWorkResultCode::InvalidToken,
            request_id,
            trace_id,
        ));
    };
    Ok(token.trim().to_string())
}

fn required_header(
    headers: &HeaderMap,
    name: &str,
    detail: &str,
    code: SdkWorkResultCode,
    request_id: &str,
    trace_id: &str,
) -> Result<String, DriveAuthError> {
    let Some(value) = optional_header(headers, name) else {
        return Err(auth_error(
            401,
            "unauthorized",
            detail.to_string(),
            code,
            request_id,
            trace_id,
        ));
    };
    if value.trim().is_empty() {
        return Err(auth_error(
            401,
            "unauthorized",
            detail.to_string(),
            code,
            request_id,
            trace_id,
        ));
    }
    Ok(value)
}

fn optional_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn first_string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    names: &[&str],
) -> Option<&'a str> {
    names
        .iter()
        .find_map(|name| object.get(*name).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn first_query_value<'a>(
    query: &'a std::collections::BTreeMap<String, String>,
    names: &[&str],
) -> Option<&'a str> {
    names
        .iter()
        .find_map(|name| query.get(*name).map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn parse_query(query: &str) -> std::collections::BTreeMap<String, String> {
    let mut values = std::collections::BTreeMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        values.insert(percent_decode(key), percent_decode(value));
    }
    values
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
                {
                    decoded.push((high << 4) | low);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            current => {
                decoded.push(current);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).to_string()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn auth_error(
    status: u16,
    title: &'static str,
    detail: impl Into<String>,
    code: SdkWorkResultCode,
    request_id: &str,
    trace_id: &str,
) -> DriveAuthError {
    DriveAuthError {
        status,
        title,
        detail: detail.into(),
        code: code.as_i32(),
        request_id: request_id.to_string(),
        trace_id: trace_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

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

    fn dev_policy() -> DriveAuthValidationPolicy {
        DriveAuthValidationPolicy::allow_unsigned_for_development()
    }

    #[test]
    fn validates_dual_tokens_without_context_projection_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION_HEADER,
            HeaderValue::from_str(&format!("Bearer {}", auth_token("tenant-001", "user-001")))
                .expect("auth header"),
        );
        headers.insert(
            ACCESS_TOKEN_HEADER,
            HeaderValue::from_str(&access_token("tenant-001", "user-001")).expect("access header"),
        );

        let context = validate_drive_app_context(
            &headers,
            &"/app/v3/api/drive/spaces?tenantId=tenant-001"
                .parse()
                .expect("uri"),
            &dev_policy(),
        )
        .expect("context should validate");

        assert_eq!(context.tenant_id, "tenant-001");
        assert_eq!(context.user_id, "user-001");
        assert_eq!(context.actor_id, "user-001");
    }

    #[test]
    fn rejects_query_tenant_conflicts() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION_HEADER,
            HeaderValue::from_str(&format!("Bearer {}", auth_token("tenant-001", "user-001")))
                .expect("auth header"),
        );
        headers.insert(
            ACCESS_TOKEN_HEADER,
            HeaderValue::from_str(&access_token("tenant-001", "user-001")).expect("access header"),
        );

        let error = validate_drive_app_context(
            &headers,
            &"/app/v3/api/drive/spaces?tenantId=tenant-002"
                .parse()
                .expect("uri"),
            &dev_policy(),
        )
        .expect_err("tenant conflict should fail");

        assert_eq!(error.status, 403);
        assert_eq!(error.code, SdkWorkResultCode::TenantAccessDenied.as_i32());
    }

    #[test]
    fn rejects_query_subject_conflicts() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION_HEADER,
            HeaderValue::from_str(&format!("Bearer {}", auth_token("tenant-001", "user-001")))
                .expect("auth header"),
        );
        headers.insert(
            ACCESS_TOKEN_HEADER,
            HeaderValue::from_str(&access_token("tenant-001", "user-001")).expect("access header"),
        );

        let error = validate_drive_app_context(
            &headers,
            &"/app/v3/api/drive/shared_with_me?subjectId=user-002"
                .parse()
                .expect("uri"),
            &dev_policy(),
        )
        .expect_err("subject conflict should fail");

        assert_eq!(error.status, 403);
        assert_eq!(error.code, SdkWorkResultCode::TenantAccessDenied.as_i32());
    }

    #[test]
    fn ignores_supplied_context_projection_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION_HEADER,
            HeaderValue::from_str(&format!("Bearer {}", auth_token("tenant-001", "user-001")))
                .expect("auth header"),
        );
        headers.insert(
            ACCESS_TOKEN_HEADER,
            HeaderValue::from_str(&access_token("tenant-001", "user-001")).expect("access header"),
        );
        headers.insert(
            "x-sdkwork-tenant-id",
            HeaderValue::from_static("tenant-forged"),
        );
        headers.insert("x-sdkwork-user-id", HeaderValue::from_static("user-forged"));

        let context = validate_drive_app_context(
            &headers,
            &"/app/v3/api/drive/spaces".parse().expect("uri"),
            &dev_policy(),
        )
        .expect("token-derived context wins");

        assert_eq!(context.tenant_id, "tenant-001");
        assert_eq!(context.user_id, "user-001");
    }

    #[test]
    fn ignores_legacy_request_id_header_and_prefers_canonical_trace_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION_HEADER,
            HeaderValue::from_str(&format!("Bearer {}", auth_token("tenant-001", "user-001")))
                .expect("auth header"),
        );
        headers.insert(
            ACCESS_TOKEN_HEADER,
            HeaderValue::from_str(&access_token("tenant-001", "user-001")).expect("access header"),
        );
        headers.insert(
            "x-request-id",
            HeaderValue::from_static("client-forged-request"),
        );
        headers.insert("x-trace-id", HeaderValue::from_static("legacy-trace"));
        headers.insert(
            sdkwork_utils_rust::SDKWORK_TRACE_ID_HEADER,
            HeaderValue::from_static("canonical-trace"),
        );

        let context = validate_drive_app_context(
            &headers,
            &"/app/v3/api/drive/spaces".parse().expect("uri"),
            &dev_policy(),
        )
        .expect("context should validate");

        assert_ne!(context.request_id, "client-forged-request");
        assert_ne!(context.request_id, "request-unset");
        assert_eq!(context.trace_id, "canonical-trace");
    }

    #[test]
    fn generates_server_correlation_when_trace_header_is_absent() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION_HEADER,
            HeaderValue::from_str(&format!("Bearer {}", auth_token("tenant-001", "user-001")))
                .expect("auth header"),
        );
        headers.insert(
            ACCESS_TOKEN_HEADER,
            HeaderValue::from_str(&access_token("tenant-001", "user-001")).expect("access header"),
        );

        let context = validate_drive_app_context(
            &headers,
            &"/app/v3/api/drive/spaces".parse().expect("uri"),
            &dev_policy(),
        )
        .expect("context should validate");

        assert_ne!(context.request_id, "request-unset");
        assert_ne!(context.trace_id, "trace-unset");
        assert_eq!(context.request_id.len(), 36);
        assert_eq!(context.trace_id.len(), 36);
    }
}
