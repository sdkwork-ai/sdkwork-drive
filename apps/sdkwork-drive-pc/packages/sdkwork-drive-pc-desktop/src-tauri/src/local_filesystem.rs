use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalFilesystemEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified_at: Option<String>,
    pub mime_type: Option<String>,
    pub entry_kind: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalFilesystemListRequest {
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalFilesystemOpenRequest {
    pub path: String,
}

pub fn list_local_filesystem(request: LocalFilesystemListRequest) -> Result<Vec<LocalFilesystemEntry>, String> {
    match request.path {
        Some(path) if !path.trim().is_empty() => list_directory(&path),
        _ => Ok(list_roots()),
    }
}

pub fn open_local_filesystem_path(request: LocalFilesystemOpenRequest) -> Result<(), String> {
    let path = normalize_existing_path(&request.path)?;
    open::that(path).map_err(|error| format!("Failed to open local path: {error}"))
}

fn list_roots() -> Vec<LocalFilesystemEntry> {
    let mut entries = Vec::new();

    #[cfg(windows)]
    {
        for drive in b'A'..=b'Z' {
            let path = format!("{}:\\", drive as char);
            if Path::new(&path).exists() {
                entries.push(LocalFilesystemEntry {
                    name: format!("{}:", drive as char),
                    path: path.clone(),
                    is_directory: true,
                    size: None,
                    modified_at: None,
                    mime_type: None,
                    entry_kind: "drive".to_string(),
                });
            }
        }
    }

    #[cfg(not(windows))]
    {
        entries.push(LocalFilesystemEntry {
            name: "/".to_string(),
            path: "/".to_string(),
            is_directory: true,
            size: None,
            modified_at: None,
            mime_type: None,
            entry_kind: "root".to_string(),
        });
    }

    if let Some(home) = resolve_home_dir() {
        append_special_folder(&mut entries, &home, "Desktop", "desktop");
        append_special_folder(&mut entries, &home, "Documents", "documents");
        append_special_folder(&mut entries, &home, "Downloads", "downloads");
        entries.push(directory_entry(
            home.to_string_lossy().into_owned(),
            home_display_name(),
            "home",
        ));
    }

    entries.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    entries
}

fn list_directory(path: &str) -> Result<Vec<LocalFilesystemEntry>, String> {
    let directory = normalize_existing_path(path)?;
    if !directory.is_dir() {
        return Err("The selected path is not a directory.".to_string());
    }

    let mut entries = Vec::new();
    let read_dir = fs::read_dir(&directory).map_err(|error| {
        format!(
            "Unable to read directory {}: {error}",
            directory.to_string_lossy()
        )
    })?;

    for item in read_dir {
        let item = match item {
            Ok(value) => value,
            Err(_) => continue,
        };
        let metadata = match item.metadata() {
            Ok(value) => value,
            Err(_) => continue,
        };
        let file_name = item.file_name().to_string_lossy().into_owned();
        if file_name.starts_with('.') {
            continue;
        }
        let entry_path = item.path();
        let is_directory = metadata.is_dir();
        entries.push(LocalFilesystemEntry {
            name: file_name,
            path: entry_path.to_string_lossy().into_owned(),
            is_directory,
            size: if is_directory { None } else { Some(metadata.len()) },
            modified_at: metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| value.as_millis().to_string()),
            mime_type: if is_directory {
                None
            } else {
                Some(guess_mime_type(&entry_path))
            },
            entry_kind: if is_directory {
                "folder".to_string()
            } else {
                "file".to_string()
            },
        });
    }

    entries.sort_by(|left, right| {
        match (left.is_directory, right.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        }
    });

    Ok(entries)
}

fn append_special_folder(
    entries: &mut Vec<LocalFilesystemEntry>,
    home: &Path,
    folder_name: &str,
    entry_kind: &str,
) {
    let folder_path = home.join(folder_name);
    if folder_path.is_dir() {
        entries.push(directory_entry(
            folder_path.to_string_lossy().into_owned(),
            folder_name.to_string(),
            entry_kind,
        ));
    }
}

fn directory_entry(path: String, name: String, entry_kind: &str) -> LocalFilesystemEntry {
    LocalFilesystemEntry {
        name,
        path,
        is_directory: true,
        size: None,
        modified_at: None,
        mime_type: None,
        entry_kind: entry_kind.to_string(),
    }
}

fn resolve_home_dir() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("HOME") {
        let path = PathBuf::from(value);
        if path.is_dir() {
            return Some(path);
        }
    }

      if let Ok(value) = std::env::var("USERPROFILE") {
        let path = PathBuf::from(value);
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

fn home_display_name() -> String {
    #[cfg(windows)]
    {
        "Home".to_string()
    }
    #[cfg(not(windows))]
    {
        "Home".to_string()
    }
}

fn normalize_existing_path(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("A local filesystem path is required.".to_string());
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.components().any(|component| matches!(component, Component::ParentDir)) {
        return Err("Parent directory traversal is not allowed.".to_string());
    }

    let canonical = fs::canonicalize(&candidate).map_err(|error| {
        format!("Unable to resolve local path {trimmed}: {error}")
    })?;

    Ok(canonical)
}

fn guess_mime_type(path: &Path) -> String {
    match path.extension().and_then(|value| value.to_str()).map(|value| value.to_ascii_lowercase()) {
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
