use crate::domain::resource_resolution::{DriveResourceScopeKind, ResolvedDriveResource};
use crate::infrastructure::sql::resource_resolution_store::SqlResourceResolutionStore;
use crate::ports::resource_resolution_store::{DriveResourceResolutionStore, ResolveDriveResource};
use crate::DriveServiceError;

const MAX_RELATIVE_PATH_BYTES: usize = 4096;
const MAX_PATH_SEGMENTS: usize = 128;
const MAX_PATH_SEGMENT_BYTES: usize = 255;
const RESERVED_PATH_SEGMENTS: [&str; 7] = [
    ".sdkwork",
    ".trash",
    "trash",
    ".staging",
    "staging",
    ".versions",
    "versions",
];

#[derive(Debug, Clone)]
pub struct ResolveDriveResourceCommand {
    pub tenant_id: String,
    pub scope_kind: DriveResourceScopeKind,
    pub scope_uuid: String,
    pub relative_path: String,
    pub pinned_generation: Option<i64>,
    pub pinned_node_version_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DriveResourceResolutionService<S>
where
    S: DriveResourceResolutionStore,
{
    store: S,
}

impl<S> DriveResourceResolutionService<S>
where
    S: DriveResourceResolutionStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn resolve(
        &self,
        command: ResolveDriveResourceCommand,
    ) -> Result<ResolvedDriveResource, DriveServiceError> {
        let tenant_id = require_text(command.tenant_id, "tenant_id", 64)?;
        let scope_uuid = require_text(command.scope_uuid, "scope_uuid", 64)?;
        let relative_path = validate_relative_path(command.relative_path)?;
        if command.pinned_generation.is_some_and(|value| value < 1) {
            return Err(DriveServiceError::Validation(
                "pinned_generation must be greater than zero".to_string(),
            ));
        }
        let pinned_node_version_id = command
            .pinned_node_version_id
            .map(|value| require_text(value, "pinned_node_version_id", 64))
            .transpose()?;

        self.store
            .resolve(&ResolveDriveResource {
                tenant_id,
                scope_kind: command.scope_kind,
                scope_uuid,
                relative_path,
                pinned_generation: command.pinned_generation,
                pinned_node_version_id,
            })
            .await
    }
}

pub type SqlDriveResourceResolutionService =
    DriveResourceResolutionService<SqlResourceResolutionStore>;

fn require_text(
    value: String,
    field_name: &str,
    max_length: usize,
) -> Result<String, DriveServiceError> {
    let value = value.trim().to_string();
    if value.is_empty() || value.len() > max_length {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} must contain between 1 and {max_length} characters"
        )));
    }
    Ok(value)
}

fn validate_relative_path(value: String) -> Result<String, DriveServiceError> {
    if value.is_empty() || value.len() > MAX_RELATIVE_PATH_BYTES {
        return Err(DriveServiceError::Validation(format!(
            "relative_path must contain between 1 and {MAX_RELATIVE_PATH_BYTES} UTF-8 bytes"
        )));
    }
    if value.starts_with('/') || value.ends_with('/') || value.contains('\\') {
        return Err(DriveServiceError::Validation(
            "relative_path must be canonical, relative, and slash-separated".to_string(),
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(DriveServiceError::Validation(
            "relative_path must not contain control characters".to_string(),
        ));
    }

    let segments = value.split('/').collect::<Vec<_>>();
    if segments.is_empty() || segments.len() > MAX_PATH_SEGMENTS {
        return Err(DriveServiceError::Validation(format!(
            "relative_path must contain at most {MAX_PATH_SEGMENTS} segments"
        )));
    }
    for segment in segments {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || segment.len() > MAX_PATH_SEGMENT_BYTES
        {
            return Err(DriveServiceError::Validation(
                "relative_path contains an invalid segment".to_string(),
            ));
        }
        if RESERVED_PATH_SEGMENTS
            .iter()
            .any(|reserved| segment.eq_ignore_ascii_case(reserved))
        {
            return Err(DriveServiceError::PermissionDenied(
                "relative_path targets a reserved Drive namespace".to_string(),
            ));
        }
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_validation_is_fail_closed() {
        for invalid in [
            "",
            "/index.html",
            "index.html/",
            "docs//index.md",
            "docs/../index.md",
            "docs\\index.md",
            ".sdkwork/manifest.json",
            "docs/.VERSIONS/1.md",
        ] {
            assert!(
                validate_relative_path(invalid.to_string()).is_err(),
                "{invalid}"
            );
        }
        assert_eq!(
            validate_relative_path("docs/getting started.md".to_string()).expect("valid path"),
            "docs/getting started.md"
        );
    }
}
