mod models;
mod engine;
mod helpers;
mod commands;

use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;

use tauri::Manager;

use models::RootConfig;
use commands::*;

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

fn load_or_create_config(app: &tauri::AppHandle) -> RootConfig {
    let path = match config_path(app) {
        Ok(p) => p,
        Err(e) => { eprintln!("[expandly] {e}"); return RootConfig::default(); }
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<RootConfig>(&raw) {
                Ok(config) => { println!("[expandly] Config loaded from {:?}", path); return config; }
                Err(e) => {
                    eprintln!("[expandly] Corrupt config ({e}), backing up.");
                    let _ = fs::rename(&path, path.with_extension("json.bak"));
                }
            },
            Err(e) => eprintln!("[expandly] Could not read config: {e}"),
        }
    }

    let mut default_config = RootConfig::default();
    default_config.version = app.package_info().version.to_string();
    for var in &mut default_config.custom_variables {
        if var.name == "version" {
            var.value = app.package_info().version.to_string();
        }
    }

    helpers::persist_config(&path, &default_config);
    println!("[expandly] Default config written to {:?}", path);
    default_config
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
            let config = load_or_create_config(&app.handle());
            let config = Arc::new(Mutex::new(config));
            let path = config_path(&app.handle()).unwrap_or_default();

            engine::start(Arc::clone(&config), path);

            let (minimise_to_tray, launch_minimised) = {
                let cfg = config.lock().unwrap();
                (cfg.minimise_to_tray, cfg.launch_minimised)
            };

            app.manage(AppState { config });

            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
            use tauri::menu::{Menu, MenuItem};

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
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // config
            config::get_config,
            config::save_config,
            config::export_config,
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
            // stats
            stats::record_expansion,
            stats::update_track_stats,
            stats::reset_stats,
            // system
            system::get_app_version,
            system::open_url,
            system::close_splash,
            system::save_sound_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running expandly");
}
