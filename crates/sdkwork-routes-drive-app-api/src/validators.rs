use crate::constants::{DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE};
use crate::dto::PageRequest;
use crate::error::{problem, ProblemDetail, SdkWorkResultCode};
use crate::time::current_epoch_ms;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_contract::api::pagination_cursor::{
    decode_change_sequence_cursor, decode_offset_cursor, encode_offset_cursor,
};

pub(crate) fn require_query_value(
    value: Option<String>,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    require_body_value(value, field_name)
}

pub(crate) fn require_body_value(
    value: Option<String>,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let Some(raw) = value else {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} is required"),
            SdkWorkResultCode::ValidationError,
        ));
    };
    let trimmed = raw.trim().to_string();
    if trimmed.is_empty() {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} is required"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(trimmed)
}

pub(crate) fn require_non_empty_text(
    value: String,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} is required"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(trimmed)
}

pub(crate) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
}

pub(crate) fn parse_page_request(
    page_size: Option<i64>,
    cursor: Option<String>,
) -> Result<PageRequest, (StatusCode, Json<ProblemDetail>)> {
    let limit = validate_page_size_i64(
        page_size,
        DEFAULT_LIST_PAGE_SIZE,
        1,
        MAX_LIST_PAGE_SIZE,
        "page_size",
    )?;
    let offset = decode_offset_cursor(cursor.as_deref()).map_err(|_| {
        problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "cursor is invalid",
            SdkWorkResultCode::ValidationError,
        )
    })?;
    Ok(PageRequest { limit, offset })
}

pub(crate) fn parse_nodes_list_order_clause(
    sort_by: Option<String>,
    sort_order: Option<String>,
) -> Result<&'static str, (StatusCode, Json<ProblemDetail>)> {
    let sort_by = normalize_optional_text(sort_by).unwrap_or_else(|| "name".to_string());
    let sort_order = normalize_optional_text(sort_order).unwrap_or_else(|| "asc".to_string());
    let desc = sort_order.eq_ignore_ascii_case("desc");
    match (sort_by.as_str(), desc) {
        ("name", false) => Ok("node_type ASC, node_name ASC"),
        ("name", true) => Ok("node_type ASC, node_name DESC"),
        ("owner", false) => Ok("node_type ASC, created_by ASC, node_name ASC"),
        ("owner", true) => Ok("node_type ASC, created_by DESC, node_name ASC"),
        ("lastModified", false) => Ok("node_type ASC, updated_at ASC, node_name ASC"),
        ("lastModified", true) => Ok("node_type ASC, updated_at DESC, node_name ASC"),
        ("contentLength", false) => {
            Ok("node_type ASC, COALESCE(head_content_length, 0) ASC, node_name ASC")
        }
        ("contentLength", true) => {
            Ok("node_type ASC, COALESCE(head_content_length, 0) DESC, node_name ASC")
        }
        ("type", false) => {
            Ok("node_type ASC, COALESCE(head_content_type_group, '') ASC, node_name ASC")
        }
        ("type", true) => {
            Ok("node_type ASC, COALESCE(head_content_type_group, '') DESC, node_name ASC")
        }
        _ => Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "sortBy is invalid",
            SdkWorkResultCode::ValidationError,
        )),
    }
}

pub(crate) fn resolve_node_list_order_by(
    sort_by: Option<String>,
    sort_order: Option<String>,
    default_clause: &'static str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    if sort_by.is_none() && sort_order.is_none() {
        return Ok(default_clause.to_string());
    }
    Ok(parse_nodes_list_order_clause(sort_by, sort_order)?.to_string())
}

pub(crate) fn qualify_nodes_list_order_clause(clause: &str, node_alias: &str) -> String {
    clause
        .replace("node_type", &format!("{node_alias}.node_type"))
        .replace("node_name", &format!("{node_alias}.node_name"))
        .replace("created_by", &format!("{node_alias}.created_by"))
        .replace("updated_at", &format!("{node_alias}.updated_at"))
        .replace(
            "head_content_length",
            &format!("{node_alias}.head_content_length"),
        )
        .replace(
            "head_content_type_group",
            &format!("{node_alias}.head_content_type_group"),
        )
}

