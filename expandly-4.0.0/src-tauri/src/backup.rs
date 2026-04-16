// src-tauri/src/backup.rs
//
// Export and import logic for Expandly v4.0.0
//
// Converts between the SQLite DB and a single merged JSON file.
// The JSON format maps 1:1 to the DB tables and is independent of
// RootConfig struct layout, making it forward-compatible across versions.
//
// Export format:
// {
//   "version":   "4.0.0",
//   "config":    { ...all settings fields... },
//   "snippets":  [ { id, name, text }, ... ],
//   "triggers":  [ { id, key, expansion_id, word_boundary, case_sensitive }, ... ],
//   "hotkeys":   [ { id, keys, expansion_id }, ... ],
//   "variables": [ { id, name, value }, ... ],
//   "stats": {
//     "per_day":       { "YYYY-MM-DD": count, ... },
//     "per_expansion": { "uuid": count, ... }
//   }
// }

use rusqlite::{params, Connection};

fn json_sql_error(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}

fn collect_json_rows<F>(conn: &Connection, sql: &str, map: F) -> rusqlite::Result<Vec<serde_json::Value>>
where
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<serde_json::Value>,
{
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], map)?.collect();
    rows
}

fn collect_json_map<F>(conn: &Connection, sql: &str, mut map: F) -> rusqlite::Result<serde_json::Map<String, serde_json::Value>>
where
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<(String, serde_json::Value)>,
{
    let mut values = serde_json::Map::new();
    let mut stmt = conn.prepare(sql)?;
    for row in stmt.query_map([], |row| map(row))? {
        let (key, value) = row?;
        values.insert(key, value);
    }
    Ok(values)
}

fn write_json_array<F>(
    root: &serde_json::Value,
    key: &str,
    mut write: F,
) -> Result<(), String>
where
    F: FnMut(&serde_json::Value) -> Result<(), String>,
{
    if let Some(items) = root.get(key).and_then(|value| value.as_array()) {
        for item in items {
            write(item)?;
        }
    }
    Ok(())
}

/// Build a merged JSON export from all DB tables.
pub fn export_to_json(conn: &Connection) -> rusqlite::Result<String> {
    use serde_json::{json, Map, Value};

    // Config row — reuse db's load helper via a direct query
    let cfg = crate::db::load_config_for_export(conn)?;

    let config_obj = json!({
        "version":                cfg.version,
        "enabled":                cfg.enabled,
        "sound_enabled":          cfg.sound_enabled,
        "sound_path":             cfg.sound_path,
        "launch_at_startup":      cfg.launch_at_startup,
        "launch_minimised":       cfg.launch_minimised,
        "minimise_to_tray":       cfg.minimise_to_tray,
        "theme":                  cfg.theme,
        "track_stats":            cfg.track_stats,
        "expansion_delay_ms":     cfg.expansion_delay_ms,
        "buffer_size":            cfg.buffer_size,
        "hotkey_delay_ms":        cfg.hotkey_delay_ms,
        "clear_buffer_on_switch": cfg.clear_buffer_on_switch,
        "debug_enabled":          cfg.debug_enabled,
        "debug_level":            cfg.debug_level,
    });

    // Snippets
    let snippets = collect_json_rows(conn, "SELECT id, name, text FROM snippets", |r| Ok(json!({
            "id":   r.get::<_, String>(0)?,
            "name": r.get::<_, String>(1)?,
            "text": r.get::<_, String>(2)?,
        })))?;

    // Triggers
    let triggers = collect_json_rows(conn, "SELECT id, key, expansion_id, word_boundary, case_sensitive FROM triggers", |r| Ok(json!({
            "id":            r.get::<_, String>(0)?,
            "key":           r.get::<_, String>(1)?,
            "expansion_id":  r.get::<_, String>(2)?,
            "word_boundary": r.get::<_, i64>(3)? != 0,
            "case_sensitive": r.get::<_, i64>(4)? != 0,
        })))?;

    // Hotkeys
    let hotkeys = collect_json_rows(conn, "SELECT id, keys, expansion_id FROM hotkeys", |r| Ok(json!({
            "id":           r.get::<_, String>(0)?,
            "keys":         r.get::<_, String>(1)?,
            "expansion_id": r.get::<_, String>(2)?,
        })))?;

    // Variables
    let variables = collect_json_rows(conn, "SELECT id, name, value FROM variables", |r| Ok(json!({
            "id":    r.get::<_, String>(0)?,
            "name":  r.get::<_, String>(1)?,
            "value": r.get::<_, String>(2)?,
        })))?;

    // Stats
    let per_day: Map<String, Value> = collect_json_map(conn, "SELECT date, count FROM stats_per_day", |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?.into()))
    })?;
    let per_expansion: Map<String, Value> = collect_json_map(conn, "SELECT expansion_id, count FROM stats_per_expansion", |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?.into()))
    })?;

    let export = json!({
        "version":   cfg.version,
        "config":    config_obj,
        "snippets":  snippets,
        "triggers":  triggers,
        "hotkeys":   hotkeys,
        "variables": variables,
        "stats": {
            "per_day":       per_day,
            "per_expansion": per_expansion,
        }
    });

    serde_json::to_string_pretty(&export).map_err(json_sql_error)
}

