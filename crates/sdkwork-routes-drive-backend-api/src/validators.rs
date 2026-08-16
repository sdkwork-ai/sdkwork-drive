use crate::dto::OffsetPage;
use crate::error::{problem, ProblemDetail, SdkWorkResultCode};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_contract::api::pagination_cursor::{decode_offset_cursor, encode_offset_cursor};
use sdkwork_utils_rust::{DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE};

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
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} is required"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(trimmed)
}

pub(crate) fn parse_offset_page(
    page_size: Option<i64>,
    page_token: Option<String>,
) -> Result<OffsetPage, (StatusCode, Json<ProblemDetail>)> {
    let limit = validate_page_size_i64(
        page_size,
        i64::from(DEFAULT_LIST_PAGE_SIZE),
        1,
        i64::from(MAX_LIST_PAGE_SIZE),
        "page_size",
    )?;
    let offset = decode_offset_cursor(page_token.as_deref()).map_err(|_| {
        problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "cursor is invalid",
            SdkWorkResultCode::ValidationError,
        )
    })?;
    Ok(OffsetPage { limit, offset })
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

pub(crate) fn validate_page_u32(
    value: Option<u32>,
    default_value: u32,
    min_value: u32,
    max_value: u32,
    field_name: &str,
) -> Result<u32, (StatusCode, Json<ProblemDetail>)> {
    let page = value.unwrap_or(default_value);
    if page < min_value || page > max_value {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("{field_name} must be between {min_value} and {max_value}"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(page)
}

pub(crate) fn validate_page_size_u32(
    value: Option<u32>,
    default_value: u32,
    min_value: u32,
    max_value: u32,
    field_name: &str,
) -> Result<u32, (StatusCode, Json<ProblemDetail>)> {
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

pub(crate) fn next_page_token<T>(items: &mut Vec<T>, page: OffsetPage) -> Option<String> {
    if items.len() as i64 > page.limit {
        items.pop();
        encode_offset_cursor(page.offset + page.limit)
    } else {
        None
    }
}

pub(crate) fn validate_lifecycle_status(
    status: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(status, "active" | "deleted") {
        return Ok(());
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "lifecycleStatus is invalid",
        SdkWorkResultCode::ValidationError,
    ))
}

pub(crate) fn validate_download_package_state(
    state: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(state, "creating" | "ready" | "failed" | "expired") {
        return Ok(());
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "state is invalid",
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

pub(crate) fn validate_label_color(color: &str) -> Result<&str, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = color.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() == 7
        && bytes[0] == b'#'
        && bytes[1..].iter().all(|byte| byte.is_ascii_hexdigit())
    {
        return Ok(trimmed);
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "color is invalid",
        SdkWorkResultCode::ValidationError,
    ))
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
