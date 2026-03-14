mod models;
mod engine;

use models::{CustomVariable, Expansion, Hotkey, RootConfig, Trigger};

use std::{fs, path::PathBuf, sync::{Arc, Mutex}};

use tauri::{Manager, State};

pub struct AppState {
    pub config: Arc<Mutex<RootConfig>>,
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data directory: {e}"))?;
    Ok(data_dir.join("config.json"))
}

pub fn persist_config(path: &PathBuf, config: &RootConfig) {
    let json = match serde_json::to_string_pretty(config) {
        Ok(j) => j,
        Err(e) => { eprintln!("[text-expander] Failed to serialise config: {e}"); return; }
    };
    let tmp_path = path.with_extension("json.tmp");
    if let Err(e) = fs::write(&tmp_path, &json) {
        eprintln!("[text-expander] Failed to write temp config: {e}"); return;
    }
    if let Err(e) = fs::rename(&tmp_path, path) {
        eprintln!("[text-expander] Failed to replace config.json: {e}");
    }
}

fn load_or_create_config(app: &tauri::AppHandle) -> RootConfig {
    let path = match config_path(app) {
        Ok(p) => p,
        Err(e) => { eprintln!("[text-expander] {e}"); return RootConfig::default(); }
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<RootConfig>(&raw) {
                Ok(config) => { println!("[text-expander] Config loaded from {:?}", path); return config; }
                Err(e) => {
                    eprintln!("[text-expander] Corrupt config ({e}), backing up.");
                    let _ = fs::rename(&path, path.with_extension("json.bak"));
                }
            },
            Err(e) => eprintln!("[text-expander] Could not read config: {e}"),
        }
    }
    let default_config = RootConfig::default();
    persist_config(&path, &default_config);
    default_config
}

