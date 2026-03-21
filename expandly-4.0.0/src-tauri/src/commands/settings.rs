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
pub fn update_engine_settings(
    enabled: bool,
    sound_enabled: bool,
    sound_path: Option<String>,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.enabled = enabled;
    config.sound_enabled = sound_enabled;
    config.sound_path = sound_path;
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
pub fn update_system_settings(
    launch_at_startup: bool,
    launch_minimised: bool,
    minimise_to_tray: bool,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.launch_at_startup = launch_at_startup;
    config.launch_minimised = launch_minimised;
    config.minimise_to_tray = minimise_to_tray;
    persist_config(&config_path(&app)?, &config);
    drop(config);

    {
        use tauri_plugin_autostart::ManagerExt;
        if launch_at_startup {
            app.autolaunch().enable().map_err(|e| format!("{e}"))?;
        } else {
            app.autolaunch().disable().map_err(|e| format!("{e}"))?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn update_expansion_delay(
    expansion_delay_ms: u64,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.expansion_delay_ms = expansion_delay_ms;
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
pub fn update_buffer_size(
    buffer_size: usize,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.buffer_size = buffer_size;
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
pub fn update_performance_settings(
    hotkey_delay_ms: u64,
    clear_buffer_on_switch: bool,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.hotkey_delay_ms = hotkey_delay_ms;
    config.clear_buffer_on_switch = clear_buffer_on_switch;
    persist_config(&config_path(&app)?, &config);
    Ok(())
}
