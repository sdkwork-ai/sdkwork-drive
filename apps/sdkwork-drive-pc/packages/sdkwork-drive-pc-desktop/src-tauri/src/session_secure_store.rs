use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use keyring::Entry;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const KEYRING_SERVICE: &str = "sdkwork-drive-pc";
const LEGACY_SNAPSHOT_FILE: &str = "secure-session.json";
const KEY_INDEX_FILE: &str = "secure-session-keys.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct SecureSessionSnapshot {
    values: HashMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SecureSessionKeyIndex {
    keys: Vec<String>,
}

pub struct SecureSessionState {
    keys_path: PathBuf,
    keys: Mutex<Vec<String>>,
}

impl SecureSessionState {
    fn new(app_data_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;

        let keys_path = app_data_dir.join(KEY_INDEX_FILE);
        let legacy_path = app_data_dir.join(LEGACY_SNAPSHOT_FILE);
        let mut keys = load_key_index(&keys_path);

        if legacy_path.exists() {
            migrate_legacy_snapshot(&legacy_path, &mut keys)?;
            let _ = fs::remove_file(&legacy_path);
        }

        persist_key_index(&keys_path, &keys)?;
        Ok(Self {
            keys_path,
            keys: Mutex::new(keys),
        })
    }

    fn track_key(&self, key: &str) -> Result<(), String> {
        let mut keys = self
            .keys
            .lock()
            .map_err(|_| "secure session lock poisoned".to_string())?;
        if !keys.iter().any(|existing| existing == key) {
            keys.push(key.to_string());
            persist_key_index(&self.keys_path, &keys)?;
        }
        Ok(())
    }

    fn untrack_key(&self, key: &str) -> Result<(), String> {
        let mut keys = self
            .keys
            .lock()
            .map_err(|_| "secure session lock poisoned".to_string())?;
        let original_len = keys.len();
        keys.retain(|existing| existing != key);
        if keys.len() != original_len {
            persist_key_index(&self.keys_path, &keys)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecureSessionKeyRequest {
    key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecureSessionWriteRequest {
    key: String,
    value: String,
}

fn keyring_entry(key: &str) -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, key).map_err(|error| error.to_string())
}

fn load_key_index(path: &Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }
    let raw = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str::<SecureSessionKeyIndex>(&raw)
        .map(|index| index.keys)
        .unwrap_or_default()
}

fn persist_key_index(path: &Path, keys: &[String]) -> Result<(), String> {
    let payload = SecureSessionKeyIndex {
        keys: keys.to_vec(),
    };
    let serialized = serde_json::to_string_pretty(&payload)
        .map_err(|error: serde_json::Error| error.to_string())?;
    fs::write(path, serialized).map_err(|error| error.to_string())
}

fn migrate_legacy_snapshot(legacy_path: &Path, keys: &mut Vec<String>) -> Result<(), String> {
    let raw = fs::read_to_string(legacy_path).map_err(|error| error.to_string())?;
    let snapshot = serde_json::from_str::<SecureSessionSnapshot>(&raw).unwrap_or_default();
    for (key, value) in snapshot.values {
        keyring_entry(&key)?
            .set_password(&value)
            .map_err(|error| error.to_string())?;
        if !keys.iter().any(|existing| existing == &key) {
            keys.push(key);
        }
    }
    Ok(())
}

pub fn init_secure_session_state(app: &AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let state = SecureSessionState::new(app_data_dir)?;
    app.manage(state);
    Ok(())
}

#[tauri::command]
pub fn write_secure_session_value(
    state: tauri::State<'_, SecureSessionState>,
    request: SecureSessionWriteRequest,
) -> Result<(), String> {
    keyring_entry(&request.key)?
        .set_password(&request.value)
        .map_err(|error| error.to_string())?;
    state.track_key(&request.key)
}

#[tauri::command]
pub fn remove_secure_session_value(
    state: tauri::State<'_, SecureSessionState>,
    request: SecureSessionKeyRequest,
) -> Result<(), String> {
    if let Ok(entry) = keyring_entry(&request.key) {
        let _ = entry.delete_credential();
    }
    state.untrack_key(&request.key)
}

#[tauri::command]
pub fn clear_secure_session_values(state: tauri::State<'_, SecureSessionState>) -> Result<(), String> {
    let keys = state
        .keys
        .lock()
        .map_err(|_| "secure session lock poisoned".to_string())?
        .clone();
    for key in keys {
        if let Ok(entry) = keyring_entry(&key) {
            let _ = entry.delete_credential();
        }
    }
    {
        let mut keys = state
            .keys
            .lock()
            .map_err(|_| "secure session lock poisoned".to_string())?;
        keys.clear();
        persist_key_index(&state.keys_path, &keys)?;
    }
    Ok(())
}

#[tauri::command]
pub fn read_secure_session_snapshot(
    state: tauri::State<'_, SecureSessionState>,
) -> Result<HashMap<String, String>, String> {
    let keys = state
        .keys
        .lock()
        .map_err(|_| "secure session lock poisoned".to_string())?
        .clone();
    let mut values = HashMap::new();
    for key in keys {
        if let Ok(entry) = keyring_entry(&key) {
            if let Ok(value) = entry.get_password() {
                values.insert(key, value);
            }
        }
    }
    Ok(values)
}