// ── Config ────────────────────────────────────────────────────────────────

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<RootConfig, String> {
    Ok(state.config.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
fn save_config(new_config: RootConfig, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    *config = new_config.clone();
    persist_config(&config_path(&app)?, &new_config);
    Ok(())
}

#[tauri::command]
fn update_engine_settings(
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
fn save_sound_file(
    file_name: String,
    file_data: Vec<u8>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data directory: {e}"))?;

    let sounds_dir = data_dir.join("sounds");
    std::fs::create_dir_all(&sounds_dir)
        .map_err(|e| format!("Could not create sounds directory: {e}"))?;

    let dest = sounds_dir.join(&file_name);
    std::fs::write(&dest, &file_data)
        .map_err(|e| format!("Could not write sound file: {e}"))?;

    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
async fn export_config(state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;

    let json = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?
    };

    let file_path = app
        .dialog()
        .file()
        .set_file_name("expandly-backup.json")
        .add_filter("JSON", &["json"])
        .blocking_save_file();

    if let Some(path) = file_path {
        std::fs::write(path.as_path().unwrap(), json)
            .map_err(|e| format!("Failed to write file: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
fn reset_config(state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    *config = RootConfig::default();
    let path = config_path(&app)?;
    persist_config(&path, &config);
    Ok(())
}

#[tauri::command]
fn update_system_settings(
    launch_at_startup: bool,
    minimise_to_tray: bool,
    show_in_taskbar: bool,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.launch_at_startup = launch_at_startup;
    config.minimise_to_tray = minimise_to_tray;
    config.show_in_taskbar = show_in_taskbar;
    persist_config(&config_path(&app)?, &config);
    drop(config);

    // Apply autostart
    {
        use tauri_plugin_autostart::ManagerExt;
        if launch_at_startup {
            app.autolaunch().enable().map_err(|e| format!("{e}"))?;
        } else {
            app.autolaunch().disable().map_err(|e| format!("{e}"))?;
        }
    }

    // Apply taskbar visibility
    if let Some(window) = app.get_webview_window("main") {
        window.set_skip_taskbar(!show_in_taskbar).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ── Expansions ────────────────────────────────────────────────────────────

#[tauri::command]
fn create_expansion(name: String, text: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<Expansion, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let expansion = Expansion { id: id.clone(), name, text };
    config.expansions.insert(id, expansion.clone());
    persist_config(&config_path(&app)?, &config);
    Ok(expansion)
}

#[tauri::command]
fn update_expansion(id: String, name: String, text: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    match config.expansions.get_mut(&id) {
        Some(exp) => { exp.name = name; exp.text = text; }
        None => return Err(format!("Expansion '{id}' not found")),
    }
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
fn delete_expansion(id: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.expansions.remove(&id);
    config.triggers.retain(|t| t.expansion_id != id);
    config.hotkeys.retain(|h| h.expansion_id != id);
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

// ── Triggers ──────────────────────────────────────────────────────────────

#[tauri::command]
fn create_trigger(key: String, expansion_id: String, word_boundary: bool, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<Trigger, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let trigger = Trigger { id: uuid::Uuid::new_v4().to_string(), key, expansion_id, word_boundary };
    config.triggers.push(trigger.clone());
    persist_config(&config_path(&app)?, &config);
    Ok(trigger)
}

#[tauri::command]
fn update_trigger(id: String, key: String, expansion_id: String, word_boundary: bool, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    match config.triggers.iter_mut().find(|t| t.id == id) {
        Some(t) => { t.key = key; t.expansion_id = expansion_id; t.word_boundary = word_boundary; }
        None => return Err(format!("Trigger '{id}' not found")),
    }
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
fn delete_trigger(id: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.triggers.retain(|t| t.id != id);
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

// ── Hotkeys ───────────────────────────────────────────────────────────────

#[tauri::command]
fn create_hotkey(keys: String, expansion_id: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<Hotkey, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let hotkey = Hotkey { id: uuid::Uuid::new_v4().to_string(), keys, expansion_id };
    config.hotkeys.push(hotkey.clone());
    persist_config(&config_path(&app)?, &config);
    Ok(hotkey)
}

#[tauri::command]
fn update_hotkey(id: String, keys: String, expansion_id: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    match config.hotkeys.iter_mut().find(|h| h.id == id) {
        Some(h) => { h.keys = keys; h.expansion_id = expansion_id; }
        None => return Err(format!("Hotkey '{id}' not found")),
    }
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
fn delete_hotkey(id: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.hotkeys.retain(|h| h.id != id);
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

// ── Custom Variables ──────────────────────────────────────────────────────

#[tauri::command]
fn create_custom_variable(name: String, value: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<CustomVariable, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let variable = CustomVariable { id: uuid::Uuid::new_v4().to_string(), name, value };
    config.custom_variables.push(variable.clone());
    persist_config(&config_path(&app)?, &config);
    Ok(variable)
}

#[tauri::command]
fn update_custom_variable(id: String, name: String, value: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    match config.custom_variables.iter_mut().find(|v| v.id == id) {
        Some(v) => { v.name = name; v.value = value; }
        None => return Err(format!("Variable '{id}' not found")),
    }
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
fn delete_custom_variable(id: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.custom_variables.retain(|v| v.id != id);
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

// ── Run ───────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None
        ))
        .setup(|app| {
            let config = load_or_create_config(&app.handle());
            let config = Arc::new(Mutex::new(config));

            // Start the keystroke engine on a background thread
            engine::start(Arc::clone(&config));

            app.manage(AppState { config });

            // Apply minimise to tray on close if enabled
            let minimise_to_tray = {
                let cfg = app.state::<AppState>();
                let lock = cfg.config.lock().unwrap();
                let val = lock.minimise_to_tray;
                val
            };

            if minimise_to_tray {
                let window = app.get_webview_window("main").unwrap();
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        window_clone.hide().unwrap();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            create_expansion,
            update_expansion,
            delete_expansion,
            create_trigger,
            update_trigger,
            delete_trigger,
            create_hotkey,
            update_hotkey,
            delete_hotkey,
            create_custom_variable,
            update_custom_variable,
            delete_custom_variable,
            update_engine_settings,
            save_sound_file,
            export_config,
            reset_config,
            update_system_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running expandly");
}