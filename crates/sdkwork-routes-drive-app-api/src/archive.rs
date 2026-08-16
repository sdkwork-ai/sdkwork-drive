use crate::constants::{
    ARCHIVE_EXTRACT_MAX_FILE_BYTES, ARCHIVE_EXTRACT_MAX_TOTAL_UNCOMPRESSED_BYTES,
    ARCHIVE_MAX_ENTRIES, ARCHIVE_MAX_TOTAL_UNCOMPRESSED_BYTES,
};
use crate::dto::{ArchiveEntryResponse, DriveNodeResponse, SafeArchivePath};
use crate::error::{internal_problem, problem, ProblemDetail, SdkWorkResultCode};
use crate::hashing::sha256_hex;
use axum::http::StatusCode;
use axum::Json;
use std::collections::BTreeSet;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use zip::ZipArchive;

type ArchiveZipReader<'a> = ZipArchive<Cursor<&'a [u8]>>;

#[derive(Debug, Clone)]
pub(crate) struct ArchiveFileForExtract {
    pub(crate) path: SafeArchivePath,
    pub(crate) content_type: String,
    pub(crate) content: Vec<u8>,
    pub(crate) checksum_sha256_hex: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ArchiveFileForExtractPlan {
    pub(crate) archive_index: usize,
    pub(crate) path: SafeArchivePath,
    pub(crate) content_type: String,
}

pub(crate) fn validate_archive_source_node(
    node: &DriveNodeResponse,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if node.node_type == "file" {
        return Ok(());
    }
    Err(problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "archiveEntries can only be used with file nodes",
        SdkWorkResultCode::ValidationError,
    ))
}

pub(crate) fn is_supported_archive_content(node_name: &str, content_type: &str) -> bool {
    let lower_name = node_name.to_ascii_lowercase();
    let lower_content_type = content_type.to_ascii_lowercase();
    lower_name.ends_with(".zip")
        || lower_content_type == "application/zip"
        || lower_content_type == "application/x-zip-compressed"
}

pub(crate) fn read_archive_entry_list(
    archive_bytes: &[u8],
) -> Result<Vec<ArchiveEntryResponse>, (StatusCode, Json<ProblemDetail>)> {
    let mut archive = open_zip_archive(archive_bytes)?;
    validate_archive_entry_count(archive.len())?;
    let mut items = Vec::with_capacity(archive.len());
    let mut total_uncompressed = 0_i64;
    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(map_zip_archive_error)?;
        let is_directory = file.is_dir();
        let safe_path = safe_archive_path_from_enclosed(file.enclosed_name(), is_directory)?;
        let uncompressed_size = archive_size_to_i64(file.size())?;
        let compressed_size = archive_size_to_i64(file.compressed_size())?;
        total_uncompressed = total_uncompressed
            .checked_add(uncompressed_size)
            .ok_or_else(|| archive_limit_problem("archive total uncompressed size overflow"))?;
        if total_uncompressed > ARCHIVE_MAX_TOTAL_UNCOMPRESSED_BYTES {
            return Err(archive_limit_problem(&format!(
                "archive total uncompressed size must be at most {ARCHIVE_MAX_TOTAL_UNCOMPRESSED_BYTES}"
            )));
        }
        items.push(ArchiveEntryResponse {
            name: safe_path
                .segments
                .last()
                .cloned()
                .unwrap_or_else(|| safe_path.path.clone()),
            path: safe_path.path.clone(),
            is_directory,
            uncompressed_size_bytes: uncompressed_size,
            compressed_size_bytes: compressed_size,
            content_type: (!is_directory)
                .then(|| guess_content_type_from_name(&safe_path.path).to_string()),
        });
    }
    Ok(items)
}

pub(crate) fn read_archive_file_extract_plan(
    archive_bytes: &[u8],
    requested_paths: Option<&BTreeSet<String>>,
) -> Result<Vec<ArchiveFileForExtractPlan>, (StatusCode, Json<ProblemDetail>)> {
    let mut archive = open_zip_archive(archive_bytes)?;
    validate_archive_entry_count(archive.len())?;
    let mut plans = Vec::new();
    let mut selected_uncompressed = 0_i64;
    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(map_zip_archive_error)?;
        let is_directory = file.is_dir();
        let safe_path = safe_archive_path_from_enclosed(file.enclosed_name(), is_directory)?;
        let uncompressed_size = archive_size_to_i64(file.size())?;
        if is_directory {
            continue;
        }
        if requested_paths.is_some_and(|paths| !paths.contains(&safe_path.path)) {
            continue;
        }
        validate_archive_extract_file_size(uncompressed_size)?;
        selected_uncompressed =
            checked_archive_extract_total(selected_uncompressed, uncompressed_size)?;
        plans.push(ArchiveFileForExtractPlan {
            archive_index: index,
            content_type: guess_content_type_from_name(&safe_path.path).to_string(),
            path: safe_path,
        });
    }
    Ok(plans)
}

