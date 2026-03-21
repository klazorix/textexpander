use tauri::State;


use crate::models::Trigger;
use crate::AppState;
use crate::db;

#[tauri::command]
pub fn create_trigger(
    key: String,
    expansion_id: String,
    word_boundary: bool,
    state: State<'_, AppState>,
) -> Result<Trigger, String> {
    let buffer_size = state.config.lock().map_err(|e| e.to_string())?.buffer_size;
    if key.len() > buffer_size {
        return Err(format!("Trigger key cannot exceed {buffer_size} characters"));
    }
    let trigger = Trigger { id: uuid::Uuid::new_v4().to_string(), key, expansion_id, word_boundary };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_trigger(&db, &trigger).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.triggers.push(trigger.clone());
    Ok(trigger)
}

#[tauri::command]
pub fn update_trigger(
    id: String,
    key: String,
    expansion_id: String,
    word_boundary: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let trigger = {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        if key.len() > config.buffer_size {
            return Err(format!("Trigger key cannot exceed {} characters", config.buffer_size));
        }
        match config.triggers.iter_mut().find(|t| t.id == id) {
            Some(t) => { t.key = key; t.expansion_id = expansion_id; t.word_boundary = word_boundary; }
            None => return Err(format!("Trigger '{id}' not found")),
        }
        config.triggers.iter().find(|t| t.id == id).unwrap().clone()
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_trigger(&db, &trigger).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_trigger(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_trigger(&db, &id).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.triggers.retain(|t| t.id != id);
    Ok(())
}