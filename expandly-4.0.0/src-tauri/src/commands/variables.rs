use tauri::State;


use crate::models::CustomVariable;
use crate::AppState;
use crate::db;

#[tauri::command]
pub fn create_custom_variable(
    name: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<CustomVariable, String> {
    let variable = CustomVariable { id: uuid::Uuid::new_v4().to_string(), name, value };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_variable(&db, &variable).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.custom_variables.push(variable.clone());
    Ok(variable)
}

#[tauri::command]
pub fn update_custom_variable(
    id: String,
    name: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let variable = {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        match config.custom_variables.iter_mut().find(|v| v.id == id) {
            Some(v) => { v.name = name; v.value = value; }
            None => return Err(format!("Variable '{id}' not found")),
        }
        config.custom_variables.iter().find(|v| v.id == id).unwrap().clone()
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_variable(&db, &variable).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_custom_variable(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_variable(&db, &id).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.custom_variables.retain(|v| v.id != id);
    Ok(())
}