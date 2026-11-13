//! Pure upload-session rules shared by the service and the HTTP surface.
//!
//! These functions carry the predicates that used to live only in
//! `sdkwork-routes-drive-app-api`: part-list shape, content-type and checksum
//! validation, session-state mutability, the storage-key version encoding, and
//! root-prefix joining. They are pure so both the route (which maps them onto
//! problem details) and the in-process service (which maps them onto
//! [`DriveServiceError`]) apply exactly the same rule, once.
//!
//! Error messages are the operator-facing detail strings the Drive app API
//! already returns, so delegating the route to these helpers does not change a
//! single HTTP response body.

use crate::ports::upload_multipart_storage::DriveMultipartPart;
use crate::DriveServiceError;

/// Validate a `contentType` field.
///
/// # Errors
///
/// Returns [`DriveServiceError::Validation`] when the value is untrimmed,
/// empty, longer than 255 characters, or not a `type/subtype` MIME token pair.
pub fn validate_content_type<'a>(
    value: &'a str,
    field_name: &str,
) -> Result<&'a str, DriveServiceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 255 || trimmed != value {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must be a trimmed MIME type up to 255 characters"
        )));
    }
    let Some((type_part, subtype_part)) = trimmed.split_once('/') else {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must be a valid MIME type"
        )));
    };
    if !is_mime_token(type_part) || !is_mime_token(subtype_part) {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must be a valid MIME type"
        )));
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

/// Validate a `sha256:<64 lowercase hex>` checksum field.
///
/// # Errors
///
/// Returns [`DriveServiceError::Validation`] when the value is untrimmed, lacks
/// the `sha256:` prefix, or does not carry exactly 64 lowercase hex digits.
pub fn validate_sha256_checksum<'a>(
    value: &'a str,
    field_name: &str,
) -> Result<&'a str, DriveServiceError> {
    let trimmed = value.trim();
    let Some(hex) = trimmed.strip_prefix("sha256:") else {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must use sha256:<64 lowercase hex characters>"
        )));
    };
    if trimmed != value
        || hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must use sha256:<64 lowercase hex characters>"
        )));
    }
    Ok(trimmed)
}

/// Validate the completed multipart part list.
///
/// The list must be non-empty, at most 10 000 entries, sorted by strictly
/// ascending unique part numbers in `1..=10_000`, and every part must carry a
/// non-blank ETag. The ordering requirement is what keeps a retried or replayed
/// completion from silently reordering parts at the vendor.
///
/// # Errors
///
/// Returns [`DriveServiceError::Validation`] with the first violated rule.
pub fn validate_completed_multipart_parts(
    parts: &[DriveMultipartPart],
) -> Result<(), DriveServiceError> {
    if parts.is_empty() {
        return Err(DriveServiceError::Validation(
            "parts are required".to_string(),
        ));
    }
    if parts.len() > 10_000 {
        return Err(DriveServiceError::Validation(
            "parts must contain at most 10000 items".to_string(),
        ));
    }

    let mut previous_part_no = 0_u16;
    for part in parts {
        if part.part_no == 0 || part.part_no > 10_000 {
            return Err(DriveServiceError::Validation(
                "partNo must be between 1 and 10000".to_string(),
            ));
        }
        if part.etag.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "etag is required for every part".to_string(),
            ));
        }
        if part.part_no <= previous_part_no {
            return Err(DriveServiceError::Validation(
                "parts must be sorted by ascending unique partNo".to_string(),
            ));
        }
        previous_part_no = part.part_no;
    }

    Ok(())
}

/// Reject a session state that may no longer be mutated by a client.
///
/// # Errors
///
/// `Conflict` for `completing`/`completed`/`aborted`/`expired`, and
/// `Validation` for an unrecognised state value.
pub fn validate_mutable_upload_session(state: &str) -> Result<(), DriveServiceError> {
    match state {
        "created" | "uploading" => Ok(()),
        "completing" | "completed" | "aborted" | "expired" => Err(DriveServiceError::Conflict(
            format!("upload session cannot be modified from {state} state"),
        )),
        _ => Err(DriveServiceError::Validation(
            "upload session state is invalid".to_string(),
        )),
    }
}

