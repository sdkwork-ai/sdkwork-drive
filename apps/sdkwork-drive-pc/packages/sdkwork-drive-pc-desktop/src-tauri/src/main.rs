mod local_download;
mod local_filesystem;
mod local_upload;
mod session_secure_store;

use local_download::{
    abort_download_save, begin_download_save, finish_download_save, save_download_file,
    write_download_chunk, LocalDownloadBeginRequest, LocalDownloadBeginResponse,
    LocalDownloadSaveRequest, LocalDownloadSaveResponse, LocalDownloadSessionRequest,
    LocalDownloadWriteChunkRequest,
};
use local_filesystem::{
    list_local_filesystem, open_local_filesystem_path, LocalFilesystemListRequest,
    LocalFilesystemOpenRequest,
};

use local_upload::{
    checksum_local_upload_file, describe_local_upload_file, pick_upload_files,
    read_local_upload_range, LocalUploadChecksumResponse, LocalUploadFileDescriptor,
    LocalUploadPathRequest, LocalUploadReadRangeRequest, LocalUploadReadRangeResponse,
};
use session_secure_store::{
    clear_secure_session_values, init_secure_session_state, read_secure_session_snapshot,
    remove_secure_session_value, write_secure_session_value,
};
use serde::Deserialize;
use tauri::{AppHandle, Manager};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowControlRequest {
    action: WindowControlAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum WindowControlAction {
    Minimize,
    Maximize,
    Unmaximize,
    Close,
    Show,
}

#[tauri::command]
fn window_control(app: AppHandle, request: WindowControlRequest) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window is unavailable".to_string())?;

    match request.action {
        WindowControlAction::Minimize => window.minimize(),
        WindowControlAction::Maximize => window.maximize(),
        WindowControlAction::Unmaximize => window.unmaximize(),
        WindowControlAction::Close => window.close(),
        WindowControlAction::Show => window.show(),
    }
    .map_err(|_| "window control failed".to_string())
}

#[tauri::command]
fn local_filesystem_list(
    request: LocalFilesystemListRequest,
) -> Result<Vec<local_filesystem::LocalFilesystemEntry>, String> {
    list_local_filesystem(request)
}

#[tauri::command]
fn local_filesystem_open(request: LocalFilesystemOpenRequest) -> Result<(), String> {
    open_local_filesystem_path(request)
}

#[tauri::command]
fn local_upload_pick_files() -> Result<Vec<LocalUploadFileDescriptor>, String> {
    pick_upload_files()
}

#[tauri::command]
fn local_upload_describe_file(
    request: LocalUploadPathRequest,
) -> Result<LocalUploadFileDescriptor, String> {
    describe_local_upload_file(request)
}

#[tauri::command]
fn local_upload_read_range(
    request: LocalUploadReadRangeRequest,
) -> Result<LocalUploadReadRangeResponse, String> {
    read_local_upload_range(request)
}

#[tauri::command]
fn local_upload_checksum_file(
    request: LocalUploadPathRequest,
) -> Result<LocalUploadChecksumResponse, String> {
    checksum_local_upload_file(request)
}

#[tauri::command]
fn local_download_save(
    request: LocalDownloadSaveRequest,
) -> Result<LocalDownloadSaveResponse, String> {
    save_download_file(request)
}

#[tauri::command]
fn local_download_begin(
    request: LocalDownloadBeginRequest,
) -> Result<LocalDownloadBeginResponse, String> {
    begin_download_save(request)
}

#[tauri::command]
fn local_download_write_chunk(request: LocalDownloadWriteChunkRequest) -> Result<(), String> {
    write_download_chunk(request)
}

#[tauri::command]
fn local_download_finish(
    request: LocalDownloadSessionRequest,
) -> Result<LocalDownloadSaveResponse, String> {
    finish_download_save(request)
}

#[tauri::command]
fn local_download_abort(request: LocalDownloadSessionRequest) -> Result<(), String> {
    abort_download_save(request)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            init_secure_session_state(&app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            window_control,
            local_filesystem_list,
            local_filesystem_open,
            local_upload_pick_files,
            local_upload_describe_file,
            local_upload_read_range,
            local_upload_checksum_file,
            local_download_save,
            local_download_begin,
            local_download_write_chunk,
            local_download_finish,
            local_download_abort,
            write_secure_session_value,
            remove_secure_session_value,
            clear_secure_session_values,
            read_secure_session_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SDKWork Drive desktop host");
}
