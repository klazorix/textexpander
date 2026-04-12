use tauri::Manager;

use crate::AppState;

fn ensure_data_subdir(app: &tauri::AppHandle, name: &str) -> Result<std::path::PathBuf, String> {
    let dir = crate::data_dir(app)?.join(name);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Could not create {name} directory: {e}"))?;
    Ok(dir)
}

#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_debug_logs_folder(app: tauri::AppHandle) -> Result<(), String> {
    let dir = ensure_data_subdir(&app, "debug")?;
    open::that(dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_splash(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(splash) = app.get_webview_window("splash") {
        splash.close().map_err(|e| e.to_string())?;
    }
    if let Some(main) = app.get_webview_window("main") {
        let minimised = app
            .state::<AppState>()
            .config
            .lock()
            .map_err(|e| e.to_string())?
            .launch_minimised;
        if !minimised {
            main.show().map_err(|e| e.to_string())?;
            main.set_focus().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn save_sound_file(
    file_name: String,
    file_data: Vec<u8>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let dest = ensure_data_subdir(&app, "sounds")?.join(&file_name);
    std::fs::write(&dest, &file_data)
        .map_err(|e| format!("Could not write sound file: {e}"))?;

    Ok(dest.to_string_lossy().to_string())
}