/// Import a merged JSON backup into the DB.
/// Clears all domain tables first, then repopulates from the JSON.
/// Unknown keys are silently ignored for forward compatibility.
pub fn import_from_json(conn: &Connection, json: &str) -> Result<(), String> {
    use serde_json::Value;

    let root: Value = serde_json::from_str(json)
        .map_err(|e| format!("Invalid JSON: {e}"))?;

    // ── Config ────────────────────────────────────────────────────────────
    if let Some(cfg) = root.get("config").and_then(|v| v.as_object()) {
        let get_str   = |k: &str, d: &str| cfg.get(k).and_then(|v| v.as_str()).unwrap_or(d).to_string();
        let get_bool  = |k: &str, d: bool| cfg.get(k).and_then(|v| v.as_bool()).unwrap_or(d);
        let get_u64   = |k: &str, d: u64|  cfg.get(k).and_then(|v| v.as_u64()).unwrap_or(d);
        let get_usize = |k: &str, d: usize| cfg.get(k).and_then(|v| v.as_u64()).unwrap_or(d as u64) as usize;

        conn.execute(
            "INSERT INTO config (
                id, version, enabled, sound_enabled, sound_path,
                launch_at_startup, launch_minimised, minimise_to_tray,
                theme, track_stats, expansion_delay_ms, buffer_size,
                hotkey_delay_ms, clear_buffer_on_switch,
                debug_enabled, debug_level
             ) VALUES (1,?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)
             ON CONFLICT(id) DO UPDATE SET
                version                = excluded.version,
                enabled                = excluded.enabled,
                sound_enabled          = excluded.sound_enabled,
                sound_path             = excluded.sound_path,
                launch_at_startup      = excluded.launch_at_startup,
                launch_minimised       = excluded.launch_minimised,
                minimise_to_tray       = excluded.minimise_to_tray,
                theme                  = excluded.theme,
                track_stats            = excluded.track_stats,
                expansion_delay_ms     = excluded.expansion_delay_ms,
                buffer_size            = excluded.buffer_size,
                hotkey_delay_ms        = excluded.hotkey_delay_ms,
                clear_buffer_on_switch = excluded.clear_buffer_on_switch,
                debug_enabled          = excluded.debug_enabled,
                debug_level            = excluded.debug_level",
            params![
                get_str("version", ""),
                get_bool("enabled", true) as i64,
                get_bool("sound_enabled", false) as i64,
                cfg.get("sound_path").and_then(|v| v.as_str()).map(|s| s.to_string()),
                get_bool("launch_at_startup", false) as i64,
                get_bool("launch_minimised", false) as i64,
                get_bool("minimise_to_tray", true) as i64,
                get_str("theme", "starry-blue"),
                get_bool("track_stats", true) as i64,
                get_u64("expansion_delay_ms", 250) as i64,
                get_usize("buffer_size", 32) as i64,
                get_u64("hotkey_delay_ms", 80) as i64,
                get_bool("clear_buffer_on_switch", true) as i64,
                get_bool("debug_enabled", false) as i64,
                get_str("debug_level", "errors"),
            ],
        ).map_err(|e| format!("Failed to write config: {e}"))?;
    }

    // ── Clear domain tables ───────────────────────────────────────────────
    conn.execute("DELETE FROM snippets",            []).map_err(|e| format!("{e}"))?;
    conn.execute("DELETE FROM triggers",            []).map_err(|e| format!("{e}"))?;
    conn.execute("DELETE FROM hotkeys",             []).map_err(|e| format!("{e}"))?;
    conn.execute("DELETE FROM variables",           []).map_err(|e| format!("{e}"))?;
    conn.execute("DELETE FROM stats_per_day",       []).map_err(|e| format!("{e}"))?;
    conn.execute("DELETE FROM stats_per_expansion", []).map_err(|e| format!("{e}"))?;

    // ── Snippets ──────────────────────────────────────────────────────────
    write_json_array(&root, "snippets", |snippet| {
        let id   = snippet.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let name = snippet.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        let text = snippet.get("text").and_then(|v| v.as_str()).unwrap_or_default();
        if id.is_empty() { return Ok(()); }
        conn.execute(
            "INSERT OR REPLACE INTO snippets (id, name, text) VALUES (?1, ?2, ?3)",
            params![id, name, text],
        ).map_err(|e| format!("Failed to write snippet: {e}"))?;
        Ok(())
    })?;

    // ── Triggers ──────────────────────────────────────────────────────────
    write_json_array(&root, "triggers", |trigger| {
        let id = trigger.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let key = trigger.get("key").and_then(|v| v.as_str()).unwrap_or_default();
        let expansion_id = trigger.get("expansion_id").and_then(|v| v.as_str()).unwrap_or_default();
        let word_boundary = trigger.get("word_boundary").and_then(|v| v.as_bool()).unwrap_or(true);
        let case_sensitive = trigger.get("case_sensitive").and_then(|v| v.as_bool()).unwrap_or(false);
        if id.is_empty() { return Ok(()); }
        conn.execute(
            "INSERT OR REPLACE INTO triggers (id, key, expansion_id, word_boundary, case_sensitive) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, key, expansion_id, word_boundary as i64, case_sensitive as i64],
        ).map_err(|e| format!("Failed to write trigger: {e}"))?;
        Ok(())
    })?;

    // ── Hotkeys ───────────────────────────────────────────────────────────
    write_json_array(&root, "hotkeys", |hotkey| {
        let id = hotkey.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let keys = hotkey.get("keys").and_then(|v| v.as_str()).unwrap_or_default();
        let expansion_id = hotkey.get("expansion_id").and_then(|v| v.as_str()).unwrap_or_default();
        if id.is_empty() { return Ok(()); }
        conn.execute(
            "INSERT OR REPLACE INTO hotkeys (id, keys, expansion_id) VALUES (?1, ?2, ?3)",
            params![id, keys, expansion_id],
        ).map_err(|e| format!("Failed to write hotkey: {e}"))?;
        Ok(())
    })?;

    // ── Variables ─────────────────────────────────────────────────────────
    write_json_array(&root, "variables", |variable| {
        let id = variable.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let name = variable.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        let value = variable.get("value").and_then(|v| v.as_str()).unwrap_or_default();
        if id.is_empty() { return Ok(()); }
        conn.execute(
            "INSERT OR REPLACE INTO variables (id, name, value) VALUES (?1, ?2, ?3)",
            params![id, name, value],
        ).map_err(|e| format!("Failed to write variable: {e}"))?;
        Ok(())
    })?;

    // ── Stats ─────────────────────────────────────────────────────────────
    if let Some(stats) = root.get("stats").and_then(|v| v.as_object()) {
        if let Some(per_day) = stats.get("per_day").and_then(|v| v.as_object()) {
            for (date, count) in per_day {
                let count = count.as_i64().unwrap_or(0);
                conn.execute(
                    "INSERT OR REPLACE INTO stats_per_day (date, count) VALUES (?1, ?2)",
                    params![date, count],
                ).map_err(|e| format!("Failed to write stats_per_day: {e}"))?;
            }
        }
        if let Some(per_exp) = stats.get("per_expansion").and_then(|v| v.as_object()) {
            for (expansion_id, count) in per_exp {
                let count = count.as_i64().unwrap_or(0);
                conn.execute(
                    "INSERT OR REPLACE INTO stats_per_expansion (expansion_id, count) VALUES (?1, ?2)",
                    params![expansion_id, count],
                ).map_err(|e| format!("Failed to write stats_per_expansion: {e}"))?;
            }
        }
    }

    Ok(())
}