/// Reject a session whose deadline passed before the sweeper marked it expired.
///
/// # Errors
///
/// Returns [`DriveServiceError::Conflict`] when `expires_at_epoch_ms` is not
/// strictly after `now_epoch_ms`.
pub fn ensure_upload_session_not_expired(
    expires_at_epoch_ms: i64,
    now_epoch_ms: i64,
) -> Result<(), DriveServiceError> {
    if expires_at_epoch_ms <= now_epoch_ms {
        return Err(DriveServiceError::Conflict(
            "upload session has expired".to_string(),
        ));
    }
    Ok(())
}

/// Validate an object key with the Drive storage-key rules.
///
/// # Errors
///
/// Returns `Validation` for an empty or untrimmed value, one longer than 1024
/// UTF-8 bytes, one containing NUL, one with a leading/trailing slash, or one
/// with an empty, `.` or `..` path segment.
pub fn validate_object_key(value: &str, field_name: &str) -> Result<String, DriveServiceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} is required"
        )));
    }
    if trimmed.len() > 1024 {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must be at most 1024 UTF-8 bytes"
        )));
    }
    if trimmed.as_bytes().contains(&0) {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must not contain NUL bytes"
        )));
    }
    if trimmed.starts_with('/') || trimmed.ends_with('/') {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must not start or end with slash"
        )));
    }
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(DriveServiceError::Validation(format!(
                "{field_name} must not contain empty or period-only path segments"
            )));
        }
    }
    Ok(trimmed.to_string())
}

/// The root prefix a storage target defaults to for a tenant (and optional space).
pub fn default_storage_root_prefix(tenant_id: &str, space_id: Option<&str>) -> String {
    match space_id {
        Some(space_id) => format!("sdkwork-drive/v1/tenants/{tenant_id}/spaces/{space_id}"),
        None => format!("sdkwork-drive/v1/tenants/{tenant_id}"),
    }
}

/// Join a configured storage root prefix with a standard object key.
///
/// # Errors
///
/// Returns `Validation` when either side is not a valid object key or the joined
/// key exceeds the 1024-byte limit.
pub fn join_storage_root_prefix(
    storage_root_prefix: &str,
    object_key: &str,
) -> Result<String, DriveServiceError> {
    let root = validate_object_key(storage_root_prefix, "storageRootPrefix")?;
    let key = validate_object_key(object_key, "objectKey")?;
    validate_object_key(&format!("{root}/{key}"), "objectKey")
}

/// The trailing standard Drive key inside a possibly prefixed object key.
pub fn standard_storage_object_key_suffix(object_key: &str) -> Option<&str> {
    object_key
        .rmatch_indices("sdkwork-drive/v1/t/")
        .next()
        .map(|(index, _)| &object_key[index..])
}

