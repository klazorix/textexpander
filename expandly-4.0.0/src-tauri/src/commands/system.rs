use tauri::Manager;

use crate::AppState;

#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
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
            .unwrap()
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
