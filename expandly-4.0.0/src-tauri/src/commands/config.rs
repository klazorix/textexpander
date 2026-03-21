use tauri::State;


use crate::models::RootConfig;
use crate::AppState;
use crate::db;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<RootConfig, String> {
    Ok(state.config.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
pub fn save_config(
    new_config: RootConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Write to DB first, then update in-memory
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::write_all(&db, &new_config).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    *config = new_config;
    Ok(())
}

#[tauri::command]
pub async fn export_config(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;

    let json = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?
    };

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
pub fn reset_config(state: State<'_, AppState>) -> Result<(), String> {
    let new_config = RootConfig::default();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::write_all(&db, &new_config).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    *config = new_config;
    Ok(())
}