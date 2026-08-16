use crate::error::{validation_problem, ProblemDetail};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_contract::api::pagination_cursor::{decode_offset_cursor, encode_offset_cursor};
use sdkwork_utils_rust::{DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE};

/// 路径参数形式的对象 key 解码与校验。
///
/// axum 的 `Path` 提取器已对路径段做一次 percent-decode，这里**不再手动解码**，
/// 否则字面 `%` 字符会被二次解码静默改写（如 `50%20off.txt` → `50 off.txt`）。
/// 允许单个尾斜杠（目录占位对象，如 `docs/`），拒绝前导斜杠、双斜杠与
/// 空/`.`/`..` 段；返回规范化后的 key（尾斜杠折叠为单个）。
pub(crate) fn decode_path_object_key(
    raw: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    validate_path_object_key(raw, "objectKey")
}

fn validate_path_object_key(
    value: &str,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = require_non_empty_text(value.to_string(), field_name)?;
    if trimmed.len() > 1024 {
        return Err(validation_problem(format!(
            "{field_name} must be at most 1024 UTF-8 bytes"
        )));
    }
    if trimmed.as_bytes().contains(&0) {
        return Err(validation_problem(format!(
            "{field_name} must not contain NUL bytes"
        )));
    }
    if trimmed.starts_with('/') {
        return Err(validation_problem(format!(
            "{field_name} must not start with slash"
        )));
    }
    let (effective, has_trailing_slash) = match trimmed.strip_suffix('/') {
        Some(rest) => (rest, true),
        None => (trimmed.as_str(), false),
    };
    if effective.is_empty() {
        return Err(validation_problem(format!(
            "{field_name} must not be only slashes"
        )));
    }
    for segment in effective.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(validation_problem(format!(
                "{field_name} must not contain empty or period-only path segments"
            )));
        }
    }
    Ok(if has_trailing_slash {
        format!("{effective}/")
    } else {
        effective.to_string()
    })
}

pub(crate) fn validate_object_key(
    value: String,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = require_non_empty_text(value, field_name)?;
    if trimmed.len() > 1024 {
        return Err(validation_problem(format!(
            "{field_name} must be at most 1024 UTF-8 bytes"
        )));
    }
    if trimmed.as_bytes().contains(&0) {
        return Err(validation_problem(format!(
            "{field_name} must not contain NUL bytes"
        )));
    }
    if trimmed.starts_with('/') || trimmed.ends_with('/') {
        return Err(validation_problem(format!(
            "{field_name} must not start or end with slash"
        )));
    }
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(validation_problem(format!(
                "{field_name} must not contain empty or period-only path segments"
            )));
        }
    }
    Ok(trimmed)
}

pub(crate) fn validate_object_prefix(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    let Some(trimmed) = normalize_optional_text(value) else {
        return Ok(None);
    };
    if trimmed.len() > 1024 {
        return Err(validation_problem(format!(
            "{field_name} must be at most 1024 UTF-8 bytes"
        )));
    }
    if trimmed.as_bytes().contains(&0) || trimmed.starts_with('/') {
        return Err(validation_problem(format!("{field_name} is invalid")));
    }
    if trimmed.contains("//") {
        return Err(validation_problem(format!(
            "{field_name} must not contain empty path segments"
        )));
    }
    for segment in trimmed.trim_end_matches('/').split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(validation_problem(format!(
                "{field_name} must not contain empty or period-only path segments"
            )));
        }
    }
    Ok(Some(trimmed))
}

pub(crate) fn validate_object_delimiter(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    let Some(trimmed) = normalize_optional_text(value) else {
        return Ok(None);
    };
    if trimmed != "/" {
        return Err(validation_problem(format!(
            "{field_name} must be '/' when provided"
        )));
    }
    Ok(Some(trimmed))
}

pub(crate) fn validate_page_size_u16(
    value: Option<u16>,
    default_value: u16,
    min_value: u16,
    max_value: u16,
    field_name: &str,
) -> Result<u16, (StatusCode, Json<ProblemDetail>)> {
    let page_size = value.unwrap_or(default_value);
    if page_size < min_value || page_size > max_value {
        return Err(validation_problem(format!(
            "{field_name} must be between {min_value} and {max_value}"
        )));
    }
    Ok(page_size)
}

pub(crate) fn parse_offset_page(
    page_size: Option<i64>,
    page_token: Option<String>,
) -> Result<crate::dto::OffsetPage, (StatusCode, Json<ProblemDetail>)> {
    let limit = validate_page_size_i64(
        page_size,
        i64::from(DEFAULT_LIST_PAGE_SIZE),
        1,
        i64::from(MAX_LIST_PAGE_SIZE),
        "page_size",
    )?;
    let offset = decode_offset_cursor(page_token.as_deref())
        .map_err(|_| validation_problem("cursor is invalid"))?;
    Ok(crate::dto::OffsetPage { limit, offset })
}

pub(crate) fn validate_page_size_i64(
    value: Option<i64>,
    default_value: i64,
    min_value: i64,
    max_value: i64,
    field_name: &str,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    let page_size = value.unwrap_or(default_value);
    if page_size < min_value || page_size > max_value {
        return Err(validation_problem(format!(
            "{field_name} must be between {min_value} and {max_value}"
        )));
    }
    Ok(page_size)
}

