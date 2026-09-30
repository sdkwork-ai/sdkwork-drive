//! Global shortcut host capability.
//!
//! Shortcut bindings are configured in the Settings Center and persisted by the
//! renderer. The native host only owns registration and dispatch: it registers
//! accelerators through `tauri-plugin-global-shortcut` and emits
//! `sdkwork:shortcut:triggered` back to the renderer, keeping accelerator
//! semantics out of feature code.
//!
//! Method names follow the bridge protocol (`sdkwork:shortcut:<action>`).

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutBindingPayload {
    pub id: String,
    pub accelerator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutActionRequest {
    pub id: String,
    pub accelerator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutRegisterManyRequest {
    pub bindings: Vec<ShortcutBindingPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutRegistrationResult {
    pub id: String,
    pub registered: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Event emitted when a registered accelerator fires.
pub const SHORTCUT_TRIGGERED_EVENT: &str = "sdkwork:shortcut:triggered";

/// Parses an accelerator string such as `Ctrl+Shift+X` or `CommandOrControl+X`.
fn parse_accelerator(accelerator: &str) -> Result<Shortcut, String> {
    let normalized = accelerator.trim();
    if normalized.is_empty() {
        return Err("shortcut accelerator is empty".to_string());
    }
    normalized
        .parse::<Shortcut>()
        .map_err(|_| format!("shortcut accelerator is invalid: {normalized}"))
}

/// Registers one accelerator. Existing registration for the same accelerator is
/// replaced so the Settings Center can rebind without leaking stale handlers.
pub fn register_shortcut<R: Runtime>(
    app: &AppHandle<R>,
    request: ShortcutActionRequest,
) -> Result<ShortcutRegistrationResult, String> {
    let shortcut = parse_accelerator(&request.accelerator)?;
    let manager = app.global_shortcut();

    if manager.is_registered(shortcut) {
        manager
            .unregister(shortcut)
            .map_err(|_| "shortcut unregister failed".to_string())?;
    }

    let id = request.id.clone();
    manager
        .on_shortcut(shortcut, move |app_handle, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = app_handle.emit(SHORTCUT_TRIGGERED_EVENT, id.clone());
            }
        })
        .map_err(|_| "shortcut registration failed".to_string())?;

    Ok(ShortcutRegistrationResult {
        id: request.id,
        registered: true,
        reason: None,
    })
}

/// Registers a batch of bindings, reporting per-binding outcome so one bad
/// accelerator does not discard the rest of the user's configuration.
pub fn register_shortcuts<R: Runtime>(
    app: &AppHandle<R>,
    request: ShortcutRegisterManyRequest,
) -> Vec<ShortcutRegistrationResult> {
    request
        .bindings
        .into_iter()
        .map(|binding| {
            let result = register_shortcut(
                app,
                ShortcutActionRequest {
                    id: binding.id.clone(),
                    accelerator: binding.accelerator,
                },
            );
            match result {
                Ok(outcome) => outcome,
                Err(reason) => ShortcutRegistrationResult {
                    id: binding.id,
                    registered: false,
                    reason: Some(reason),
                },
            }
        })
        .collect()
}

/// Unregisters a single accelerator.
pub fn unregister_shortcut<R: Runtime>(
    app: &AppHandle<R>,
    request: ShortcutActionRequest,
) -> Result<(), String> {
    let shortcut = parse_accelerator(&request.accelerator)?;
    let manager = app.global_shortcut();
    if !manager.is_registered(shortcut) {
        return Ok(());
    }
    manager
        .unregister(shortcut)
        .map_err(|_| "shortcut unregister failed".to_string())
}

/// Unregisters every accelerator owned by this host.
///
/// Used when the user disables global shortcuts or signs out, so accelerators
/// are never left bound to a dead renderer.
pub fn unregister_all_shortcuts<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|_| "shortcut unregister-all failed".to_string())
}
