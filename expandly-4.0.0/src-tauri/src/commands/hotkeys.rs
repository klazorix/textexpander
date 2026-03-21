use tauri::Manager;

use std::path::PathBuf;

use tauri::State;

use crate::models::Hotkey;
use crate::helpers::persist_config;
use crate::AppState;

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data directory: {e}"))?;
    Ok(data_dir.join("config.json"))
}

#[tauri::command]
pub fn create_hotkey(
    keys: String,
    expansion_id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Hotkey, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let hotkey = Hotkey { id: uuid::Uuid::new_v4().to_string(), keys, expansion_id };
    config.hotkeys.push(hotkey.clone());
    persist_config(&config_path(&app)?, &config);
    Ok(hotkey)
}

#[tauri::command]
pub fn update_hotkey(
    id: String,
    keys: String,
    expansion_id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    match config.hotkeys.iter_mut().find(|h| h.id == id) {
        Some(h) => { h.keys = keys; h.expansion_id = expansion_id; }
        None => return Err(format!("Hotkey '{id}' not found")),
    }
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
pub fn delete_hotkey(
    id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.hotkeys.retain(|h| h.id != id);
    persist_config(&config_path(&app)?, &config);
    Ok(())
}
