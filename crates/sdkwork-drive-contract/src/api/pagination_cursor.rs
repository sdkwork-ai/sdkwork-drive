//! Opaque Drive API pagination cursor helpers.

use sdkwork_utils_rust::{base64url_decode, base64url_encode};

const CURSOR_PREFIX: &str = "sdwdrvc1_";
const CURSOR_VERSION: &str = "v1";
const OFFSET_CURSOR_KIND: &str = "drive-offset";
const CHANGE_SEQUENCE_CURSOR_KIND: &str = "drive-change-sequence";
const FAVORITE_UPDATED_CURSOR_KIND: &str = "drive-favorite-updated";
const CURSOR_FIELD_SEPARATOR: char = '|';

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrivePaginationCursorError {
    InvalidToken,
    NegativeValue,
}

/// Keyset position inside the drive favorites list: the favorite row's
/// `updated_at` (epoch microseconds) plus the node id tiebreaker of the last
/// row on the previously served page (`ORDER BY updated_at DESC, id ASC`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FavoriteUpdatedCursor {
    pub updated_at_epoch_micros: i64,
    pub node_id: String,
}

pub fn encode_offset_cursor(offset: i64) -> Option<String> {
    encode_i64_cursor(OFFSET_CURSOR_KIND, offset)
}

pub fn decode_offset_cursor(cursor: Option<&str>) -> Result<i64, DrivePaginationCursorError> {
    decode_i64_cursor(OFFSET_CURSOR_KIND, cursor)
}

pub fn encode_change_sequence_cursor(sequence_no: i64) -> Option<String> {
    encode_i64_cursor(CHANGE_SEQUENCE_CURSOR_KIND, sequence_no)
}

pub fn decode_change_sequence_cursor(
    cursor: Option<&str>,
) -> Result<i64, DrivePaginationCursorError> {
    decode_i64_cursor(CHANGE_SEQUENCE_CURSOR_KIND, cursor)
}

/// Encode the favorites keyset position. `None` only when the node id is blank
/// or carries the reserved field separator, which real snowflake ids never do.
pub fn encode_favorite_updated_cursor(
    updated_at_epoch_micros: i64,
    node_id: &str,
) -> Option<String> {
    let trimmed = node_id.trim();
    if trimmed.is_empty() || trimmed.contains(CURSOR_FIELD_SEPARATOR) {
        return None;
    }
    let payload = format!(
        "{CURSOR_VERSION}:{FAVORITE_UPDATED_CURSOR_KIND}:{updated_at_epoch_micros}{CURSOR_FIELD_SEPARATOR}{trimmed}"
    );
    Some(format!(
        "{CURSOR_PREFIX}{}",
        base64url_encode(payload.as_bytes())
    ))
}

/// Decode a favorites keyset cursor. An absent or blank cursor is the first
/// page (`Ok(None)`), matching the offset cursor convention.
pub fn decode_favorite_updated_cursor(
    cursor: Option<&str>,
) -> Result<Option<FavoriteUpdatedCursor>, DrivePaginationCursorError> {
    let Some(trimmed) = cursor.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let encoded = trimmed
        .strip_prefix(CURSOR_PREFIX)
        .ok_or(DrivePaginationCursorError::InvalidToken)?;
    let decoded = base64url_decode(encoded).ok_or(DrivePaginationCursorError::InvalidToken)?;
    let payload =
        String::from_utf8(decoded).map_err(|_| DrivePaginationCursorError::InvalidToken)?;
    let mut parts = payload.splitn(3, ':');
    let Some(version) = parts.next() else {
        return Err(DrivePaginationCursorError::InvalidToken);
    };
    let Some(decoded_kind) = parts.next() else {
        return Err(DrivePaginationCursorError::InvalidToken);
    };
    let Some(value_part) = parts.next() else {
        return Err(DrivePaginationCursorError::InvalidToken);
    };
    if version != CURSOR_VERSION || decoded_kind != FAVORITE_UPDATED_CURSOR_KIND {
        return Err(DrivePaginationCursorError::InvalidToken);
    }
    let mut fields = value_part.splitn(2, CURSOR_FIELD_SEPARATOR);
    let Some(updated_at_epoch_micros) = fields.next().and_then(|value| value.parse::<i64>().ok())
    else {
        return Err(DrivePaginationCursorError::InvalidToken);
    };
    let Some(node_id) = fields.next().map(str::trim).filter(|value| !value.is_empty())
    else {
        return Err(DrivePaginationCursorError::InvalidToken);
    };
    Ok(Some(FavoriteUpdatedCursor {
        updated_at_epoch_micros,
        node_id: node_id.to_string(),
    }))
}

