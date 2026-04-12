use tauri::State;

use crate::AppState;
use crate::db;
use crate::models::RootConfig;

fn update_config_row<F>(state: &State<'_, AppState>, update: F) -> Result<RootConfig, String>
where
    F: FnOnce(&mut RootConfig),
{
    let snapshot = {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        update(&mut config);
        config.clone()
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::save_config_row(&db, &snapshot).map_err(|e| e.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
pub fn update_engine_settings(
    enabled: bool,
    sound_enabled: bool,
    sound_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    update_config_row(&state, |config| {
        config.enabled = enabled;
        config.sound_enabled = sound_enabled;
        config.sound_path = sound_path;
    })?;
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
    update_config_row(&state, |config| {
        config.launch_at_startup = launch_at_startup;
        config.launch_minimised = launch_minimised;
        config.minimise_to_tray = minimise_to_tray;
    })?;

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
) -> Result<(), String> {
    update_config_row(&state, |config| {
        config.expansion_delay_ms = expansion_delay_ms;
    })?;
    Ok(())
}

#[tauri::command]
pub fn update_buffer_size(
    buffer_size: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    update_config_row(&state, |config| {
        config.buffer_size = buffer_size;
    })?;
    Ok(())
}

#[tauri::command]
pub fn update_performance_settings(
    hotkey_delay_ms: u64,
    clear_buffer_on_switch: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    update_config_row(&state, |config| {
        config.hotkey_delay_ms = hotkey_delay_ms;
        config.clear_buffer_on_switch = clear_buffer_on_switch;
    })?;
    Ok(())
}

#[tauri::command]
pub fn update_debug_settings(
    debug_enabled: bool,
    debug_level: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    update_config_row(&state, |config| {
        config.debug_enabled = debug_enabled;
        config.debug_level = debug_level.clone();
    })?;
    // Update the live logger
    {
        let mut logger = state.logger.lock().map_err(|e| e.to_string())?;
        logger.enabled = debug_enabled;
        logger.level = crate::logger::LogLevel::from_str(&debug_level);
    }
    Ok(())
}
