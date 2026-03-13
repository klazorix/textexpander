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
        .setup(|app| {
            let config = load_or_create_config(&app.handle());
            let config = Arc::new(Mutex::new(config));

            // Start the keystroke engine on a background thread
            engine::start(Arc::clone(&config));

            app.manage(AppState { config });
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Text Expander");
}