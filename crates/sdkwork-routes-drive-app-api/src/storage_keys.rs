use crate::error::ProblemDetail;
use crate::validators::validate_object_key;
use axum::http::StatusCode;
use axum::Json;

pub(crate) fn uploader_final_object_key(
    uploader_session_object_key: &str,
    storage_root_prefix: &str,
    standard_object_key: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    let Some(prefix) = uploader_locator_prefix(uploader_session_object_key) else {
        return Ok(None);
    };
    // The standard key already starts with the resolved root prefix. Strip it
    // once so the uploader layout can sit *below* that root instead of
    // duplicating it; an unprefixed key (older resolver) is used as-is.
    let standard_suffix = standard_object_key
        .strip_prefix(storage_root_prefix)
        .and_then(|rest| rest.strip_prefix('/'))
        .unwrap_or(standard_object_key);
    let final_key = format!("{storage_root_prefix}/{prefix}/{standard_suffix}");
    if final_key.len() <= 1024 {
        return validate_object_key(final_key, "objectKey").map(Some);
    }

    let compact_prefix = compact_uploader_locator_prefix(prefix);
    let compact_key = format!("{storage_root_prefix}/{compact_prefix}/{standard_suffix}");
    validate_object_key(compact_key, "objectKey").map(Some)
}

fn uploader_locator_prefix(object_key: &str) -> Option<&str> {
    // Uploader session keys now carry the logical storage root before
    // `sdkwork-drive/uploader/`, so the marker must be located rather than
    // assumed at position 0.
    let marker = "sdkwork-drive/uploader/";
    let start = object_key.find(marker)?;
    let rest = &object_key[start..];
    rest.find("/nodes/")
        .map(|index| rest[..index].trim_end_matches('/'))
}

fn compact_uploader_locator_prefix(prefix: &str) -> String {
    let segments = prefix.split('/').collect::<Vec<_>>();
    let value_after = |marker: &str| {
        segments
            .windows(2)
            .find_map(|window| (window[0] == marker).then_some(window[1]))
            .unwrap_or("unspecified")
    };
    format!(
        "sdkwork-drive/uploader/tenants/{}/organizations/{}/users/{}/spaces/{}/actors/{}/{}/scene/{}/source/{}/profile/{}/dt/{}",
        short_object_key_segment(value_after("tenants"), 64),
        short_object_key_segment(value_after("organizations"), 64),
        short_object_key_segment(value_after("users"), 64),
        short_object_key_segment(value_after("spaces"), 64),
        short_object_key_segment(value_after("actors"), 32),
        short_object_key_segment(
            segments
                .windows(3)
                .find_map(|window| (window[0] == "actors").then_some(window[2]))
                .unwrap_or("unspecified"),
            64,
        ),
        short_object_key_segment(value_after("scene"), 64),
        short_object_key_segment(value_after("source"), 64),
        short_object_key_segment(value_after("profile"), 32),
        short_object_key_segment(value_after("dt"), 16),
    )
}

fn short_object_key_segment(value: &str, max_chars: usize) -> String {
    let mut segment = value
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '@' | '-') {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while segment.contains("--") {
        segment = segment.replace("--", "-");
    }
    let segment = segment.trim_matches('-').to_string();
    let segment = if segment.is_empty() {
        "unspecified".to_string()
    } else {
        segment.chars().take(max_chars).collect()
    };
    if segment == "." || segment == ".." {
        "unspecified".to_string()
    } else {
        segment
    }
}

pub(crate) fn normalize_optional_id(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}
