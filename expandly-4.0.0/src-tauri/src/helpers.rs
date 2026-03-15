use std::{fs, path::PathBuf};
use crate::models::RootConfig;

pub fn persist_config(path: &PathBuf, config: &RootConfig) {
    let json = match serde_json::to_string_pretty(config) {
        Ok(j) => j,
        Err(e) => { eprintln!("[expandly] Failed to serialise config: {e}"); return; }
    };
    let tmp_path = path.with_extension("json.tmp");
    if let Err(e) = fs::write(&tmp_path, &json) {
        eprintln!("[expandly] Failed to write temp config: {e}"); return;
    }
    if let Err(e) = fs::rename(&tmp_path, path) {
        eprintln!("[expandly] Failed to replace config.json: {e}");
    }
}