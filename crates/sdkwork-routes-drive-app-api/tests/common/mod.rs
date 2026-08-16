#![allow(dead_code)]

use axum::body::{to_bytes, Body};
use axum::Router;
use http::{Method, Request, Response, StatusCode};
use sdkwork_routes_drive_app_api::build_router_with_pool_and_iam;
use sdkwork_web_core::{access_token_jwt, auth_token_jwt, encode_unsigned_test_jwt};
use serde_json::json;
use sqlx::PgPool;

const DEFAULT_SESSION_ID: &str = "session-1";
const ANONYMOUS_SESSION_ID: &str = "anonymous-session-1";

pub fn auth_token(tenant: &str, user: &str, app_id: &str) -> String {
    auth_token_jwt(tenant, user, DEFAULT_SESSION_ID, app_id)
}

pub fn access_token(tenant: &str, user: &str, app_id: &str) -> String {
    access_token_jwt(tenant, user, DEFAULT_SESSION_ID, app_id)
}

pub fn anonymous_auth_token(tenant: &str, subject_id: &str, app_id: &str) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "auth",
        "tenant_id": tenant,
        "user_id": subject_id,
        "session_id": ANONYMOUS_SESSION_ID,
        "app_id": app_id,
        "subject_type": "user",
        "auth_level": "anonymous",
        "login_scope": "TENANT",
    }))
}

pub fn anonymous_access_token(tenant: &str, subject_id: &str, app_id: &str) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "access",
        "tenant_id": tenant,
        "user_id": subject_id,
        "session_id": ANONYMOUS_SESSION_ID,
        "app_id": app_id,
        "subject_type": "user",
        "environment": "prod",
        "deployment_mode": "saas",
        "login_scope": "TENANT",
    }))
}

pub fn auth_token_for_organization(
    tenant: &str,
    user: &str,
    organization_id: &str,
    app_id: &str,
) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "auth",
        "tenant_id": tenant,
        "user_id": user,
        "organization_id": organization_id,
        "session_id": DEFAULT_SESSION_ID,
        "app_id": app_id,
        "auth_level": "password",
        "login_scope": "ORGANIZATION",
    }))
}

pub fn access_token_for_organization(
    tenant: &str,
    user: &str,
    organization_id: &str,
    app_id: &str,
) -> String {
    encode_unsigned_test_jwt(json!({
        "token_type": "access",
        "tenant_id": tenant,
        "user_id": user,
        "organization_id": organization_id,
        "session_id": DEFAULT_SESSION_ID,
        "app_id": app_id,
        "environment": "prod",
        "deployment_mode": "saas",
        "login_scope": "ORGANIZATION",
    }))
}

pub fn default_user_for_tenant(tenant: &str) -> String {
    if let Some(suffix) = tenant.strip_prefix("tenant-") {
        return format!("user-{suffix}");
    }
    "user-001".to_string()
}

pub fn user_from_uri(uri: &str, tenant: &str) -> String {
    if let Some((_, query)) = uri.split_once('?') {
        for segment in query.split('&') {
            if let Some(subject_id) = segment.strip_prefix("subjectId=") {
                return percent_decode(subject_id);
            }
            if let Some(operator_id) = segment.strip_prefix("operatorId=") {
                return percent_decode(operator_id);
            }
            if let Some(user_id) = segment.strip_prefix("userId=") {
                return percent_decode(user_id);
            }
        }
    }
    default_user_for_tenant(tenant)
}

pub fn authed_get_uri(uri: impl AsRef<str>, tenant: &str) -> Request<Body> {
    let user = user_from_uri(uri.as_ref(), tenant);
    authed_get(uri, tenant, &user, "appbase")
}

fn percent_decode(value: &str) -> String {
    value.replace("%40", "@")
}

pub fn test_router_with_pool(pool: PgPool) -> Router {
    build_router_with_pool_and_iam(pool)
}

