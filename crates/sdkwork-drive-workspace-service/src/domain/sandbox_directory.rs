use sdkwork_utils_rust::sha256_hash;
use std::fmt;

pub const MAX_SANDBOX_DIRECTORY_PAGE_SIZE: usize = 1_000;
pub const MAX_SANDBOX_LOGICAL_PATH_BYTES: usize = 4096;
pub const MAX_SANDBOX_ENTRY_NAME_BYTES: usize = 255;
pub const MAX_SANDBOX_CURSOR_BYTES: usize = 2048;
pub const MAX_SANDBOX_FILE_CONTENT_BYTES: usize = 4 * 1024 * 1024;
pub const MIN_SANDBOX_IDEMPOTENCY_KEY_BYTES: usize = 8;
pub const MAX_SANDBOX_IDEMPOTENCY_KEY_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxDirectoryAccess {
    Read,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxEntryKind {
    Directory,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxDirectoryEntry {
    pub id: String,
    pub sandbox_id: String,
    pub parent_id: String,
    pub parent_logical_path: String,
    pub name: String,
    pub kind: SandboxEntryKind,
    pub logical_path: String,
    pub revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxDirectoryPage {
    pub items: Vec<SandboxDirectoryEntry>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxDirectoryPageRequest {
    pub page_size: usize,
    pub cursor: Option<String>,
}

impl SandboxDirectoryPageRequest {
    pub fn new(page_size: usize, cursor: Option<String>) -> Result<Self, SandboxInputError> {
        if !(1..=MAX_SANDBOX_DIRECTORY_PAGE_SIZE).contains(&page_size) {
            return Err(SandboxInputError::InvalidPageSize);
        }
        if cursor
            .as_deref()
            .is_some_and(|value| value.is_empty() || value.len() > MAX_SANDBOX_CURSOR_BYTES)
        {
            return Err(SandboxInputError::InvalidCursor);
        }
        Ok(Self { page_size, cursor })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SandboxLogicalPath(String);

impl SandboxLogicalPath {
    pub fn root() -> Self {
        Self(String::new())
    }

    pub fn parse(value: &str) -> Result<Self, SandboxInputError> {
        if value.len() > MAX_SANDBOX_LOGICAL_PATH_BYTES || value.chars().any(char::is_control) {
            return Err(SandboxInputError::InvalidLogicalPath);
        }
        if value.is_empty() {
            return Ok(Self::root());
        }
        if value.starts_with('/')
            || value.ends_with('/')
            || value.contains('\\')
            || value
                .split('/')
                .any(|segment| SandboxEntryName::parse(segment).is_err())
        {
            return Err(SandboxInputError::InvalidLogicalPath);
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn join(&self, name: &SandboxEntryName) -> Self {
        if self.0.is_empty() {
            Self(name.0.clone())
        } else {
            Self(format!("{}/{}", self.0, name.0))
        }
    }

    pub fn split_entry(&self) -> Result<(Self, SandboxEntryName), SandboxInputError> {
        if self.0.is_empty() {
            return Err(SandboxInputError::InvalidLogicalPath);
        }
        let (parent, name) = self
            .0
            .rsplit_once('/')
            .map_or(("", self.0.as_str()), |(parent, name)| (parent, name));
        Ok((Self(parent.to_string()), SandboxEntryName::parse(name)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxFileContent {
    pub entry: SandboxDirectoryEntry,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SandboxEntryName(String);

impl SandboxEntryName {
    pub fn parse(value: &str) -> Result<Self, SandboxInputError> {
        if value.is_empty()
            || value.trim() != value
            || value.len() > MAX_SANDBOX_ENTRY_NAME_BYTES
            || matches!(value, "." | "..")
            || value.ends_with('.')
            || value.to_ascii_lowercase().starts_with(".sdkwork-")
            || value.chars().any(|character| {
                character.is_control()
                    || matches!(
                        character,
                        '/' | '\\' | '<' | '>' | ':' | '"' | '|' | '?' | '*'
                    )
            })
            || is_windows_reserved_name(value)
        {
            return Err(SandboxInputError::InvalidEntryName);
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SandboxIdempotencyKey(String);

impl SandboxIdempotencyKey {
    pub fn parse(value: &str) -> Result<Self, SandboxInputError> {
        if !(MIN_SANDBOX_IDEMPOTENCY_KEY_BYTES..=MAX_SANDBOX_IDEMPOTENCY_KEY_BYTES)
            .contains(&value.len())
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'@' | b'-')
            })
        {
            return Err(SandboxInputError::InvalidIdempotencyKey);
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn sandbox_idempotency_key_hash(scope: &str, key: &SandboxIdempotencyKey) -> String {
    sha256_hash(format!("{scope}\0{}", key.as_str()).as_bytes())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxInputError {
    InvalidLogicalPath,
    InvalidEntryName,
    InvalidPageSize,
    InvalidCursor,
    InvalidIdempotencyKey,
}

impl fmt::Display for SandboxInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidLogicalPath => "sandbox logical path is invalid",
            Self::InvalidEntryName => "sandbox entry name is invalid",
            Self::InvalidPageSize => "sandbox page size must be in range [1, 1000]",
            Self::InvalidCursor => "sandbox cursor is invalid",
            Self::InvalidIdempotencyKey => "sandbox idempotency key is invalid",
        };
        formatter.write_str(message)
    }
}

fn is_windows_reserved_name(value: &str) -> bool {
    let stem = value.split('.').next().unwrap_or(value);
    let upper = stem.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || matches_reserved_numbered_name(&upper, "COM")
        || matches_reserved_numbered_name(&upper, "LPT")
}

fn matches_reserved_numbered_name(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|suffix| matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_pages_allow_the_documented_file_browser_limit() {
        assert!(SandboxDirectoryPageRequest::new(1_000, None).is_ok());
        assert_eq!(
            SandboxDirectoryPageRequest::new(1_001, None),
            Err(SandboxInputError::InvalidPageSize)
        );
    }

    #[test]
    fn logical_paths_use_canonical_forward_slash_segments() {
        assert_eq!(SandboxLogicalPath::parse("").expect("root").as_str(), "");
        assert_eq!(
            SandboxLogicalPath::parse("projects/demo/src")
                .expect("path")
                .as_str(),
            "projects/demo/src"
        );
        for invalid in [
            "/root",
            "root/",
            "root//child",
            "root/../escape",
            "root\\child",
        ] {
            assert_eq!(
                SandboxLogicalPath::parse(invalid),
                Err(SandboxInputError::InvalidLogicalPath)
            );
        }
    }

    #[test]
    fn entry_paths_split_into_a_canonical_parent_and_portable_name() {
        let (parent, name) = SandboxLogicalPath::parse("projects/demo/src/main.rs")
            .expect("entry path")
            .split_entry()
            .expect("split entry");
        assert_eq!(parent.as_str(), "projects/demo/src");
        assert_eq!(name.as_str(), "main.rs");
        assert_eq!(
            SandboxLogicalPath::root().split_entry(),
            Err(SandboxInputError::InvalidLogicalPath)
        );
    }

    #[test]
    fn entry_names_are_portable_across_supported_desktop_operating_systems() {
        assert!(SandboxEntryName::parse("src").is_ok());
        assert!(SandboxEntryName::parse(".config").is_ok());
        for invalid in [
            "",
            ".",
            "..",
            " trailing",
            "trailing ",
            "name.",
            "a/b",
            "a\\b",
            "CON",
            "com1.txt",
            ".sdkwork-tmp-reserved",
        ] {
            assert_eq!(
                SandboxEntryName::parse(invalid),
                Err(SandboxInputError::InvalidEntryName)
            );
        }
    }

    #[test]
    fn idempotency_keys_are_bounded_opaque_identifiers() {
        assert!(SandboxIdempotencyKey::parse("request-1234").is_ok());
        for invalid in ["short", "contains space", "contains/slash"] {
            assert_eq!(
                SandboxIdempotencyKey::parse(invalid),
                Err(SandboxInputError::InvalidIdempotencyKey)
            );
        }
    }

    #[test]
    fn idempotency_hashes_are_scoped_without_persisting_the_raw_key() {
        let key = SandboxIdempotencyKey::parse("request-1234").expect("key");
        let create = sandbox_idempotency_key_hash("create_file", &key);
        let update = sandbox_idempotency_key_hash("update_file", &key);
        assert_ne!(create, update);
        assert!(!create.contains(key.as_str()));
    }
}
