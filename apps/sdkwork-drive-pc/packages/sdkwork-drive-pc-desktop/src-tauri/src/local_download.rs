use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const MAX_CHUNK_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDownloadSaveRequest {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDownloadSaveResponse {
    pub saved: bool,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDownloadBeginRequest {
    pub file_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDownloadBeginResponse {
    pub session_id: String,
    pub saved: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDownloadWriteChunkRequest {
    pub session_id: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDownloadSessionRequest {
    pub session_id: String,
}

struct DownloadSession {
    path: PathBuf,
    file: File,
}

fn sessions() -> &'static Mutex<HashMap<String, DownloadSession>> {
    static SESSIONS: OnceLock<Mutex<HashMap<String, DownloadSession>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn sanitize_download_file_name(file_name: &str) -> String {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return "download.bin".to_string();
    }
    trimmed
        .replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_")
}

fn ensure_parent_directory(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create download directory: {error}"))?;
        }
    }
    Ok(())
}

pub fn save_download_file(
    request: LocalDownloadSaveRequest,
) -> Result<LocalDownloadSaveResponse, String> {
    let default_name = sanitize_download_file_name(&request.file_name);
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(&default_name)
        .save_file()
    else {
        return Ok(LocalDownloadSaveResponse {
            saved: false,
            path: None,
        });
    };

    ensure_parent_directory(&path)?;
    fs::write(&path, &request.bytes)
        .map_err(|error| format!("failed to write download file: {error}"))?;
    Ok(LocalDownloadSaveResponse {
        saved: true,
        path: Some(path.to_string_lossy().into_owned()),
    })
}

pub fn begin_download_save(
    request: LocalDownloadBeginRequest,
) -> Result<LocalDownloadBeginResponse, String> {
    let default_name = sanitize_download_file_name(&request.file_name);
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(&default_name)
        .save_file()
    else {
        return Ok(LocalDownloadBeginResponse {
            session_id: String::new(),
            saved: false,
        });
    };

    ensure_parent_directory(&path)?;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .map_err(|error| format!("failed to open download file: {error}"))?;
    let session_id = uuid::Uuid::new_v4().to_string();
    sessions()
        .lock()
        .map_err(|_| "download session lock poisoned".to_string())?
        .insert(
            session_id.clone(),
            DownloadSession {
                path,
                file,
            },
        );
    Ok(LocalDownloadBeginResponse {
        session_id,
        saved: true,
    })
}

pub fn write_download_chunk(request: LocalDownloadWriteChunkRequest) -> Result<(), String> {
    if request.bytes.len() > MAX_CHUNK_BYTES {
        return Err(format!(
            "download chunk exceeds {MAX_CHUNK_BYTES} byte limit"
        ));
    }
    let mut guard = sessions()
        .lock()
        .map_err(|_| "download session lock poisoned".to_string())?;
    let Some(session) = guard.get_mut(&request.session_id) else {
        return Err("download session not found".to_string());
    };
    session
        .file
        .write_all(&request.bytes)
        .map_err(|error| format!("failed to write download chunk: {error}"))?;
    Ok(())
}

pub fn finish_download_save(
    request: LocalDownloadSessionRequest,
) -> Result<LocalDownloadSaveResponse, String> {
    let session = {
        let mut guard = sessions()
            .lock()
            .map_err(|_| "download session lock poisoned".to_string())?;
        guard.remove(&request.session_id)
    };
    let Some(session) = session else {
        return Err("download session not found".to_string());
    };
    session
        .file
        .sync_all()
        .map_err(|error| format!("failed to finalize download file: {error}"))?;
    Ok(LocalDownloadSaveResponse {
        saved: true,
        path: Some(session.path.to_string_lossy().into_owned()),
    })
}

pub fn abort_download_save(request: LocalDownloadSessionRequest) -> Result<(), String> {
    let session = {
        let mut guard = sessions()
            .lock()
            .map_err(|_| "download session lock poisoned".to_string())?;
        guard.remove(&request.session_id)
    };
    if let Some(session) = session {
        let _ = fs::remove_file(session.path);
    }
    Ok(())
}
