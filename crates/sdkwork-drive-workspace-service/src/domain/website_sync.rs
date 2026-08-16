use std::collections::HashSet;

use sdkwork_utils_rust::sha256_hash;

use crate::DriveServiceError;

pub const MAX_WEBSITE_SYNC_FILES: usize = 100_000;
pub const MAX_WEBSITE_SYNC_NODES: usize = 200_000;
pub const MAX_WEBSITE_SYNC_DEPTH: i64 = 64;
pub const MAX_WEBSITE_SYNC_PATH_BYTES: usize = 4_096;
pub const MAX_WEBSITE_SYNC_TOTAL_BYTES: i64 = 1_099_511_627_776;

const RESERVED_PATH_SEGMENTS: [&str; 7] = [
    ".sdkwork",
    ".trash",
    "trash",
    ".staging",
    "staging",
    ".versions",
    "versions",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveWebsiteSyncStatus {
    Created,
    Uploading,
    Ready,
    Validating,
    Active,
    Completed,
    Failed,
    Aborted,
    Expired,
}

impl DriveWebsiteSyncStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Uploading => "uploading",
            Self::Ready => "ready",
            Self::Validating => "validating",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Aborted => "aborted",
            Self::Expired => "expired",
        }
    }

    pub fn try_from_str(raw: &str) -> Option<Self> {
        match raw {
            "created" => Some(Self::Created),
            "uploading" => Some(Self::Uploading),
            "ready" => Some(Self::Ready),
            "validating" => Some(Self::Validating),
            "active" => Some(Self::Active),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "aborted" => Some(Self::Aborted),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveWebsiteSync {
    pub id: String,
    pub tenant_id: String,
    pub website_root_id: String,
    pub website_root_uuid: String,
    pub space_id: String,
    pub idempotency_key: String,
    pub expected_root_version: i64,
    pub expected_generation: i64,
    pub staging_node_id: String,
    pub manifest_sha256: String,
    pub manifest_file_count: i64,
    pub manifest_total_bytes: i64,
    pub uploaded_file_count: i64,
    pub uploaded_total_bytes: i64,
    pub status: DriveWebsiteSyncStatus,
    pub expires_at: String,
    pub validated_at: Option<String>,
    pub activated_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_code: Option<String>,
    pub error_summary: Option<String>,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveWebsiteGeneration {
    pub generation_no: i64,
    pub root_node_id: String,
    pub manifest_sha256: Option<String>,
    pub file_count: i64,
    pub total_bytes: i64,
    pub generation_status: String,
    pub activated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveWebsiteSyncTreeEntry {
    pub relative_path: String,
    pub depth: i64,
    pub node_type: String,
    pub content_state: String,
    pub content_length: Option<i64>,
    pub checksum_sha256_hex: Option<String>,
    pub shortcut_target_node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveWebsiteManifestSummary {
    pub sha256: String,
    pub file_count: i64,
    pub total_bytes: i64,
}

pub fn validate_website_sync_tree(
    entries: &[DriveWebsiteSyncTreeEntry],
) -> Result<DriveWebsiteManifestSummary, DriveServiceError> {
    if entries.len() > MAX_WEBSITE_SYNC_NODES {
        return Err(validation("WEBSITE_SYNC_NODE_LIMIT_EXCEEDED"));
    }

    let mut canonical_paths = HashSet::with_capacity(entries.len());
    let mut manifest_entries = Vec::new();
    let mut total_bytes = 0_i64;
    let mut has_index = false;

    for entry in entries {
        validate_tree_entry_path(entry)?;
        if entry.depth > MAX_WEBSITE_SYNC_DEPTH {
            return Err(validation("WEBSITE_SYNC_DEPTH_LIMIT_EXCEEDED"));
        }
        let collision_key = entry.relative_path.to_lowercase();
        if !canonical_paths.insert(collision_key) {
            return Err(validation("WEBSITE_SYNC_CASE_COLLISION"));
        }
        if entry.shortcut_target_node_id.is_some()
            || !matches!(entry.node_type.as_str(), "file" | "folder")
        {
            return Err(validation("WEBSITE_SYNC_UNSUPPORTED_NODE_TYPE"));
        }
        if entry.node_type == "folder" {
            continue;
        }
        if entry.content_state != "ready" {
            return Err(validation("WEBSITE_SYNC_FILE_NOT_READY"));
        }
        let content_length = entry
            .content_length
            .filter(|value| *value >= 0)
            .ok_or_else(|| validation("WEBSITE_SYNC_FILE_LENGTH_INVALID"))?;
        let checksum = entry
            .checksum_sha256_hex
            .as_deref()
            .filter(|value| is_sha256(value))
            .ok_or_else(|| validation("WEBSITE_SYNC_FILE_CHECKSUM_INVALID"))?;
        total_bytes = total_bytes
            .checked_add(content_length)
            .ok_or_else(|| validation("WEBSITE_SYNC_TOTAL_BYTES_OVERFLOW"))?;
        if total_bytes > MAX_WEBSITE_SYNC_TOTAL_BYTES {
            return Err(validation("WEBSITE_SYNC_TOTAL_BYTES_LIMIT_EXCEEDED"));
        }
        has_index |= entry.relative_path == "index.html";
        manifest_entries.push((entry.relative_path.as_str(), content_length, checksum));
    }

    if manifest_entries.len() > MAX_WEBSITE_SYNC_FILES {
        return Err(validation("WEBSITE_SYNC_FILE_LIMIT_EXCEEDED"));
    }
    if !has_index {
        return Err(validation("WEBSITE_SYNC_INDEX_REQUIRED"));
    }

    manifest_entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
    let mut canonical_manifest = Vec::new();
    for (path, content_length, checksum) in manifest_entries {
        canonical_manifest.extend_from_slice(path.as_bytes());
        canonical_manifest.push(0);
        canonical_manifest.extend_from_slice(content_length.to_string().as_bytes());
        canonical_manifest.push(0);
        canonical_manifest.extend_from_slice(checksum.as_bytes());
        canonical_manifest.push(b'\n');
    }

    Ok(DriveWebsiteManifestSummary {
        sha256: format!("sha256:{}", sha256_hash(&canonical_manifest)),
        file_count: canonical_manifest_file_count(&canonical_manifest),
        total_bytes,
    })
}

fn validate_tree_entry_path(entry: &DriveWebsiteSyncTreeEntry) -> Result<(), DriveServiceError> {
    let path = entry.relative_path.as_str();
    if path.is_empty()
        || path.len() > MAX_WEBSITE_SYNC_PATH_BYTES
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(validation("WEBSITE_SYNC_PATH_INVALID"));
    }
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() as i64 != entry.depth
        || segments.iter().any(|segment| {
            segment.is_empty()
                || matches!(*segment, "." | "..")
                || RESERVED_PATH_SEGMENTS
                    .iter()
                    .any(|reserved| segment.eq_ignore_ascii_case(reserved))
        })
    {
        return Err(validation("WEBSITE_SYNC_PATH_INVALID"));
    }
    Ok(())
}

fn canonical_manifest_file_count(manifest: &[u8]) -> i64 {
    manifest.iter().filter(|byte| **byte == b'\n').count() as i64
}

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validation(code: &str) -> DriveServiceError {
    DriveServiceError::Validation(code.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, length: i64, digest_character: char) -> DriveWebsiteSyncTreeEntry {
        DriveWebsiteSyncTreeEntry {
            relative_path: path.to_string(),
            depth: path.split('/').count() as i64,
            node_type: "file".to_string(),
            content_state: "ready".to_string(),
            content_length: Some(length),
            checksum_sha256_hex: Some(format!(
                "sha256:{}",
                digest_character.to_string().repeat(64)
            )),
            shortcut_target_node_id: None,
        }
    }

    #[test]
    fn manifest_is_stable_and_requires_index() {
        let first = validate_website_sync_tree(&[
            file("assets/app.js", 12, 'b'),
            file("index.html", 7, 'a'),
        ])
        .expect("valid website tree should hash");
        let second = validate_website_sync_tree(&[
            file("index.html", 7, 'a'),
            file("assets/app.js", 12, 'b'),
        ])
        .expect("manifest order should be canonical");

        assert_eq!(first, second);
        assert_eq!(first.file_count, 2);
        assert_eq!(first.total_bytes, 19);
        assert!(matches!(
            validate_website_sync_tree(&[file("main.js", 1, 'c')]),
            Err(DriveServiceError::Validation(code)) if code == "WEBSITE_SYNC_INDEX_REQUIRED"
        ));
    }

    #[test]
    fn manifest_rejects_case_collisions_and_reserved_paths() {
        assert!(matches!(
            validate_website_sync_tree(&[
                file("index.html", 1, 'a'),
                file("Assets/app.js", 1, 'b'),
                file("assets/app.js", 1, 'c'),
            ]),
            Err(DriveServiceError::Validation(code)) if code == "WEBSITE_SYNC_CASE_COLLISION"
        ));
        assert!(matches!(
            validate_website_sync_tree(&[
                file("index.html", 1, 'a'),
                file(".staging/private.txt", 1, 'b'),
            ]),
            Err(DriveServiceError::Validation(code)) if code == "WEBSITE_SYNC_PATH_INVALID"
        ));
    }
}
