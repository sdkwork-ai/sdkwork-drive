mod host_clipboard;
mod host_shortcut;
mod host_tray;
mod local_download;
mod local_filesystem;
mod local_upload;
mod session_secure_store;

use host_clipboard::{
    cut_paths_to_clipboard, read_clipboard_paths, write_text_to_clipboard, ClipboardCutRequest,
    ClipboardCutResult, ClipboardWriteTextRequest,
};
use host_shortcut::{
    register_shortcuts, unregister_all_shortcuts, unregister_shortcut, ShortcutActionRequest,
    ShortcutRegisterManyRequest, ShortcutRegistrationResult,
};
use host_tray::{
    apply_tray_menu, apply_tray_visibility, emit_tray_menu_event, handle_tray_event, TRAY_ID,
    TraySetMenuRequest, TraySetVisibleRequest,
};
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
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowControlRequest {
    action: WindowControlAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrayMenuEmitRequest {
    menu_id: String,
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

#[tauri::command]
fn tray_set_visible(app: AppHandle, request: TraySetVisibleRequest) -> Result<(), String> {
    apply_tray_visibility(&app, request.visible)
}

#[tauri::command]
fn tray_set_menu(app: AppHandle, request: TraySetMenuRequest) -> Result<(), String> {
    apply_tray_menu(&app, request)
}

#[tauri::command]
fn tray_emit_menu(app: AppHandle, request: TrayMenuEmitRequest) -> Result<(), String> {
    emit_tray_menu_event(&app, &request.menu_id);
    Ok(())
}

#[tauri::command]
fn tray_restore_window(app: AppHandle) -> Result<(), String> {
    host_tray::restore_main_window(&app);
    Ok(())
}

#[tauri::command]
fn shortcut_register_all(
    app: AppHandle,
    request: ShortcutRegisterManyRequest,
) -> Vec<ShortcutRegistrationResult> {
    register_shortcuts(&app, request)
}

#[tauri::command]
fn shortcut_unregister(app: AppHandle, request: ShortcutActionRequest) -> Result<(), String> {
    unregister_shortcut(&app, request)
}

#[tauri::command]
fn shortcut_unregister_all(app: AppHandle) -> Result<(), String> {
    unregister_all_shortcuts(&app)
}

#[tauri::command]
fn clipboard_cut_paths(
    app: AppHandle,
    request: ClipboardCutRequest,
) -> Result<ClipboardCutResult, String> {
    cut_paths_to_clipboard(&app, request)
}

#[tauri::command]
fn clipboard_read_paths(app: AppHandle) -> Vec<String> {
    read_clipboard_paths(&app)
}

#[tauri::command]
fn clipboard_write_text(app: AppHandle, request: ClipboardWriteTextRequest) -> Result<(), String> {
    write_text_to_clipboard(&app, request)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let _ = app.emit(
                            host_shortcut::SHORTCUT_TRIGGERED_EVENT,
                            _shortcut.to_string(),
                        );
                    }
                })
                .build(),
        )
        .setup(|app| {
            init_secure_session_state(&app.handle())?;
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                tray.on_tray_icon_event(|tray, event| {
                    handle_tray_event(&tray.app_handle().clone(), &event);
                });
            }
            Ok(())
        })
        .on_menu_event(|app, event| {
            let menu_id = event.id().as_ref().to_string();
            if menu_id.starts_with("drive.") || menu_id.starts_with("tray.") {
                emit_tray_menu_event(app, &menu_id);
            }
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
            read_secure_session_snapshot,
            tray_set_visible,
            tray_set_menu,
            tray_emit_menu,
            tray_restore_window,
            shortcut_register_all,
            shortcut_unregister,
            shortcut_unregister_all,
            clipboard_cut_paths,
            clipboard_read_paths,
            clipboard_write_text
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SDKWork Drive desktop host");
}
