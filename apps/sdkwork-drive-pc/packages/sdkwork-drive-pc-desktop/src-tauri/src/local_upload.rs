use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sdkwork_utils_rust::hex_encode;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::UNIX_EPOCH;

static ALLOWED_UPLOAD_PATHS: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalUploadFileDescriptor {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified_at: String,
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalUploadPathRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalUploadReadRangeRequest {
    pub path: String,
    pub offset_bytes: u64,
    pub length_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalUploadReadRangeResponse {
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalUploadChecksumResponse {
    pub checksum_sha256_hex: String,
}

pub fn pick_upload_files() -> Result<Vec<LocalUploadFileDescriptor>, String> {
    let Some(paths) = rfd::FileDialog::new().pick_files() else {
        return Ok(Vec::new());
    };

    paths
        .into_iter()
        .map(|path| {
            let descriptor = describe_upload_file(path)?;
            register_allowed_upload_path(&descriptor.path)?;
            Ok(descriptor)
        })
        .collect()
}

pub fn describe_local_upload_file(request: LocalUploadPathRequest) -> Result<LocalUploadFileDescriptor, String> {
    let path = ensure_allowed_upload_path(&request.path)?;
    describe_upload_file(path)
}

pub fn read_local_upload_range(
    request: LocalUploadReadRangeRequest,
) -> Result<LocalUploadReadRangeResponse, String> {
    const MAX_READ_BYTES: u64 = 8 * 1024 * 1024;
    let path = ensure_allowed_upload_path(&request.path)?;
    if request.length_bytes == 0 {
        return Ok(LocalUploadReadRangeResponse { bytes: Vec::new() });
    }
    if request.length_bytes > MAX_READ_BYTES {
        return Err(format!(
            "Local upload read range exceeds the maximum allowed size of {MAX_READ_BYTES} bytes."
        ));
    }

    let mut file = File::open(&path).map_err(|error| {
        format!(
            "Unable to open local upload file {}: {error}",
            path.to_string_lossy()
        )
    })?;
    file.seek(SeekFrom::Start(request.offset_bytes))
        .map_err(|error| format!("Unable to seek local upload file: {error}"))?;

    let mut buffer = vec![0u8; request.length_bytes as usize];
    file.read_exact(&mut buffer)
        .map_err(|error| format!("Unable to read local upload file range: {error}"))?;

    Ok(LocalUploadReadRangeResponse { bytes: buffer })
}

pub fn checksum_local_upload_file(
    request: LocalUploadPathRequest,
) -> Result<LocalUploadChecksumResponse, String> {
    let path = ensure_allowed_upload_path(&request.path)?;
    let mut file = File::open(&path).map_err(|error| {
        format!(
            "Unable to open local upload file {}: {error}",
            path.to_string_lossy()
        )
    })?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read_bytes = file
            .read(&mut buffer)
            .map_err(|error| format!("Unable to hash local upload file: {error}"))?;
        if read_bytes == 0 {
            break;
        }
        hasher.update(&buffer[..read_bytes]);
    }

    let digest = hasher.finalize();
    Ok(LocalUploadChecksumResponse {
        checksum_sha256_hex: format!("sha256:{}", hex_encode(digest.as_slice())),
    })
}

fn register_allowed_upload_path(path: &str) -> Result<(), String> {
    let canonical = normalize_existing_path(path)?;
    ALLOWED_UPLOAD_PATHS
        .lock()
        .map_err(|_| "Local upload allowlist is unavailable.".to_string())?
        .insert(canonical);
    Ok(())
}

fn ensure_allowed_upload_path(path: &str) -> Result<PathBuf, String> {
    let canonical = normalize_existing_path(path)?;
    let allowed = ALLOWED_UPLOAD_PATHS
        .lock()
        .map_err(|_| "Local upload allowlist is unavailable.".to_string())?;
    if allowed.contains(&canonical) {
        Ok(canonical)
    } else {
        Err("Local upload path was not selected through the native file picker.".to_string())
    }
}

fn describe_upload_file(path: PathBuf) -> Result<LocalUploadFileDescriptor, String> {
    let canonical = normalize_existing_path(&path.to_string_lossy())?;
    let metadata = fs::metadata(&canonical).map_err(|error| {
        format!(
            "Unable to read local upload file metadata {}: {error}",
            canonical.to_string_lossy()
        )
    })?;
    if !metadata.is_file() {
        return Err("Only regular files can be uploaded.".to_string());
    }

    let name = canonical
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Local upload file name is unavailable.".to_string())?;
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis().to_string())
        .unwrap_or_else(|| "0".to_string());

    Ok(LocalUploadFileDescriptor {
        path: canonical.to_string_lossy().into_owned(),
        name,
        size: metadata.len(),
        modified_at,
        mime_type: guess_mime_type(&canonical),
    })
}

fn normalize_existing_path(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("A local upload path is required.".to_string());
    }

    let candidate = PathBuf::from(trimmed);
    if candidate
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("Parent directory traversal is not allowed.".to_string());
    }

    fs::canonicalize(&candidate).map_err(|error| {
        format!("Unable to resolve local upload path {trimmed}: {error}")
    })
}

fn guess_mime_type(path: &Path) -> String {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
    {
        Some(ext) if ext == "txt" => "text/plain".to_string(),
        Some(ext) if ext == "pdf" => "application/pdf".to_string(),
        Some(ext) if ext == "png" => "image/png".to_string(),
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg".to_string(),
        Some(ext) if ext == "gif" => "image/gif".to_string(),
        Some(ext) if ext == "webp" => "image/webp".to_string(),
        Some(ext) if ext == "zip" => "application/zip".to_string(),
        Some(ext) if ext == "json" => "application/json".to_string(),
        Some(ext) if ext == "md" => "text/markdown".to_string(),
        Some(ext) if ext == "html" || ext == "htm" => "text/html".to_string(),
        Some(ext) if ext == "mp4" => "video/mp4".to_string(),
        Some(ext) if ext == "mp3" => "audio/mpeg".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}