pub(crate) fn read_archive_file_for_extract_plan(
    archive_bytes: &[u8],
    plan: &ArchiveFileForExtractPlan,
) -> Result<ArchiveFileForExtract, (StatusCode, Json<ProblemDetail>)> {
    let mut archive = open_zip_archive(archive_bytes)?;
    let mut file = archive
        .by_index(plan.archive_index)
        .map_err(map_zip_archive_error)?;
    if file.is_dir() {
        return Err(unsafe_archive_path_problem());
    }
    let safe_path = safe_archive_path_from_enclosed(file.enclosed_name(), false)?;
    if safe_path.path != plan.path.path {
        return Err(unsafe_archive_path_problem());
    }
    let uncompressed_size = archive_size_to_i64(file.size())?;
    validate_archive_extract_file_size(uncompressed_size)?;
    let content = read_archive_file_content_bounded(&mut file, uncompressed_size)?;
    Ok(ArchiveFileForExtract {
        content_type: plan.content_type.clone(),
        checksum_sha256_hex: sha256_hex(&content),
        path: safe_path,
        content,
    })
}

pub(crate) fn validate_archive_file_extract_actual_total(
    archive_bytes: &[u8],
    plans: &[ArchiveFileForExtractPlan],
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    let mut actual_uncompressed = 0_i64;
    for plan in plans {
        let actual_file_size = read_archive_file_actual_size_for_extract_plan(archive_bytes, plan)?;
        actual_uncompressed = checked_archive_extract_total(actual_uncompressed, actual_file_size)?;
    }
    Ok(())
}

fn open_zip_archive(
    archive_bytes: &[u8],
) -> Result<ArchiveZipReader<'_>, (StatusCode, Json<ProblemDetail>)> {
    ZipArchive::new(Cursor::new(archive_bytes)).map_err(map_zip_archive_error)
}

fn map_zip_archive_error(error: zip::result::ZipError) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        format!("ZIP archive cannot be inspected: {error}"),
        SdkWorkResultCode::ValidationError,
    )
}

fn validate_archive_entry_count(count: usize) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if count > ARCHIVE_MAX_ENTRIES {
        return Err(archive_limit_problem(&format!(
            "archive can include at most {ARCHIVE_MAX_ENTRIES} entries"
        )));
    }
    Ok(())
}

fn archive_limit_problem(detail: &str) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        detail,
        SdkWorkResultCode::ValidationError,
    )
}

fn archive_payload_too_large_problem(detail: &str) -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::PAYLOAD_TOO_LARGE,
        "validation failed",
        detail,
        SdkWorkResultCode::ValidationError,
    )
}

fn validate_archive_extract_file_size(
    uncompressed_size: i64,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if uncompressed_size > ARCHIVE_EXTRACT_MAX_FILE_BYTES {
        return Err(archive_payload_too_large_problem(&format!(
            "archive extraction file size must be at most {ARCHIVE_EXTRACT_MAX_FILE_BYTES} bytes"
        )));
    }
    Ok(())
}

fn checked_archive_extract_total(
    current_total: i64,
    next_size: i64,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    let total = current_total.checked_add(next_size).ok_or_else(|| {
        archive_payload_too_large_problem("archive extraction total uncompressed size overflow")
    })?;
    if total > ARCHIVE_EXTRACT_MAX_TOTAL_UNCOMPRESSED_BYTES {
        return Err(archive_payload_too_large_problem(&format!(
            "archive extraction total uncompressed size must be at most {ARCHIVE_EXTRACT_MAX_TOTAL_UNCOMPRESSED_BYTES} bytes"
        )));
    }
    Ok(total)
}

fn read_archive_file_content_bounded<R: Read>(
    reader: &mut R,
    expected_uncompressed_size: i64,
) -> Result<Vec<u8>, (StatusCode, Json<ProblemDetail>)> {
    let mut content = Vec::with_capacity(expected_uncompressed_size as usize);
    let max_read = ARCHIVE_EXTRACT_MAX_FILE_BYTES as u64 + 1;
    let mut limited_reader = reader.take(max_read);
    limited_reader
        .read_to_end(&mut content)
        .map_err(|error| internal_problem(format!("read zip archive entry failed: {error}")))?;
    if content.len() as i64 > ARCHIVE_EXTRACT_MAX_FILE_BYTES {
        return Err(archive_payload_too_large_problem(&format!(
            "archive extraction file size must be at most {ARCHIVE_EXTRACT_MAX_FILE_BYTES} bytes"
        )));
    }
    Ok(content)
}