pub(crate) fn resolve_aliased_node_list_order_by(
    sort_by: Option<String>,
    sort_order: Option<String>,
    node_alias: &str,
    default_clause: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    if sort_by.is_none() && sort_order.is_none() {
        return Ok(default_clause.to_string());
    }
    let clause = parse_nodes_list_order_clause(sort_by, sort_order)?;
    Ok(qualify_nodes_list_order_clause(clause, node_alias))
}

pub(crate) fn next_page_token<T>(items: &mut Vec<T>, page: PageRequest) -> Option<String> {
    if items.len() as i64 > page.limit {
        items.pop();
        encode_offset_cursor(page.offset + page.limit)
    } else {
        None
    }
}

pub(crate) fn parse_change_page_request(
    page_size: Option<i64>,
    cursor: Option<String>,
) -> Result<PageRequest, (StatusCode, Json<ProblemDetail>)> {
    let limit = validate_page_size_i64(
        page_size,
        DEFAULT_LIST_PAGE_SIZE,
        1,
        MAX_LIST_PAGE_SIZE,
        "page_size",
    )?;
    let offset = decode_change_sequence_cursor(cursor.as_deref()).map_err(|_| {
        problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "cursor is invalid",
            SdkWorkResultCode::ValidationError,
        )
    })?;
    Ok(PageRequest { limit, offset })
}

pub(crate) fn validate_permission_role(
    role: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(role, "reader" | "commenter" | "writer" | "owner") {
        return Ok(());
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "permission role is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

pub(crate) fn validate_share_link_role(
    role: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(role, "reader" | "commenter" | "writer") {
        return Ok(());
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "share link role is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

pub(crate) fn validate_share_link_token(
    token: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if let Err(error) = sdkwork_drive_workspace_service::validate_share_link_token(token) {
        let message = match error {
            sdkwork_drive_workspace_service::DriveServiceError::Validation(message) => message,
            _ => "share link token is invalid".to_string(),
        };
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            &message,
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(())
}

pub(crate) fn validate_optional_non_negative_i64(
    value: Option<i64>,
    field_name: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if value.is_some_and(|value| value < 0) {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must be greater than or equal to 0"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(())
}

pub(crate) fn validate_optional_future_epoch_ms(
    value: Option<i64>,
    field_name: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if value.is_some_and(|value| value <= current_epoch_ms()) {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must be in the future"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(())
}

pub(crate) fn validate_requested_ttl_seconds(
    value: Option<u32>,
    default_seconds: u32,
    min_seconds: u32,
    max_seconds: u32,
    field_name: &str,
) -> Result<u32, (StatusCode, Json<ProblemDetail>)> {
    let ttl_seconds = value.unwrap_or(default_seconds);
    if ttl_seconds < min_seconds || ttl_seconds > max_seconds {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must be between {min_seconds} and {max_seconds} seconds"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(ttl_seconds)
}

pub(crate) fn validate_content_type<'a>(
    value: &'a str,
    field_name: &str,
) -> Result<&'a str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 255 || trimmed != value {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must be a trimmed MIME type up to 255 characters"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    let Some((type_part, subtype_part)) = trimmed.split_once('/') else {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must be a valid MIME type"),
            SdkWorkResultCode::ValidationError,
        ));
    };
    if !is_mime_token(type_part) || !is_mime_token(subtype_part) {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must be a valid MIME type"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(trimmed)
}

fn is_mime_token(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

pub(crate) fn validate_sha256_checksum<'a>(
    value: &'a str,
    field_name: &str,
) -> Result<&'a str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = value.trim();
    let Some(hex) = trimmed.strip_prefix("sha256:") else {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must use sha256:<64 lowercase hex characters>"),
            SdkWorkResultCode::ValidationError,
        ));
    };
    if trimmed != value
        || hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must use sha256:<64 lowercase hex characters>"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(trimmed)
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
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must be between {min_value} and {max_value}"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(page_size)
}

pub(crate) fn validate_object_key(
    value: String,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = require_non_empty_text(value, field_name)?;
    if trimmed.len() > 1024 {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must be at most 1024 UTF-8 bytes"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    if trimmed.as_bytes().contains(&0) {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must not contain NUL bytes"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    if trimmed.starts_with('/') || trimmed.ends_with('/') {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must not start or end with slash"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                format!("{field_name} must not contain empty or period-only path segments"),
                SdkWorkResultCode::ValidationError,
            ));
        }
    }
    Ok(trimmed)
}

