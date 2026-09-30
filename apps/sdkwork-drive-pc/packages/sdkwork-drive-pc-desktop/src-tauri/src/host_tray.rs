//! System tray host capability.
//!
//! The tray belongs to the native host layer (`window | tray | ...` in
//! `DESKTOP_APP_ARCHITECTURE_SPEC.md` section 5.5) and is reached from the
//! renderer only through the bridge protocol method `sdkwork:tray:<action>`.
//!
//! Commands intentionally stay capability-shaped (`tray_set_visible`,
//! `tray_set_menu`, ...) and never carry product semantics.

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// Tray icon id owned by this host. One tray per desktop host instance.
pub const TRAY_ID: &str = "sdkwork-drive-tray";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayMenuItemPayload {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraySetMenuRequest {
    pub items: Vec<TrayMenuItemPayload>,
    #[serde(default)]
    pub tooltip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraySetVisibleRequest {
    pub visible: bool,
}

impl TrayMenuItemPayload {
    fn is_separator(&self) -> bool {
        self.id == "separator" || self.id == "-"
    }
}

/// Builds the tray menu from renderer-supplied items.
///
/// The menu is data-only: the host renders labels and forwards activation back
/// to the renderer through `sdkwork:tray:menu` events, so no product decision
/// is taken inside the native process.
pub fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    items: &[TrayMenuItemPayload],
) -> Result<Menu<R>, String> {
    let mut menu_items: Vec<Box<dyn tauri::menu::IsMenuItem<R>>> = Vec::new();

    for item in items {
        if item.is_separator() {
            menu_items.push(Box::new(
                PredefinedMenuItem::separator(app).map_err(|_| "tray menu separator failed".to_string())?,
            ));
            continue;
        }

        let entry = MenuItem::with_id(app, item.id.clone(), item.label.clone(), item.enabled, None::<&str>)
            .map_err(|_| format!("tray menu item failed: {}", item.id))?;
        menu_items.push(Box::new(entry));
    }

    let refs: Vec<&dyn tauri::menu::IsMenuItem<R>> = menu_items.iter().map(|item| item.as_ref()).collect();
    Menu::with_items(app, &refs).map_err(|_| "tray menu build failed".to_string())
}

/// Applies a tray menu definition to the existing tray icon.
pub fn apply_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    request: TraySetMenuRequest,
) -> Result<(), String> {
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| "tray icon is unavailable".to_string())?;

    let menu = build_tray_menu(app, &request.items)?;
    tray.set_menu(Some(menu)).map_err(|_| "tray menu apply failed".to_string())?;

    if let Some(tooltip) = request.tooltip {
        tray.set_tooltip(Some(tooltip))
            .map_err(|_| "tray tooltip apply failed".to_string())?;
    }

    Ok(())
}

/// Shows or hides the tray icon at runtime.
pub fn apply_tray_visibility<R: Runtime>(
    app: &AppHandle<R>,
    visible: bool,
) -> Result<(), String> {
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| "tray icon is unavailable".to_string())?;
    tray.set_visible(visible)
        .map_err(|_| "tray visibility change failed".to_string())
}

/// Handles tray icon interactions: left click restores the main window, and
/// menu activation is forwarded to the renderer as `sdkwork:tray:menu`.
pub fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: &TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
        | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => {
            restore_main_window(app);
        }
        TrayIconEvent::Enter { .. } => {}
        _ => {}
    }
}

/// Shows, unminimizes, and focuses the `main` window.
pub fn restore_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Emits a tray menu activation to the renderer.
///
/// The renderer receives `sdkwork:tray:menu` with the menu item id and decides
/// the product behavior (open settings, toggle visibility, quit, ...).
pub fn emit_tray_menu_event<R: Runtime>(app: &AppHandle<R>, menu_id: &str) {
    let _ = app.emit("sdkwork:tray:menu", menu_id.to_string());
}
