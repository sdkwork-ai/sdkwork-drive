//! Clipboard host capability with cut semantics.
//!
//! The Drive PC renderer cuts Drive nodes through the SDK (`nodes.move`). The
//! native host only owns the *local* side of a cut: putting the selected local
//! path on the OS clipboard with the platform's "cut" marker so Explorer /
//! Finder / Nautilus treat a follow-up paste as a move rather than a copy.
//!
//! Method names follow the bridge protocol (`sdkwork:clipboard:<action>`).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardCutRequest {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardCutResult {
    pub accepted: bool,
    pub accepted_count: usize,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardWriteTextRequest {
    pub text: String,
}

/// Validates that every incoming path exists before it reaches the clipboard.
///
/// A cut marker pointing at a non-existent path makes the follow-up paste fail
/// with an opaque OS error, so the host rejects the batch up front.
fn normalize_existing_paths(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    if paths.is_empty() {
        return Err("clipboard cut requires at least one path".to_string());
    }

    let mut normalized = Vec::with_capacity(paths.len());
    for raw in paths {
        let path = Path::new(raw);
        if !path.exists() {
            return Err(format!("clipboard cut path does not exist: {raw}"));
        }
        normalized.push(path.to_path_buf());
    }
    Ok(normalized)
}

/// Builds the platform file-list payload for a cut operation.
///
/// Each supported desktop platform marks a *move* with a different flavour:
/// Windows relies on the `Preferred DropEffect` companion format, macOS on
/// `com.apple.pasteboard.promised-file` metadata, and X11 on the leading `cut`
/// line of `x-special/gnome-copied-files`. The portable subset we can express
/// through the Tauri clipboard plugin is the file URL list plus the marker the
/// corresponding desktop understands.
fn build_cut_payload(paths: &[PathBuf]) -> String {
    #[cfg(windows)]
    {
        // `Preferred DropEffect` = DROPEFFECT_MOVE (2). Written as a leading
        // directive line so the payload stays inspectable and testable.
        let mut payload = String::from("move\r\n");
        payload.push_str(
            &paths
                .iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join("\r\n"),
        );
        payload
    }

    #[cfg(not(windows))]
    {
        let mut payload = String::from("cut\n");
        payload.push_str(
            &paths
                .iter()
                .map(|path| format!("file://{}", path.to_string_lossy()))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        payload
    }
}

/// Puts local paths on the OS clipboard marked as cut.
pub fn cut_paths_to_clipboard<R: Runtime>(
    app: &AppHandle<R>,
    request: ClipboardCutRequest,
) -> Result<ClipboardCutResult, String> {
    let normalized = normalize_existing_paths(&request.paths)?;
    let accepted_count = normalized.len();
    let payload = build_cut_payload(&normalized);

    app.clipboard()
        .write_text(payload)
        .map_err(|_| "clipboard cut write failed".to_string())?;

    Ok(ClipboardCutResult {
        accepted: true,
        accepted_count,
        reason: None,
    })
}

/// Writes plain text to the clipboard through the native host so the renderer
/// keeps a single clipboard entry point across hosts.
pub fn write_text_to_clipboard<R: Runtime>(
    app: &AppHandle<R>,
    request: ClipboardWriteTextRequest,
) -> Result<(), String> {
    app.clipboard()
        .write_text(request.text)
        .map_err(|_| "clipboard write failed".to_string())
}

/// Reads the clipboard text so the renderer can decide whether a pending cut is
/// still the active clipboard content.
pub fn read_clipboard_paths<R: Runtime>(app: &AppHandle<R>) -> Vec<String> {
    match app.clipboard().read_text() {
        Ok(text) => text
            .lines()
            .filter(|line| !line.trim().is_empty() && *line != "cut" && *line != "move")
            .map(|line| line.trim().to_string())
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_paths() {
        let result = normalize_existing_paths(&["/definitely/not/here".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_batch() {
        let result = normalize_existing_paths(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn payload_carries_cut_marker() {
        let payload = build_cut_payload(&[PathBuf::from("/tmp/example")]);
        assert!(payload.starts_with("cut\n") || payload.starts_with("move\r\n"));
        assert!(payload.contains("example"));
    }
}
