//! Opaque Drive API pagination cursor helpers.

use sdkwork_utils_rust::{base64url_decode, base64url_encode};

const CURSOR_PREFIX: &str = "sdwdrvc1_";
const CURSOR_VERSION: &str = "v1";
const OFFSET_CURSOR_KIND: &str = "drive-offset";
const CHANGE_SEQUENCE_CURSOR_KIND: &str = "drive-change-sequence";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrivePaginationCursorError {
    InvalidToken,
    NegativeValue,
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
        decode_change_sequence_cursor, decode_offset_cursor, encode_change_sequence_cursor,
        encode_offset_cursor, DrivePaginationCursorError,
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
}