fn read_archive_file_actual_size_for_extract_plan(
    archive_bytes: &[u8],
    plan: &ArchiveFileForExtractPlan,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    let mut archive = open_zip_archive(archive_bytes)?;
    let mut file = archive
        .by_index(plan.archive_index)
        .map_err(map_zip_archive_error)?;
    if file.is_dir() {
        return Err(unsafe_archive_path_problem());
    }
    let safe_path = safe_archive_path_from_enclosed(file.enclosed_name(), false)?;
    if safe_path.path != plan.path.path {
        return Err(unsafe_archive_path_problem());
    }
    let metadata_uncompressed_size = archive_size_to_i64(file.size())?;
    validate_archive_extract_file_size(metadata_uncompressed_size)?;
    read_archive_file_actual_size_bounded(&mut file)
}

fn read_archive_file_actual_size_bounded<R: Read>(
    reader: &mut R,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    let max_read = ARCHIVE_EXTRACT_MAX_FILE_BYTES as u64 + 1;
    let mut limited_reader = reader.take(max_read);
    let actual_size = std::io::copy(&mut limited_reader, &mut std::io::sink())
        .map_err(|error| internal_problem(format!("read zip archive entry failed: {error}")))?;
    if actual_size > ARCHIVE_EXTRACT_MAX_FILE_BYTES as u64 {
        return Err(archive_payload_too_large_problem(&format!(
            "archive extraction file size must be at most {ARCHIVE_EXTRACT_MAX_FILE_BYTES} bytes"
        )));
    }
    archive_size_to_i64(actual_size)
}

fn archive_size_to_i64(value: u64) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    if value > i64::MAX as u64 {
        return Err(archive_limit_problem(
            "archive entry size exceeds supported range",
        ));
    }
    Ok(value as i64)
}

fn safe_archive_path_from_enclosed(
    enclosed_name: Option<PathBuf>,
    is_directory: bool,
) -> Result<SafeArchivePath, (StatusCode, Json<ProblemDetail>)> {
    let Some(path) = enclosed_name else {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "ZIP archive contains an unsafe entry path",
            SdkWorkResultCode::ValidationError,
        ));
    };
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(value) => {
                let raw = value.to_string_lossy();
                if raw.trim().is_empty() || raw == "." || raw == ".." {
                    return Err(unsafe_archive_path_problem());
                }
                segments.push(sanitize_archive_path_segment(&raw));
            }
            _ => return Err(unsafe_archive_path_problem()),
        }
    }
    if segments.is_empty() {
        return Err(unsafe_archive_path_problem());
    }
    let mut path = segments.join("/");
    if is_directory && !path.ends_with('/') {
        path.push('/');
    }
    Ok(SafeArchivePath { path, segments })
}

pub(crate) fn unsafe_archive_path_problem() -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::BAD_REQUEST,
        "validation failed",
        "ZIP archive contains an unsafe entry path",
        SdkWorkResultCode::ValidationError,
    )
}

pub(crate) fn normalize_archive_entry_selection(
    entry_paths: Option<Vec<String>>,
) -> Result<Option<BTreeSet<String>>, (StatusCode, Json<ProblemDetail>)> {
    let Some(entry_paths) = entry_paths else {
        return Ok(None);
    };
    if entry_paths.is_empty() {
        return Ok(None);
    }
    let mut normalized = BTreeSet::new();
    for entry_path in entry_paths {
        let path = normalize_archive_entry_selection_path(&entry_path)?;
        normalized.insert(path);
    }
    Ok(Some(normalized))
}

fn normalize_archive_entry_selection_path(
    entry_path: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = entry_path.trim().replace('\\', "/");
    if trimmed.is_empty() || trimmed.starts_with('/') || trimmed.contains(':') {
        return Err(unsafe_archive_path_problem());
    }
    let mut segments = Vec::new();
    for segment in trimmed.split('/') {
        if segment.trim().is_empty() || segment == "." || segment == ".." {
            return Err(unsafe_archive_path_problem());
        }
        segments.push(sanitize_archive_path_segment(segment));
    }
    Ok(segments.join("/"))
}

