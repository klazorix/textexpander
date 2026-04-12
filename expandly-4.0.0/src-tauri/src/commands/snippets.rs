use tauri::State;


use crate::models::Expansion;
use crate::AppState;
use crate::db;

fn find_expansion<'a>(
    config: &'a mut crate::models::RootConfig,
    id: &str,
) -> Result<&'a mut Expansion, String> {
    config
        .expansions
        .get_mut(id)
        .ok_or_else(|| format!("Expansion '{id}' not found"))
}

#[tauri::command]
pub fn create_expansion(
    name: String,
    text: String,
    state: State<'_, AppState>,
) -> Result<Expansion, String> {
    let expansion = Expansion { id: uuid::Uuid::new_v4().to_string(), name, text };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_snippet(&db, &expansion).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.expansions.insert(expansion.id.clone(), expansion.clone());
    Ok(expansion)
}

#[tauri::command]
pub fn update_expansion(
    id: String,
    name: String,
    text: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let expansion = {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        let expansion = find_expansion(&mut config, &id)?;
        expansion.name = name;
        expansion.text = text;
        expansion.clone()
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_snippet(&db, &expansion).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_expansion(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_snippet(&db, &id).map_err(|e| e.to_string())?;
    drop(db);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.expansions.remove(&id);
    config.triggers.retain(|t| t.expansion_id != id);
    config.hotkeys.retain(|h| h.expansion_id != id);
    Ok(())
}
