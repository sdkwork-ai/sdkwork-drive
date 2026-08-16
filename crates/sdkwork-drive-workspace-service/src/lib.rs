pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod ports;

pub mod uploader {
    pub use crate::application::uploader_service::{
        CompleteStoredUploaderUploadCommand, DriveUploaderService, MarkUploaderPartUploadedCommand,
        PrepareUploaderUploadCommand, UploadBytesCommand, UploaderActor, UploaderRetention,
        UploaderTarget,
    };
    pub use crate::domain::uploader::{DriveUploadItem, DriveUploadPart};
}

use sdkwork_utils_rust::sha256_hash;

/// Enable the canonical process-local database pool before any Drive bootstrap.
pub fn enable_process_shared_database_pool() {
    sdkwork_database_sqlx::enable_process_shared_database_pool();
}

pub fn drive_share_token_hash(token: &str) -> String {
    format!("sha256:{}", sha256_hash(token.trim().as_bytes()))
}

pub fn drive_share_access_code_hash(access_code: &str) -> String {
    format!("sha256:{}", sha256_hash(access_code.trim().as_bytes()))
}

pub const MIN_SHARE_LINK_ACCESS_CODE_LEN: usize = 4;
pub const MAX_SHARE_LINK_ACCESS_CODE_LEN: usize = 64;

pub fn validate_share_link_access_code(access_code: &str) -> Result<(), DriveServiceError> {
    let trimmed = access_code.trim();
    if trimmed.len() < MIN_SHARE_LINK_ACCESS_CODE_LEN {
        return Err(DriveServiceError::Validation(format!(
            "share link access code must be at least {MIN_SHARE_LINK_ACCESS_CODE_LEN} characters"
        )));
    }
    if trimmed.len() > MAX_SHARE_LINK_ACCESS_CODE_LEN {
        return Err(DriveServiceError::Validation(format!(
            "share link access code must be at most {MAX_SHARE_LINK_ACCESS_CODE_LEN} characters"
        )));
    }
    if trimmed.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(DriveServiceError::Validation(
            "share link access code contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

pub fn share_link_access_code_matches(stored_hash: Option<&str>, provided: Option<&str>) -> bool {
    let stored_hash = stored_hash.map(str::trim).filter(|value| !value.is_empty());
    let provided = provided.map(str::trim).filter(|value| !value.is_empty());
    match (stored_hash, provided) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(stored), Some(code)) => stored == drive_share_access_code_hash(code),
    }
}

pub fn generate_share_link_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

pub const MIN_SHARE_LINK_TOKEN_LEN: usize = 32;

pub fn validate_share_link_token(token: &str) -> Result<(), DriveServiceError> {
    let trimmed = token.trim();
    if trimmed.len() < MIN_SHARE_LINK_TOKEN_LEN {
        return Err(DriveServiceError::Validation(format!(
            "share link token must be at least {MIN_SHARE_LINK_TOKEN_LEN} characters"
        )));
    }
    if !trimmed
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(DriveServiceError::Validation(
            "share link token contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveServiceError {
    Validation(String),
    Conflict(String),
    NotFound(String),
    PermissionDenied(String),
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::drive_share_token_hash;

    #[test]
    fn drive_share_token_hash_uses_sha256_hex_digest() {
        let digest = drive_share_token_hash("abc");

        assert_eq!(
            digest,
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn drive_share_token_hash_normalizes_surrounding_whitespace() {
        assert_eq!(
            drive_share_token_hash("abc"),
            drive_share_token_hash(" abc\n")
        );
    }

    #[test]
    fn share_link_access_code_requires_match_when_hash_is_configured() {
        let hash = super::drive_share_access_code_hash("1234");
        assert!(super::share_link_access_code_matches(
            Some(&hash),
            Some("1234")
        ));
        assert!(!super::share_link_access_code_matches(
            Some(&hash),
            Some("5678")
        ));
        assert!(!super::share_link_access_code_matches(Some(&hash), None));
        assert!(super::share_link_access_code_matches(None, None));
        assert!(super::share_link_access_code_matches(None, Some("1234")));
    }
}
