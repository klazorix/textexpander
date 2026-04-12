use tauri::State;


use crate::models::Hotkey;
use crate::AppState;
use crate::db;

fn find_hotkey<'a>(config: &'a mut crate::models::RootConfig, id: &str) -> Result<&'a mut Hotkey, String> {
    config
        .hotkeys
        .iter_mut()
        .find(|hotkey| hotkey.id == id)
        .ok_or_else(|| format!("Hotkey '{id}' not found"))
}

#[tauri::command]
pub fn create_hotkey(
    keys: String,
    expansion_id: String,
    state: State<'_, AppState>,
) -> Result<Hotkey, String> {
    let hotkey = Hotkey { id: uuid::Uuid::new_v4().to_string(), keys, expansion_id };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_hotkey(&db, &hotkey).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.hotkeys.push(hotkey.clone());
    Ok(hotkey)
}

#[tauri::command]
pub fn update_hotkey(
    id: String,
    keys: String,
    expansion_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let hotkey = {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        let hotkey = find_hotkey(&mut config, &id)?;
        hotkey.keys = keys;
        hotkey.expansion_id = expansion_id;
        hotkey.clone()
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_hotkey(&db, &hotkey).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_hotkey(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_hotkey(&db, &id).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.hotkeys.retain(|h| h.id != id);
    Ok(())
}
