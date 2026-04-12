use tauri::State;

use crate::models::RootConfig;
use crate::AppState;
use crate::db;
use crate::backup;

fn replace_config(state: &State<'_, AppState>, config: RootConfig) -> Result<(), String> {
    *state.config.lock().map_err(|e| e.to_string())? = config;
    Ok(())
}

fn write_and_replace_config(state: &State<'_, AppState>, config: RootConfig) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::write_all(&db, &config).map_err(|e| e.to_string())?;
    replace_config(state, config)
}

fn export_backup_json(state: &State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    backup::export_to_json(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<RootConfig, String> {
    Ok(state.config.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
pub fn save_config(
    new_config: RootConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    write_and_replace_config(&state, new_config)
}

#[tauri::command]
pub async fn export_config(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;

    let json = export_backup_json(&state)?;

    let file_path = app
        .dialog()
        .file()
        .set_file_name("expandly-backup.json")
        .add_filter("JSON", &["json"])
        .blocking_save_file();

    if let Some(path) = file_path {
        std::fs::write(path.as_path().unwrap(), json)
            .map_err(|e| format!("Failed to write file: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
pub fn import_config(
    json: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    backup::import_from_json(&db, &json)?;
    let new_config = db::load_all(&db).map_err(|e| e.to_string())?;
    replace_config(&state, new_config)
}

#[tauri::command]
pub fn reset_config(state: State<'_, AppState>) -> Result<(), String> {
    write_and_replace_config(&state, RootConfig::default())
}
