use tauri::State;


use crate::AppState;
use crate::db;

#[tauri::command]
pub fn update_track_stats(
    track_stats: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config_snapshot = {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        config.track_stats = track_stats;
        config.clone()
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_config_row(&db, &config_snapshot).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reset_stats(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        config.stats = crate::models::GlobalStats::default();
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM stats_per_day", []).map_err(|e| e.to_string())?;
    db.execute("DELETE FROM stats_per_expansion", []).map_err(|e| e.to_string())?;
    Ok(())
}