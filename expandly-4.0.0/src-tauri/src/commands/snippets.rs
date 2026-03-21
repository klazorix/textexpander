use tauri::Manager;

use std::path::PathBuf;

use tauri::State;

use crate::models::Expansion;
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
pub fn create_expansion(
    name: String,
    text: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Expansion, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let expansion = Expansion { id: id.clone(), name, text };
    config.expansions.insert(id, expansion.clone());
    persist_config(&config_path(&app)?, &config);
    Ok(expansion)
}

#[tauri::command]
pub fn update_expansion(
    id: String,
    name: String,
    text: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    match config.expansions.get_mut(&id) {
        Some(exp) => { exp.name = name; exp.text = text; }
        None => return Err(format!("Expansion '{id}' not found")),
    }
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
pub fn delete_expansion(
    id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.expansions.remove(&id);
    config.triggers.retain(|t| t.expansion_id != id);
    config.hotkeys.retain(|h| h.expansion_id != id);
    persist_config(&config_path(&app)?, &config);
    Ok(())
}
