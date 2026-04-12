use tauri::State;


use crate::models::CustomVariable;
use crate::AppState;
use crate::db;

fn save_variable(db: &rusqlite::Connection, variable: &CustomVariable) -> Result<(), String> {
    db::save_variable(db, variable).map_err(|e| e.to_string())
}

fn find_variable<'a>(config: &'a mut crate::models::RootConfig, id: &str) -> Result<&'a mut CustomVariable, String> {
    config
        .custom_variables
        .iter_mut()
        .find(|variable| variable.id == id)
        .ok_or_else(|| format!("Variable '{id}' not found"))
}

#[tauri::command]
pub fn create_custom_variable(
    name: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<CustomVariable, String> {
    let variable = CustomVariable { id: uuid::Uuid::new_v4().to_string(), name, value };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    save_variable(&db, &variable)?;
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
        let variable = find_variable(&mut config, &id)?;
        variable.name = name;
        variable.value = value;
        variable.clone()
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    save_variable(&db, &variable)?;
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
