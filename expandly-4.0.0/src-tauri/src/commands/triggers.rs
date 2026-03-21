use tauri::Manager;

use std::path::PathBuf;

use tauri::State;

use crate::models::Trigger;
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
pub fn create_trigger(
    key: String,
    expansion_id: String,
    word_boundary: bool,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Trigger, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if key.len() > config.buffer_size {
        return Err(format!("Trigger key cannot exceed {} characters", config.buffer_size));
    }
    let trigger = Trigger { id: uuid::Uuid::new_v4().to_string(), key, expansion_id, word_boundary };
    config.triggers.push(trigger.clone());
    persist_config(&config_path(&app)?, &config);
    Ok(trigger)
}

#[tauri::command]
pub fn update_trigger(
    id: String,
    key: String,
    expansion_id: String,
    word_boundary: bool,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if key.len() > config.buffer_size {
        return Err(format!("Trigger key cannot exceed {} characters", config.buffer_size));
    }
    match config.triggers.iter_mut().find(|t| t.id == id) {
        Some(t) => { t.key = key; t.expansion_id = expansion_id; t.word_boundary = word_boundary; }
        None => return Err(format!("Trigger '{id}' not found")),
    }
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
pub fn delete_trigger(
    id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.triggers.retain(|t| t.id != id);
    persist_config(&config_path(&app)?, &config);
    Ok(())
}
