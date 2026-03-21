use tauri::Manager;

use std::path::PathBuf;

use tauri::State;

use crate::models::CustomVariable;
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
pub fn create_custom_variable(
    name: String,
    value: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<CustomVariable, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let variable = CustomVariable { id: uuid::Uuid::new_v4().to_string(), name, value };
    config.custom_variables.push(variable.clone());
    persist_config(&config_path(&app)?, &config);
    Ok(variable)
}

#[tauri::command]
pub fn update_custom_variable(
    id: String,
    name: String,
    value: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    match config.custom_variables.iter_mut().find(|v| v.id == id) {
        Some(v) => { v.name = name; v.value = value; }
        None => return Err(format!("Variable '{id}' not found")),
    }
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
pub fn delete_custom_variable(
    id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.custom_variables.retain(|v| v.id != id);
    persist_config(&config_path(&app)?, &config);
    Ok(())
}
