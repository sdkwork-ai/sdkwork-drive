use std::future::Future;

use base64::Engine;
use http::HeaderMap;
use sdkwork_drive_security::{
    resolve_dual_token_context, DriveAppContext, DriveAuthError, DriveAuthValidationPolicy,
    TokenClaimsError, ACCESS_TOKEN_HEADER, AUTHORIZATION_HEADER, TRACE_ID_HEADER,
};
use sdkwork_utils_rust::{uuid, SdkWorkResultCode};
use tokio::task_local;

task_local! {
    static CURRENT_GIT_AUTH: GitAuthContext;
}

#[derive(Debug, Clone)]
pub struct GitAuthContext {
    pub tenant_id: String,
    pub user_id: String,
    pub actor_id: String,
}

impl GitAuthContext {
    pub fn from_drive_context(context: &DriveAppContext) -> Self {
        Self {
            tenant_id: context.tenant_id.clone(),
            user_id: context.user_id.clone(),
            actor_id: context.actor_id.clone(),
        }
    }

    pub fn current() -> Option<Self> {
        CURRENT_GIT_AUTH.try_with(|context| context.clone()).ok()
    }

    pub async fn scope<F, T>(context: Self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        CURRENT_GIT_AUTH.scope(context, future).await
    }
}

pub fn validate_git_request_auth(
    headers: &HeaderMap,
    policy: &DriveAuthValidationPolicy,
) -> Result<GitAuthContext, DriveAuthError> {
    let request_id = uuid();
    let trace_id = optional_header(headers, TRACE_ID_HEADER).unwrap_or_else(uuid);
    let (auth_token, access_token) = extract_dual_tokens(headers, &request_id, &trace_id)?;
    let resolved = resolve_dual_token_context(&auth_token, &access_token, policy)
        .map_err(|error| map_token_claims_error(error, &request_id, &trace_id))?;

    if resolved.user_id.trim().is_empty() {
        return Err(auth_error(
            401,
            "unauthorized",
            "authenticated git user is required",
            SdkWorkResultCode::AuthenticationRequired,
            &request_id,
            &trace_id,
        ));
    }

    Ok(GitAuthContext {
        tenant_id: resolved.tenant_id,
        user_id: resolved.user_id,
        actor_id: resolved.actor_id,
    })
}

fn extract_dual_tokens(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
) -> Result<(String, String), DriveAuthError> {
    if let Some(access_token) = optional_header(headers, ACCESS_TOKEN_HEADER) {
        let auth_token = bearer_token(headers, request_id, trace_id)?;
        return Ok((auth_token, access_token));
    }

    let authorization = required_header(
        headers,
        AUTHORIZATION_HEADER,
        "Authorization header is required",
        SdkWorkResultCode::AuthenticationRequired,
        request_id,
        trace_id,
    )?;

    if authorization.starts_with("Bearer ") {
        return Err(auth_error(
            401,
            "unauthorized",
            "git requests require both auth and access tokens; use HTTP Basic auth with auth_token:access_token or provide an access-token header",
            SdkWorkResultCode::AuthenticationRequired,
            request_id,
            trace_id,
        ));
    }

    if let Some(encoded) = authorization.strip_prefix("Basic ") {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| {
                auth_error(
                    401,
                    "unauthorized",
                    "invalid basic authorization credentials",
                    SdkWorkResultCode::AuthenticationRequired,
                    request_id,
                    trace_id,
                )
            })?;
        let credentials = String::from_utf8(decoded).map_err(|_| {
            auth_error(
                401,
                "unauthorized",
                "invalid basic authorization credentials",
                SdkWorkResultCode::AuthenticationRequired,
                request_id,
                trace_id,
            )
        })?;
        let (auth_token, access_token) = credentials.split_once(':').ok_or_else(|| {
            auth_error(
                401,
                "unauthorized",
                "basic authorization must encode auth_token:access_token",
                SdkWorkResultCode::AuthenticationRequired,
                request_id,
                trace_id,
            )
        })?;
        if auth_token.trim().is_empty() || access_token.trim().is_empty() {
            return Err(auth_error(
                401,
                "unauthorized",
                "basic authorization must encode auth_token:access_token",
                SdkWorkResultCode::AuthenticationRequired,
                request_id,
                trace_id,
            ));
        }
        return Ok((auth_token.to_string(), access_token.to_string()));
    }

    Err(auth_error(
        401,
        "unauthorized",
        "Authorization bearer or basic credentials are required",
        SdkWorkResultCode::AuthenticationRequired,
        request_id,
        trace_id,
    ))
}

fn bearer_token(
    headers: &HeaderMap,
    request_id: &str,
    trace_id: &str,
) -> Result<String, DriveAuthError> {
    let authorization = required_header(
        headers,
        AUTHORIZATION_HEADER,
        "Authorization bearer token is required",
        SdkWorkResultCode::AuthenticationRequired,
        request_id,
        trace_id,
    )?;
    authorization
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            auth_error(
                401,
                "unauthorized",
                "Authorization bearer token is required",
                SdkWorkResultCode::AuthenticationRequired,
                request_id,
                trace_id,
            )
        })
}

fn optional_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn required_header(
    headers: &HeaderMap,
    name: &str,
    detail: &str,
    code: SdkWorkResultCode,
    request_id: &str,
    trace_id: &str,
) -> Result<String, DriveAuthError> {
    optional_header(headers, name).ok_or_else(|| auth_error(401, "unauthorized", detail, code, request_id, trace_id))
}

fn auth_error(
    status: u16,
    title: &'static str,
    detail: &str,
    code: SdkWorkResultCode,
    request_id: &str,
    trace_id: &str,
) -> DriveAuthError {
    DriveAuthError {
        status,
        title,
        detail: detail.to_string(),
        code: code.as_i32(),
        request_id: request_id.to_string(),
        trace_id: trace_id.to_string(),
    }
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
            &format!("{field} claim is required in SDKWork dual-token credentials"),
            SdkWorkResultCode::InvalidToken,
            request_id,
            trace_id,
        ),
        TokenClaimsError::InvalidCredentials(detail) => auth_error(
            401,
            "unauthorized",
            &detail,
            SdkWorkResultCode::InvalidToken,
            request_id,
            trace_id,
        ),
        TokenClaimsError::Forbidden(detail) => auth_error(
            403,
            "forbidden",
            &detail,
            SdkWorkResultCode::TenantAccessDenied,
            request_id,
            trace_id,
        ),
    }
}