pub(crate) fn validate_subject_type(
    subject_type: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(subject_type, "user" | "group" | "domain" | "app") {
        return Ok(());
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "subjectType is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

pub(crate) fn validate_node_property_key(
    property_key: &str,
) -> Result<&str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = property_key.trim();
    if trimmed.is_empty()
        || trimmed.len() > 128
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "propertyKey is invalid",
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(trimmed)
}

pub(crate) fn validate_node_property_visibility(
    visibility: &str,
) -> Result<&str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = visibility.trim();
    if matches!(trimmed, "private" | "app_public") {
        return Ok(trimmed);
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "visibility is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

pub(crate) fn validate_label_key(
    label_key: &str,
) -> Result<&str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = label_key.trim();
    if trimmed.is_empty()
        || trimmed.len() > 128
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "labelKey is invalid",
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(trimmed)
}

pub(crate) fn validate_watch_channel_id(
    channel_id: &str,
) -> Result<&str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = channel_id.trim();
    if trimmed.is_empty()
        || trimmed.len() > 64
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "id is invalid",
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(trimmed)
}

pub(crate) fn validate_watch_channel_address(
    address: &str,
) -> Result<&str, (StatusCode, Json<ProblemDetail>)> {
    crate::webhook_url::validate_webhook_https_url(address).map_err(|detail| {
        problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            detail,
            SdkWorkResultCode::ValidationError,
        )
    })
}

pub(crate) fn validate_watch_channel_type(
    channel_type: &str,
) -> Result<&str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = channel_type.trim();
    if trimmed == "web_hook" {
        return Ok(trimmed);
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "channelType is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

pub(crate) fn validate_watch_resource_type(
    resource_type: &str,
) -> Result<&str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = resource_type.trim();
    if matches!(trimmed, "changes" | "node") {
        return Ok(trimmed);
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "resourceType is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

pub(crate) fn validate_watch_lifecycle_status(
    lifecycle_status: &str,
) -> Result<&str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = lifecycle_status.trim();
    if matches!(trimmed, "active" | "stopped" | "expired") {
        return Ok(trimmed);
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "lifecycleStatus is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

pub(crate) fn validate_watch_expiration(
    expiration_epoch_ms: i64,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if expiration_epoch_ms <= current_epoch_ms() {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "expirationEpochMs must be in the future",
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod nodes_list_order_tests {
    use super::parse_nodes_list_order_clause;

    #[test]
    fn defaults_to_folder_first_name_asc() {
        let clause = parse_nodes_list_order_clause(None, None).expect("default sort");
        assert_eq!(clause, "node_type ASC, node_name ASC");
    }

    #[test]
    fn maps_content_length_descending() {
        let clause = parse_nodes_list_order_clause(
            Some("contentLength".to_string()),
            Some("desc".to_string()),
        )
        .expect("contentLength desc");
        assert!(clause.contains("head_content_length"));
        assert!(clause.contains("DESC"));
    }

    #[test]
    fn rejects_unknown_sort_field() {
        let err = parse_nodes_list_order_clause(Some("invalid".to_string()), None)
            .expect_err("invalid sort");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_legacy_size_sort_field() {
        let err = parse_nodes_list_order_clause(Some("size".to_string()), Some("desc".to_string()))
            .expect_err("legacy size sort");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }
}

#[cfg(test)]
mod pagination_cursor_tests {
    use super::{next_page_token, parse_change_page_request, parse_page_request};
    use crate::dto::PageRequest;

    fn is_numeric_token(value: &str) -> bool {
        value.bytes().all(|byte| byte.is_ascii_digit())
    }

    #[test]
    fn list_page_tokens_are_opaque_and_round_trip() {
        let mut items = vec![1, 2];

        let token = next_page_token(
            &mut items,
            PageRequest {
                limit: 1,
                offset: 0,
            },
        )
        .expect("first page should expose continuation token");

        assert!(!is_numeric_token(&token));
        let parsed = parse_page_request(Some(1), Some(token)).expect("opaque cursor should parse");
        assert_eq!(parsed.offset, 1);
        assert_eq!(items, vec![1]);
    }

    #[test]
    fn list_page_request_rejects_numeric_cursor_alias() {
        let err = parse_page_request(Some(20), Some("20".to_string()))
            .expect_err("numeric cursor is pre-launch pagination debt");

        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn change_page_request_rejects_numeric_cursor_alias() {
        let err = parse_change_page_request(Some(20), Some("20".to_string()))
            .expect_err("change cursor must be an opaque token");

        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }
}