pub(crate) fn next_page_token<T>(
    items: &mut Vec<T>,
    page: crate::dto::OffsetPage,
) -> Option<String> {
    if items.len() as i64 > page.limit {
        items.pop();
        encode_offset_cursor(page.offset + page.limit)
    } else {
        None
    }
}

pub(crate) fn validate_storage_binding_lifecycle_status(
    status: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(status, "active" | "disabled" | "deleted") {
        return Ok(());
    }
    Err(validation_problem("lifecycleStatus is invalid"))
}

pub(crate) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
}

pub(crate) fn require_non_empty_text(
    value: String,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(validation_problem(format!("{field_name} is required")));
    }
    Ok(trimmed)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StorageProviderBindingTarget {
    Tenant,
    Space(String),
    SpaceType(String),
}

pub(crate) fn resolve_storage_provider_binding_target(
    space_id: Option<String>,
    space_type: Option<String>,
) -> Result<StorageProviderBindingTarget, (StatusCode, Json<ProblemDetail>)> {
    let space_id = normalize_optional_text(space_id);
    let space_type = normalize_optional_text(space_type);
    if space_id.is_some() && space_type.is_some() {
        return Err(validation_problem(
            "spaceId and spaceType are mutually exclusive",
        ));
    }
    if let Some(space_id) = space_id {
        return Ok(StorageProviderBindingTarget::Space(space_id));
    }
    if let Some(space_type) = space_type {
        validate_storage_binding_space_type(&space_type)?;
        return Ok(StorageProviderBindingTarget::SpaceType(space_type));
    }
    Ok(StorageProviderBindingTarget::Tenant)
}

pub(crate) fn validate_storage_binding_space_type(
    space_type: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    match space_type {
        "personal" | "team" | "knowledge_base" | "ai_generated" | "git_repository"
        | "deployment" | "app_upload" | "im" | "rtc" | "notary" | "website" => Ok(()),
        _ => Err(validation_problem("spaceType is invalid")),
    }
}

pub(crate) fn default_storage_provider_binding_id(
    tenant_id: &str,
    target: &StorageProviderBindingTarget,
) -> String {
    match target {
        StorageProviderBindingTarget::Tenant => format!("default:tenant:{tenant_id}"),
        StorageProviderBindingTarget::Space(space_id) => {
            format!("default:space:{tenant_id}:{space_id}")
        }
        StorageProviderBindingTarget::SpaceType(space_type) => {
            format!("default:space_type:{tenant_id}:{space_type}")
        }
    }
}

pub(crate) fn storage_provider_binding_scope(
    target: &StorageProviderBindingTarget,
) -> &'static str {
    match target {
        StorageProviderBindingTarget::Tenant => "tenant",
        StorageProviderBindingTarget::Space(_) => "space",
        StorageProviderBindingTarget::SpaceType(_) => "space_type",
    }
}

pub(crate) fn storage_provider_binding_purpose(target: &StorageProviderBindingTarget) -> String {
    match target {
        StorageProviderBindingTarget::Tenant | StorageProviderBindingTarget::Space(_) => {
            "primary".to_string()
        }
        StorageProviderBindingTarget::SpaceType(space_type) => space_type.clone(),
    }
}

fn default_storage_root_prefix(tenant_id: &str, target: &StorageProviderBindingTarget) -> String {
    match target {
        StorageProviderBindingTarget::Tenant => {
            format!("sdkwork-drive/v1/tenants/{tenant_id}")
        }
        StorageProviderBindingTarget::Space(space_id) => {
            format!("sdkwork-drive/v1/tenants/{tenant_id}/spaces/{space_id}")
        }
        StorageProviderBindingTarget::SpaceType(space_type) => {
            format!("sdkwork-drive/v1/tenants/{tenant_id}/space-types/{space_type}")
        }
    }
}

pub(crate) fn normalize_storage_root_prefix(
    value: Option<String>,
    tenant_id: &str,
    target: &StorageProviderBindingTarget,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let prefix = normalize_optional_text(value)
        .unwrap_or_else(|| default_storage_root_prefix(tenant_id, target));
    let prefix = validate_object_prefix(Some(prefix), "storageRootPrefix")?
        .ok_or_else(|| validation_problem("storageRootPrefix is required"))?;
    if prefix.len() > 512 {
        return Err(validation_problem(
            "storageRootPrefix must be at most 512 UTF-8 bytes",
        ));
    }
    if prefix.ends_with('/') {
        return Err(validation_problem(
            "storageRootPrefix must not end with slash",
        ));
    }
    Ok(prefix)
}

#[cfg(test)]
mod pagination_cursor_tests {
    use super::{next_page_token, parse_offset_page};
    use crate::dto::OffsetPage;

    fn is_numeric_token(value: &str) -> bool {
        value.bytes().all(|byte| byte.is_ascii_digit())
    }

    #[test]
    fn offset_page_tokens_are_opaque_and_round_trip() {
        let mut items = vec![1, 2];

        let token = next_page_token(
            &mut items,
            OffsetPage {
                limit: 1,
                offset: 0,
            },
        )
        .expect("first page should expose continuation token");

        assert!(!is_numeric_token(&token));
        let parsed = parse_offset_page(Some(1), Some(token)).expect("opaque cursor should parse");
        assert_eq!(parsed.offset, 1);
        assert_eq!(items, vec![1]);
    }

    #[test]
    fn offset_page_rejects_numeric_cursor_alias() {
        let err = parse_offset_page(Some(20), Some("20".to_string()))
            .expect_err("numeric cursor is pre-launch pagination debt");

        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }
}
