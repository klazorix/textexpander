mod models;
mod engine;
mod commands;
mod db;
mod backup;
mod logger;

use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use tauri::Manager;

use models::RootConfig;
use logger::{LogLevel, SharedLogger, make_logger, purge_old_logs};
use commands::*;

pub struct AppState {
    pub config: Arc<Mutex<RootConfig>>,
    pub db:     Arc<Mutex<rusqlite::Connection>>,
    pub logger: SharedLogger,
}

pub(crate) fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data directory: {e}"))
}

fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join("expandly.db"))
}

fn config_json_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join("config.json"))
}

fn open_db(app: &tauri::AppHandle) -> Result<rusqlite::Connection, String> {
    let path = db_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create app data dir: {e}"))?;
    }
    rusqlite::Connection::open(&path)
        .map_err(|e| format!("Could not open database: {e}"))
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let quit = MenuItem::with_id(app, "quit", "Quit Expandly", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let _tray = TrayIconBuilder::with_id("expandly-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Expandly")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "show" => show_main_window(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(&tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn configure_main_window(app: &mut tauri::App, launch_minimised: bool, minimise_to_tray: bool) {
    if let Some(window) = app.get_webview_window("main") {
        if launch_minimised {
            let _ = window.hide();
        }
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
}

fn load_or_create_config(app: &tauri::AppHandle) -> (RootConfig, rusqlite::Connection) {
    let conn = match open_db(app) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[expandly] DB open failed: {e}");
            rusqlite::Connection::open_in_memory().expect("in-memory DB failed")
        }
    };

    if let Err(e) = db::create_schema(&conn) {
        eprintln!("[expandly] Schema creation failed: {e}");
    }

    db::migrate_schema(&conn);

    let json_path = config_json_path(app).unwrap_or_default();
    if let Err(e) = db::migrate_if_needed(&conn, &json_path) {
        eprintln!("[expandly] Migration failed: {e}");
    }

    let mut config = match db::load_all(&conn) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[expandly] Failed to load config from DB ({e}), using defaults");
            RootConfig::default()
        }
    };

    let version = app.package_info().version.to_string();
    config.version = version.clone();
    for var in &mut config.custom_variables {
        if var.name == "version" {
            var.value = version.clone();
        }
    }

    (config, conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let (config, conn) = load_or_create_config(&app.handle());

            let app_data = data_dir(&app.handle()).unwrap_or_default();
            purge_old_logs(&app_data);
            let log = make_logger(
                config.debug_enabled,
                LogLevel::from_str(&config.debug_level),
                app_data,
            );
            log.lock().unwrap().verbose("[startup] Expandly starting up");

            let config = Arc::new(Mutex::new(config));
            let db     = Arc::new(Mutex::new(conn));

            engine::start(Arc::clone(&config), Arc::clone(&db), Arc::clone(&log));

            let (minimise_to_tray, launch_minimised) = {
                let cfg = config.lock().unwrap();
                (cfg.minimise_to_tray, cfg.launch_minimised)
            };

            app.manage(AppState { config, db, logger: log });
            setup_tray(app)?;
            configure_main_window(app, launch_minimised, minimise_to_tray);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // config
            config::get_config,
            config::save_config,
            config::export_config,
            config::import_config,
            config::reset_config,
            // snippets
            snippets::create_expansion,
            snippets::update_expansion,
            snippets::delete_expansion,
            // triggers
            triggers::create_trigger,
            triggers::update_trigger,
            triggers::delete_trigger,
            // hotkeys
            hotkeys::create_hotkey,
            hotkeys::update_hotkey,
            hotkeys::delete_hotkey,
            // variables
            variables::create_custom_variable,
            variables::update_custom_variable,
            variables::delete_custom_variable,
            // settings
            settings::update_engine_settings,
            settings::update_system_settings,
            settings::update_expansion_delay,
            settings::update_buffer_size,
            settings::update_performance_settings,
            settings::update_debug_settings,
            // stats
            stats::update_track_stats,
            stats::reset_stats,
            // system
            system::get_app_version,
            system::open_url,
            system::open_debug_logs_folder,
            system::close_splash,
            system::save_sound_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running expandly");
}
