use tauri::State;


use crate::AppState;
use crate::db;

fn save_config(state: &State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_config_row(&db, &config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_track_stats(
    track_stats: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.config.lock().map_err(|e| e.to_string())?.track_stats = track_stats;
    save_config(&state)
}

#[tauri::command]
pub fn reset_stats(state: State<'_, AppState>) -> Result<(), String> {
    state.config.lock().map_err(|e| e.to_string())?.stats = crate::models::GlobalStats::default();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM stats_per_day", []).map_err(|e| e.to_string())?;
    db.execute("DELETE FROM stats_per_expansion", []).map_err(|e| e.to_string())?;
    Ok(())
}
