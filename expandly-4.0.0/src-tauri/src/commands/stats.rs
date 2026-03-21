use tauri::Manager;

use std::path::PathBuf;

use tauri::State;

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
pub fn record_expansion(
    chars_saved: u64,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if !config.track_stats {
        return Ok(());
    }
    config.stats.total_expansions += 1;
    config.stats.total_chars_saved += chars_saved;

    let today = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (y, m, d) = crate::engine::days_from_epoch_pub((secs / 86400) as i64);
        format!("{:04}-{:02}-{:02}", y, m, d)
    };

    *config.stats.expansions_per_day.entry(today).or_insert(0) += 1;
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
pub fn update_track_stats(
    track_stats: bool,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.track_stats = track_stats;
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
pub fn reset_stats(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.stats = crate::models::GlobalStats::default();
    persist_config(&config_path(&app)?, &config);
    Ok(())
}