/// Parse the version number encoded in a standard Drive storage object key.
///
/// The version is a structural part of the key, so completion can compare it
/// with the next version number the database will allocate before writing
/// anything. A key that does not match the canonical shape for this tenant,
/// space and node is a conflict, not a validation error: the session recorded a
/// locator that cannot be published as the version it claims to be.
///
/// # Errors
///
/// Returns `Conflict` when the key is not the canonical encoding for the given
/// tenant/space/node.
pub fn parse_storage_object_version_no_from_key(
    object_key: &str,
    tenant_id: &str,
    space_id: &str,
    node_id: &str,
) -> Result<i64, DriveServiceError> {
    use crate::application::storage_key_service::{
        BuildStorageObjectKeyCommand, DriveStorageKeyService,
    };

    let invalid = || {
        DriveServiceError::Conflict(
            "upload session object key is not a standard Drive storage key".to_string(),
        )
    };
    let standard_object_key = standard_storage_object_key_suffix(object_key).ok_or_else(invalid)?;
    let segments: Vec<&str> = standard_object_key.split('/').collect();
    if segments.len() != 16
        || segments[0] != "sdkwork-drive"
        || segments[1] != "v1"
        || segments[2] != "t"
        || segments[4] != "tenants"
        || segments[5] != tenant_id
        || segments[6] != "spaces"
        || segments[7] != space_id
        || segments[8] != "nodes"
        || segments[9] != "n"
        || segments[11] != node_id
        || segments[12] != "versions"
        || segments[15] != "content"
    {
        return Err(invalid());
    }

    let raw_version_no = segments[13];
    if raw_version_no.len() != 10 || !raw_version_no.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid());
    }
    let version_no = raw_version_no.parse::<i64>().map_err(|_| invalid())?;
    if version_no < 1 {
        return Err(invalid());
    }

    let expected = DriveStorageKeyService::build_object_key(BuildStorageObjectKeyCommand {
        tenant_id,
        space_id,
        node_id,
        version_no,
        object_id: segments[14],
    })
    .map_err(|_| invalid())?;
    if expected != standard_object_key {
        return Err(invalid());
    }

    Ok(version_no)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(part_no: u16, etag: &str) -> DriveMultipartPart {
        DriveMultipartPart {
            part_no,
            etag: etag.to_string(),
        }
    }

    #[test]
    fn accepts_ascending_unique_parts() {
        validate_completed_multipart_parts(&[part(1, "a"), part(2, "b")])
            .expect("ascending parts should be accepted");
    }

    #[test]
    fn rejects_empty_part_list() {
        let error = validate_completed_multipart_parts(&[]).expect_err("empty list");
        assert_eq!(error.message(), "parts are required");
    }

    #[test]
    fn rejects_out_of_order_parts() {
        let error = validate_completed_multipart_parts(&[part(2, "b"), part(1, "a")])
            .expect_err("descending parts");
        assert_eq!(
            error.message(),
            "parts must be sorted by ascending unique partNo"
        );
    }

    #[test]
    fn rejects_blank_etag() {
        let error = validate_completed_multipart_parts(&[part(1, "  ")]).expect_err("blank etag");
        assert_eq!(error.message(), "etag is required for every part");
    }

    #[test]
    fn rejects_zero_part_number() {
        let error = validate_completed_multipart_parts(&[part(0, "a")]).expect_err("zero part");
        assert_eq!(error.message(), "partNo must be between 1 and 10000");
    }

    #[test]
    fn mutable_states_are_created_and_uploading() {
        assert!(validate_mutable_upload_session("created").is_ok());
        assert!(validate_mutable_upload_session("uploading").is_ok());
        for state in ["completing", "completed", "aborted", "expired"] {
            assert!(validate_mutable_upload_session(state).is_err(), "{state}");
        }
    }

    #[test]
    fn expired_session_is_rejected_at_its_deadline() {
        let error = ensure_upload_session_not_expired(1_000, 1_000).expect_err("expired");
        assert_eq!(error.message(), "upload session has expired");
        ensure_upload_session_not_expired(1_001, 1_000).expect("not expired");
    }

    #[test]
    fn rejects_period_only_object_key_segments() {
        let error =
            validate_object_key("objects/../content", "objectKey").expect_err("dot segment");
        assert_eq!(
            error.message(),
            "objectKey must not contain empty or period-only path segments"
        );
    }

    #[test]
    fn joins_root_prefix_and_standard_key() {
        let joined = join_storage_root_prefix(
            "sdkwork-drive/v1/tenants/t1/spaces/s1",
            "sdkwork-drive/v1/t/aa/tenants/t1/spaces/s1/nodes/n/bb/n1/versions/0000000001/o1/content",
        )
        .expect("join");
        assert!(joined.starts_with("sdkwork-drive/v1/tenants/t1/spaces/s1/sdkwork-drive/v1/t/"));
    }

    #[test]
    fn parses_version_from_canonical_key() {
        let key =
            crate::application::storage_key_service::DriveStorageKeyService::build_object_key(
                crate::application::storage_key_service::BuildStorageObjectKeyCommand {
                    tenant_id: "t1",
                    space_id: "s1",
                    node_id: "n1",
                    version_no: 7,
                    object_id: "o1",
                },
            )
            .expect("build key");
        let parsed = parse_storage_object_version_no_from_key(&key, "t1", "s1", "n1")
            .expect("parse version");
        assert_eq!(parsed, 7);
        assert!(parse_storage_object_version_no_from_key(&key, "t1", "s1", "n2").is_err());
    }

    #[test]
    fn content_type_requires_single_mime_pair() {
        assert_eq!(
            validate_content_type("application/json", "contentType").expect("valid"),
            "application/json"
        );
        assert!(validate_content_type(" application/json", "contentType").is_err());
        assert!(validate_content_type("application", "contentType").is_err());
    }

    #[test]
    fn checksum_requires_lowercase_sha256_prefix() {
        let valid = format!("sha256:{}", "a".repeat(64));
        assert!(validate_sha256_checksum(&valid, "checksumSha256Hex").is_ok());
        assert!(validate_sha256_checksum(&"A".repeat(64), "checksumSha256Hex").is_err());
    }
}