fn encode_i64_cursor(kind: &str, value: i64) -> Option<String> {
    if value < 0 {
        return None;
    }
    let payload = format!("{CURSOR_VERSION}:{kind}:{value}");
    Some(format!(
        "{CURSOR_PREFIX}{}",
        base64url_encode(payload.as_bytes())
    ))
}

fn decode_i64_cursor(kind: &str, cursor: Option<&str>) -> Result<i64, DrivePaginationCursorError> {
    let Some(trimmed) = cursor.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(0);
    };
    let encoded = trimmed
        .strip_prefix(CURSOR_PREFIX)
        .ok_or(DrivePaginationCursorError::InvalidToken)?;
    let decoded = base64url_decode(encoded).ok_or(DrivePaginationCursorError::InvalidToken)?;
    let payload =
        String::from_utf8(decoded).map_err(|_| DrivePaginationCursorError::InvalidToken)?;
    let mut parts = payload.split(':');
    let Some(version) = parts.next() else {
        return Err(DrivePaginationCursorError::InvalidToken);
    };
    let Some(decoded_kind) = parts.next() else {
        return Err(DrivePaginationCursorError::InvalidToken);
    };
    let Some(value) = parts.next() else {
        return Err(DrivePaginationCursorError::InvalidToken);
    };
    if parts.next().is_some() || version != CURSOR_VERSION || decoded_kind != kind {
        return Err(DrivePaginationCursorError::InvalidToken);
    }
    let value = value
        .parse::<i64>()
        .map_err(|_| DrivePaginationCursorError::InvalidToken)?;
    if value < 0 {
        return Err(DrivePaginationCursorError::NegativeValue);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_change_sequence_cursor, decode_favorite_updated_cursor, decode_offset_cursor,
        encode_change_sequence_cursor, encode_favorite_updated_cursor, encode_offset_cursor,
        DrivePaginationCursorError,
    };

    fn is_numeric_token(value: &str) -> bool {
        value.bytes().all(|byte| byte.is_ascii_digit())
    }

    #[test]
    fn offset_cursor_is_opaque_and_round_trips() {
        let token = encode_offset_cursor(20).expect("non-negative offset should encode");

        assert!(!is_numeric_token(&token));
        assert_eq!(decode_offset_cursor(Some(&token)), Ok(20));
    }

    #[test]
    fn sequence_cursor_is_opaque_and_round_trips() {
        let token = encode_change_sequence_cursor(7).expect("non-negative sequence should encode");

        assert!(!is_numeric_token(&token));
        assert_eq!(decode_change_sequence_cursor(Some(&token)), Ok(7));
    }

    #[test]
    fn raw_numeric_cursor_is_rejected() {
        assert_eq!(
            decode_offset_cursor(Some("20")),
            Err(DrivePaginationCursorError::InvalidToken)
        );
    }

    #[test]
    fn cursor_kinds_are_not_interchangeable() {
        let token = encode_change_sequence_cursor(7).expect("sequence should encode");

        assert_eq!(
            decode_offset_cursor(Some(&token)),
            Err(DrivePaginationCursorError::InvalidToken)
        );
    }

    #[test]
    fn favorite_updated_cursor_is_opaque_and_round_trips() {
        let token = encode_favorite_updated_cursor(1_781_000_000_000_123, "node_123")
            .expect("snowflake node id should encode");

        assert!(!is_numeric_token(&token));
        assert_eq!(
            decode_favorite_updated_cursor(Some(&token)),
            Ok(Some(super::FavoriteUpdatedCursor {
                updated_at_epoch_micros: 1_781_000_000_000_123,
                node_id: "node_123".to_string(),
            }))
        );
    }

    #[test]
    fn favorite_updated_cursor_absent_is_first_page() {
        assert_eq!(decode_favorite_updated_cursor(None), Ok(None));
        assert_eq!(decode_favorite_updated_cursor(Some("  ")), Ok(None));
    }

    #[test]
    fn favorite_updated_cursor_rejects_other_kinds_and_garbage() {
        let offset_token = encode_offset_cursor(4).expect("offset should encode");
        assert_eq!(
            decode_favorite_updated_cursor(Some(&offset_token)),
            Err(DrivePaginationCursorError::InvalidToken)
        );
        assert_eq!(
            decode_favorite_updated_cursor(Some("not-a-cursor")),
            Err(DrivePaginationCursorError::InvalidToken)
        );
    }

    #[test]
    fn favorite_updated_cursor_rejects_missing_node_id() {
        let token = encode_favorite_updated_cursor(5, "  ");
        assert!(token.is_none());
        assert_eq!(
            decode_favorite_updated_cursor(Some("sdwdrvc1_gibberish")),
            Err(DrivePaginationCursorError::InvalidToken)
        );
    }
}