pub fn authed_request(
    method: Method,
    uri: impl AsRef<str>,
    tenant: &str,
    user: &str,
    app_id: &str,
    body: Body,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri.as_ref())
        .header(
            "authorization",
            format!("Bearer {}", auth_token(tenant, user, app_id)),
        )
        .header("access-token", access_token(tenant, user, app_id))
        .body(body)
        .expect("authed request should be built")
}

pub fn authed_get(uri: impl AsRef<str>, tenant: &str, user: &str, app_id: &str) -> Request<Body> {
    authed_request(Method::GET, uri, tenant, user, app_id, Body::empty())
}

pub fn authed_post_json(
    uri: impl AsRef<str>,
    tenant: &str,
    user: &str,
    app_id: &str,
    body: impl Into<Body>,
) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri.as_ref())
        .header(
            "authorization",
            format!("Bearer {}", auth_token(tenant, user, app_id)),
        )
        .header("access-token", access_token(tenant, user, app_id))
        .header("content-type", "application/json")
        .body(body.into())
        .expect("authed post request should be built")
}

pub fn anonymous_post_json(
    uri: impl AsRef<str>,
    tenant: &str,
    subject_id: &str,
    app_id: &str,
    body: impl Into<Body>,
) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri.as_ref())
        .header(
            "authorization",
            format!(
                "Bearer {}",
                anonymous_auth_token(tenant, subject_id, app_id)
            ),
        )
        .header(
            "access-token",
            anonymous_access_token(tenant, subject_id, app_id),
        )
        .header("content-type", "application/json")
        .body(body.into())
        .expect("anonymous post request should be built")
}

pub fn authed_idempotent_post_json(
    uri: impl AsRef<str>,
    tenant: &str,
    user: &str,
    app_id: &str,
    idempotency_key: &str,
    body: impl Into<Body>,
) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri.as_ref())
        .header(
            "authorization",
            format!("Bearer {}", auth_token(tenant, user, app_id)),
        )
        .header("access-token", access_token(tenant, user, app_id))
        .header("idempotency-key", idempotency_key)
        .header("content-type", "application/json")
        .body(body.into())
        .expect("authed idempotent post request should be built")
}

pub fn envelope_item(payload: &serde_json::Value) -> &serde_json::Value {
    if let Some(item) = payload.pointer("/data/item") {
        return item;
    }
    payload.get("item").unwrap_or(payload)
}

pub fn envelope_data(payload: &serde_json::Value) -> &serde_json::Value {
    payload
        .get("data")
        .expect("response should expose operation payload in data")
}

/// Unwrap `data.item` for resource responses, otherwise `data`, otherwise the root payload.
pub fn envelope_body(payload: &serde_json::Value) -> &serde_json::Value {
    if let Some(data) = payload.get("data") {
        if let Some(item) = data.get("item") {
            return item;
        }
        return data;
    }
    payload.get("item").unwrap_or(payload)
}

pub fn envelope_field<'a>(payload: &'a serde_json::Value, field: &str) -> &'a serde_json::Value {
    if let Some(value) = payload.pointer(&format!("/data/{field}")) {
        return value;
    }
    payload
        .get(field)
        .unwrap_or_else(|| panic!("response should expose {field} in data.{field} or {field}"))
}

pub fn envelope_items(payload: &serde_json::Value) -> &serde_json::Value {
    if let Some(items) = payload.pointer("/data/items") {
        return items;
    }
    payload
        .get("items")
        .expect("response should expose list items in data.items or items")
}

pub fn envelope_page_info(payload: &serde_json::Value) -> Option<&serde_json::Value> {
    payload
        .pointer("/data/pageInfo")
        .or_else(|| payload.get("pageInfo"))
}

pub fn envelope_next_page_token(payload: &serde_json::Value) -> Option<String> {
    envelope_page_info(payload)
        .and_then(|page_info| page_info.get("nextCursor"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .or_else(|| {
            payload
                .get("nextPageToken")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
}

pub async fn assert_no_content_response(response: Response<Body>) {
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("204 response body should be readable");
    assert!(
        body.is_empty(),
        "204 delete response must not include a JSON body"
    );
}