fn guess_content_type_from_name(name: &str) -> &'static str {
    let extension = name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "txt" => "text/plain",
        "md" => "text/markdown",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "json" => "application/json",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "zip" => "application/zip",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}

pub(crate) fn sanitize_archive_path_segment(raw: &str) -> String {
    let mut sanitized = raw
        .trim()
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        })
        .collect::<String>();
    while sanitized.starts_with('.') {
        sanitized.remove(0);
    }
    if sanitized.is_empty() {
        "unnamed".to_string()
    } else {
        sanitized
    }
}

pub(crate) fn sanitize_relative_archive_path(raw: &str) -> String {
    raw.split('/')
        .map(sanitize_archive_path_segment)
        .collect::<Vec<_>>()
        .join("/")
}

pub(crate) fn unique_archive_path(candidate: &str, used_paths: &mut BTreeSet<String>) -> String {
    let normalized = sanitize_relative_archive_path(candidate);
    if used_paths.insert(normalized.clone()) {
        return normalized;
    }
    let (stem, extension) = match normalized.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() => (stem.to_string(), format!(".{extension}")),
        _ => (normalized.clone(), String::new()),
    };
    for index in 1..10_000 {
        let candidate = sdkwork_utils_rust::format_numbered_filename_variant(
            &stem,
            index,
            extension.strip_prefix('.').filter(|ext| !ext.is_empty()),
        );
        if used_paths.insert(candidate.clone()) {
            return candidate;
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    #[test]
    fn read_archive_file_extract_plan_rejects_entries_above_sync_file_budget() {
        let archive_bytes = build_zip_with_repeated_file("large.bin", (16 * 1024 * 1024) + 1);

        let result = read_archive_file_extract_plan(&archive_bytes, None);
        let (status, Json(problem)) = match result {
            Ok(_) => panic!("oversized synchronous archive entry must be rejected"),
            Err(error) => error,
        };

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        let problem_json = serde_json::to_value(problem).expect("problem detail should serialize");
        let detail = problem_json["detail"]
            .as_str()
            .expect("problem detail should be present");
        assert!(
            detail.contains("archive extraction file size must be at most 16777216 bytes"),
            "unexpected problem detail: {detail}"
        );
    }

    #[test]
    fn validate_archive_file_extract_actual_total_rejects_selected_bytes_above_sync_budget() {
        let archive_bytes = build_zip_with_repeated_files(&[
            ("part-1.bin", 16 * 1024 * 1024),
            ("part-2.bin", 16 * 1024 * 1024),
            ("part-3.bin", 16 * 1024 * 1024),
            ("part-4.bin", 16 * 1024 * 1024),
            ("part-5.bin", 1),
        ]);
        let plans = [
            "part-1.bin",
            "part-2.bin",
            "part-3.bin",
            "part-4.bin",
            "part-5.bin",
        ]
        .iter()
        .enumerate()
        .map(|(archive_index, path)| ArchiveFileForExtractPlan {
            archive_index,
            content_type: "application/octet-stream".to_string(),
            path: SafeArchivePath {
                path: (*path).to_string(),
                segments: vec![(*path).to_string()],
            },
        })
        .collect::<Vec<_>>();

        let result = validate_archive_file_extract_actual_total(&archive_bytes, &plans);
        let (status, Json(problem)) = match result {
            Ok(_) => panic!("oversized synchronous archive extraction total must be rejected"),
            Err(error) => error,
        };

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        let problem_json = serde_json::to_value(problem).expect("problem detail should serialize");
        let detail = problem_json["detail"]
            .as_str()
            .expect("problem detail should be present");
        assert!(
            detail.contains(
                "archive extraction total uncompressed size must be at most 67108864 bytes"
            ),
            "unexpected problem detail: {detail}"
        );
    }

    fn build_zip_with_repeated_file(path: &str, size_bytes: usize) -> Vec<u8> {
        build_zip_with_repeated_files(&[(path, size_bytes)])
    }

    fn build_zip_with_repeated_files(files: &[(&str, usize)]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);
        let chunk = [b'a'; 8192];
        for (path, size_bytes) in files {
            writer
                .start_file(*path, options)
                .expect("zip file should start");
            let mut remaining = *size_bytes;
            while remaining > 0 {
                let write_len = remaining.min(chunk.len());
                writer
                    .write_all(&chunk[..write_len])
                    .expect("zip file content should be written");
                remaining -= write_len;
            }
        }
        writer
            .finish()
            .expect("zip file should finish")
            .into_inner()
    }
}
