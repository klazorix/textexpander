mod models;
mod engine;
mod helpers;

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
    
    let mut default_config = RootConfig::default();
    default_config.version = app.package_info().version.to_string();

    // Also update the version custom variable if it exists
    for var in &mut default_config.custom_variables {
        if var.name == "version" {
            var.value = app.package_info().version.to_string();
        }
    }

    persist_config(&path, &default_config);
    println!("[expandly] Default config written to {:?}", path);
    default_config
}

// ── About ────────────────────────────────────────────────────────────────

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
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
fn update_expansion_delay(
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
fn update_buffer_size(
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
fn update_performance_settings(
    hotkey_delay_ms: u64,
    engine_restart_delay_ms: u64,
    clear_buffer_on_switch: bool,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.hotkey_delay_ms = hotkey_delay_ms;
    config.engine_restart_delay_ms = engine_restart_delay_ms;
    config.clear_buffer_on_switch = clear_buffer_on_switch;
    persist_config(&config_path(&app)?, &config);
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
    if key.len() > config.buffer_size {
        return Err(format!("Trigger key cannot exceed {} characters", config.buffer_size));
    }
    let trigger = Trigger { id: uuid::Uuid::new_v4().to_string(), key, expansion_id, word_boundary };
    config.triggers.push(trigger.clone());
    persist_config(&config_path(&app)?, &config);
    Ok(trigger)
}

#[tauri::command]
fn update_trigger(id: String, key: String, expansion_id: String, word_boundary: bool, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if key.len() > config.buffer_size {
        return Err(format!("Trigger key cannot exceed {} characters", config.buffer_size));
    }
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

// ── Statistics ────────────────────────────────────────────────────────────

#[tauri::command]
fn record_expansion(
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
        let days = secs / 86400;
        let (y, m, d) = crate::engine::days_from_epoch_pub(days as i64);
        format!("{:04}-{:02}-{:02}", y, m, d)
    };

    *config.stats.expansions_per_day.entry(today).or_insert(0) += 1;
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

#[tauri::command]
fn update_track_stats(
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
fn reset_stats(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.stats = crate::models::GlobalStats::default();
    persist_config(&config_path(&app)?, &config);
    Ok(())
}

// ── Splash ────────────────────────────────────────────────────────────────

#[tauri::command]
fn close_splash(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(splash) = app.get_webview_window("splash") {
        splash.close().map_err(|e| e.to_string())?;
    }
    if let Some(main) = app.get_webview_window("main") {
        // Respect launch_minimised setting
        let config = app.state::<AppState>();
        let minimised = config.config.lock().unwrap().launch_minimised;
        if !minimised {
            main.show().map_err(|e| e.to_string())?;
            main.set_focus().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// ── Run ───────────────────────────────────────────────────────────────────

#[tauri::command]
fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

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
            let path = config_path(&app.handle()).unwrap_or_default();

            engine::start(Arc::clone(&config), path);

            // Read startup settings before managing state
            let (minimise_to_tray, launch_minimised) = {
                let cfg = config.lock().unwrap();
                let a = cfg.minimise_to_tray;
                let b = cfg.launch_minimised;
                (a, b)
            };

            app.manage(AppState { config });

            use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
            use tauri::menu::{Menu, MenuItem};

            let quit = MenuItem::with_id(app, "quit", "Quit Expandly", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::with_id("expandly-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Expandly")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Apply taskbar setting
            if let Some(window) = app.get_webview_window("main") {

                // Launch minimised
                if launch_minimised {
                    let _ = window.hide();
                }

                // Minimise to tray on close
                if minimise_to_tray {
                    let window_clone = window.clone();
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            api.prevent_close();
                            let _ = window_clone.hide();
                        }
                    });
                }
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
            update_system_settings,
            save_sound_file,
            export_config,
            reset_config,
            get_app_version,
            open_url,
            record_expansion,
            update_track_stats,
            reset_stats,
            close_splash,
            update_expansion_delay,
            update_buffer_size,
            update_performance_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running expandly");
}