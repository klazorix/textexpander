// src-tauri/src/db.rs
//
// SQLite persistence layer for Expandly v4.0.0
//
// All reads and writes go through this module.
// The engine continues to use Arc<Mutex<RootConfig>> — db.rs is only
// called from Tauri commands and startup/shutdown paths.

use std::{collections::HashMap, fs, path::PathBuf};

use rusqlite::{params, Connection};

use crate::models::{
    CustomVariable, Expansion, GlobalStats, Hotkey, RootConfig, Trigger,
};

// ── Schema ────────────────────────────────────────────────────────────────

pub fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS config (
            id                      INTEGER PRIMARY KEY CHECK (id = 1),
            version                 TEXT    NOT NULL DEFAULT '',
            enabled                 INTEGER NOT NULL DEFAULT 1,
            sound_enabled           INTEGER NOT NULL DEFAULT 0,
            sound_path              TEXT,
            launch_at_startup       INTEGER NOT NULL DEFAULT 0,
            launch_minimised        INTEGER NOT NULL DEFAULT 0,
            minimise_to_tray        INTEGER NOT NULL DEFAULT 1,
            theme                   TEXT    NOT NULL DEFAULT 'starry-blue',
            track_stats             INTEGER NOT NULL DEFAULT 1,
            expansion_delay_ms      INTEGER NOT NULL DEFAULT 250,
            buffer_size             INTEGER NOT NULL DEFAULT 32,
            hotkey_delay_ms         INTEGER NOT NULL DEFAULT 80,
            clear_buffer_on_switch  INTEGER NOT NULL DEFAULT 1,
            debug_enabled           INTEGER NOT NULL DEFAULT 0,
            debug_level             TEXT    NOT NULL DEFAULT 'errors'
        );

        CREATE TABLE IF NOT EXISTS snippets (
            id      TEXT PRIMARY KEY,
            name    TEXT NOT NULL,
            text    TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS triggers (
            id              TEXT PRIMARY KEY,
            key             TEXT NOT NULL,
            expansion_id    TEXT NOT NULL,
            word_boundary   INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS hotkeys (
            id              TEXT PRIMARY KEY,
            keys            TEXT NOT NULL,
            expansion_id    TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS variables (
            id      TEXT PRIMARY KEY,
            name    TEXT NOT NULL,
            value   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS stats_per_day (
            date    TEXT PRIMARY KEY,
            count   INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS stats_per_expansion (
            expansion_id    TEXT PRIMARY KEY,
            count           INTEGER NOT NULL DEFAULT 0
        );
    ")
}

// ── Schema migration for existing DBs ────────────────────────────────────

/// Adds any columns that didn't exist in older schema versions.
/// SQLite doesn't support IF NOT EXISTS on ALTER TABLE, so we attempt
/// each and ignore the error if the column already exists.
pub fn migrate_schema(conn: &Connection) {
    let _ = conn.execute("ALTER TABLE config ADD COLUMN debug_enabled INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE config ADD COLUMN debug_level TEXT NOT NULL DEFAULT 'errors'", []);
}

// ── Migration from config.json ────────────────────────────────────────────

/// Called once on startup. If the config table has no row, we either
/// migrate from config.json or write defaults.
pub fn migrate_if_needed(
    conn: &Connection,
    config_json_path: &PathBuf,
) -> rusqlite::Result<()> {
    let has_row: bool = conn
        .query_row("SELECT COUNT(*) FROM config", [], |r| r.get::<_, i64>(0))
        .map(|c| c > 0)
        .unwrap_or(false);

    if has_row {
        return Ok(());
    }

    // Try to load from config.json
    let root = if config_json_path.exists() {
        match fs::read_to_string(config_json_path) {
            Ok(raw) => match serde_json::from_str::<RootConfig>(&raw) {
                Ok(cfg) => {
                    println!("[expandly] Migrating config.json → expandly.db");
                    Some(cfg)
                }
                Err(e) => {
                    eprintln!("[expandly] config.json parse error during migration ({e}), using defaults");
                    None
                }
            },
            Err(e) => {
                eprintln!("[expandly] Could not read config.json during migration ({e}), using defaults");
                None
            }
        }
    } else {
        None
    };

    let cfg = root.unwrap_or_default();
    write_all(conn, &cfg)?;

    // Rename config.json → config.json.migrated
    if config_json_path.exists() {
        let migrated = config_json_path.with_extension("json.migrated");
        if let Err(e) = fs::rename(config_json_path, &migrated) {
            eprintln!("[expandly] Could not rename config.json after migration: {e}");
        } else {
            println!("[expandly] config.json renamed to config.json.migrated");
        }
    }

    Ok(())
}

// ── Full read ─────────────────────────────────────────────────────────────

/// Load the entire RootConfig from the database.
pub fn load_all(conn: &Connection) -> rusqlite::Result<RootConfig> {
    let cfg = load_config_row(conn)?;
    let expansions = load_snippets(conn)?;
    let triggers = load_triggers(conn)?;
    let hotkeys = load_hotkeys(conn)?;
    let custom_variables = load_variables(conn)?;
    let stats = load_stats(conn)?;

    Ok(RootConfig {
        version: cfg.version,
        enabled: cfg.enabled,
        sound_enabled: cfg.sound_enabled,
        sound_path: cfg.sound_path,
        launch_at_startup: cfg.launch_at_startup,
        launch_minimised: cfg.launch_minimised,
        minimise_to_tray: cfg.minimise_to_tray,
        theme: cfg.theme,
        track_stats: cfg.track_stats,
        expansion_delay_ms: cfg.expansion_delay_ms,
        buffer_size: cfg.buffer_size,
        hotkey_delay_ms: cfg.hotkey_delay_ms,
        clear_buffer_on_switch: cfg.clear_buffer_on_switch,
        debug_enabled: cfg.debug_enabled,
        debug_level: cfg.debug_level,
        expansions,
        triggers,
        hotkeys,
        custom_variables,
        stats,
    })
}

// ── Full write ────────────────────────────────────────────────────────────

/// Write the entire RootConfig to the database (used for migration and import).
pub fn write_all(conn: &Connection, cfg: &RootConfig) -> rusqlite::Result<()> {
    save_config_row(conn, cfg)?;

    // Clear and rewrite all domain tables
    conn.execute("DELETE FROM snippets", [])?;
    conn.execute("DELETE FROM triggers", [])?;
    conn.execute("DELETE FROM hotkeys", [])?;
    conn.execute("DELETE FROM variables", [])?;
    conn.execute("DELETE FROM stats_per_day", [])?;
    conn.execute("DELETE FROM stats_per_expansion", [])?;

    for exp in cfg.expansions.values() {
        insert_snippet(conn, exp)?;
    }
    for t in &cfg.triggers {
        insert_trigger(conn, t)?;
    }
    for h in &cfg.hotkeys {
        insert_hotkey(conn, h)?;
    }
    for v in &cfg.custom_variables {
        insert_variable(conn, v)?;
    }
    save_stats(conn, &cfg.stats)?;

    Ok(())
}

// ── Config row ────────────────────────────────────────────────────────────

struct ConfigRow {
    version:                String,
    enabled:                bool,
    sound_enabled:          bool,
    sound_path:             Option<String>,
    launch_at_startup:      bool,
    launch_minimised:       bool,
    minimise_to_tray:       bool,
    theme:                  String,
    track_stats:            bool,
    expansion_delay_ms:     u64,
    buffer_size:            usize,
    hotkey_delay_ms:        u64,
    clear_buffer_on_switch: bool,
    debug_enabled:          bool,
    debug_level:            String,
}

fn load_config_row(conn: &Connection) -> rusqlite::Result<ConfigRow> {
    conn.query_row(
        "SELECT version, enabled, sound_enabled, sound_path,
                launch_at_startup, launch_minimised, minimise_to_tray,
                theme, track_stats, expansion_delay_ms, buffer_size,
                hotkey_delay_ms, clear_buffer_on_switch,
                debug_enabled, debug_level
         FROM config WHERE id = 1",
        [],
        |r| Ok(ConfigRow {
            version:                r.get(0)?,
            enabled:                r.get::<_, i64>(1)? != 0,
            sound_enabled:          r.get::<_, i64>(2)? != 0,
            sound_path:             r.get(3)?,
            launch_at_startup:      r.get::<_, i64>(4)? != 0,
            launch_minimised:       r.get::<_, i64>(5)? != 0,
            minimise_to_tray:       r.get::<_, i64>(6)? != 0,
            theme:                  r.get(7)?,
            track_stats:            r.get::<_, i64>(8)? != 0,
            expansion_delay_ms:     r.get::<_, i64>(9)? as u64,
            buffer_size:            r.get::<_, i64>(10)? as usize,
            hotkey_delay_ms:        r.get::<_, i64>(11)? as u64,
            clear_buffer_on_switch: r.get::<_, i64>(12)? != 0,
            debug_enabled:          r.get::<_, i64>(13)? != 0,
            debug_level:            r.get(14)?,
        }),
    )
}

pub fn save_config_row(conn: &Connection, cfg: &RootConfig) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO config (
            id, version, enabled, sound_enabled, sound_path,
            launch_at_startup, launch_minimised, minimise_to_tray,
            theme, track_stats, expansion_delay_ms, buffer_size,
            hotkey_delay_ms, clear_buffer_on_switch,
            debug_enabled, debug_level
         ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
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
            cfg.version,
            cfg.enabled as i64,
            cfg.sound_enabled as i64,
            cfg.sound_path,
            cfg.launch_at_startup as i64,
            cfg.launch_minimised as i64,
            cfg.minimise_to_tray as i64,
            cfg.theme,
            cfg.track_stats as i64,
            cfg.expansion_delay_ms as i64,
            cfg.buffer_size as i64,
            cfg.hotkey_delay_ms as i64,
            cfg.clear_buffer_on_switch as i64,
            cfg.debug_enabled as i64,
            cfg.debug_level,
        ],
    )?;
    Ok(())
}

// ── Snippets ──────────────────────────────────────────────────────────────

fn load_snippets(conn: &Connection) -> rusqlite::Result<HashMap<String, Expansion>> {
    let mut stmt = conn.prepare("SELECT id, name, text FROM snippets")?;
    let rows = stmt.query_map([], |r| {
        let id: String = r.get(0)?;
        Ok(Expansion { id: id.clone(), name: r.get(1)?, text: r.get(2)? })
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let exp = row?;
        map.insert(exp.id.clone(), exp);
    }
    Ok(map)
}

fn insert_snippet(conn: &Connection, exp: &Expansion) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO snippets (id, name, text) VALUES (?1, ?2, ?3)",
        params![exp.id, exp.name, exp.text],
    )?;
    Ok(())
}

pub fn save_snippet(conn: &Connection, exp: &Expansion) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO snippets (id, name, text) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, text = excluded.text",
        params![exp.id, exp.name, exp.text],
    )?;
    Ok(())
}

pub fn delete_snippet(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])?;
    // Cascade: remove triggers and hotkeys that reference this snippet
    conn.execute("DELETE FROM triggers WHERE expansion_id = ?1", params![id])?;
    conn.execute("DELETE FROM hotkeys  WHERE expansion_id = ?1", params![id])?;
    Ok(())
}

// ── Triggers ──────────────────────────────────────────────────────────────

fn load_triggers(conn: &Connection) -> rusqlite::Result<Vec<Trigger>> {
    let mut stmt = conn.prepare("SELECT id, key, expansion_id, word_boundary FROM triggers")?;
    let rows = stmt.query_map([], |r| {
        Ok(Trigger {
            id:           r.get(0)?,
            key:          r.get(1)?,
            expansion_id: r.get(2)?,
            word_boundary: r.get::<_, i64>(3)? != 0,
        })
    })?;
    rows.collect()
}

fn insert_trigger(conn: &Connection, t: &Trigger) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO triggers (id, key, expansion_id, word_boundary) VALUES (?1, ?2, ?3, ?4)",
        params![t.id, t.key, t.expansion_id, t.word_boundary as i64],
    )?;
    Ok(())
}

pub fn save_trigger(conn: &Connection, t: &Trigger) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO triggers (id, key, expansion_id, word_boundary) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET
            key           = excluded.key,
            expansion_id  = excluded.expansion_id,
            word_boundary = excluded.word_boundary",
        params![t.id, t.key, t.expansion_id, t.word_boundary as i64],
    )?;
    Ok(())
}

pub fn delete_trigger(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM triggers WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Hotkeys ───────────────────────────────────────────────────────────────

fn load_hotkeys(conn: &Connection) -> rusqlite::Result<Vec<Hotkey>> {
    let mut stmt = conn.prepare("SELECT id, keys, expansion_id FROM hotkeys")?;
    let rows = stmt.query_map([], |r| {
        Ok(Hotkey { id: r.get(0)?, keys: r.get(1)?, expansion_id: r.get(2)? })
    })?;
    rows.collect()
}

fn insert_hotkey(conn: &Connection, h: &Hotkey) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO hotkeys (id, keys, expansion_id) VALUES (?1, ?2, ?3)",
        params![h.id, h.keys, h.expansion_id],
    )?;
    Ok(())
}

pub fn save_hotkey(conn: &Connection, h: &Hotkey) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO hotkeys (id, keys, expansion_id) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET
            keys         = excluded.keys,
            expansion_id = excluded.expansion_id",
        params![h.id, h.keys, h.expansion_id],
    )?;
    Ok(())
}

pub fn delete_hotkey(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM hotkeys WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Variables ─────────────────────────────────────────────────────────────

fn load_variables(conn: &Connection) -> rusqlite::Result<Vec<CustomVariable>> {
    let mut stmt = conn.prepare("SELECT id, name, value FROM variables")?;
    let rows = stmt.query_map([], |r| {
        Ok(CustomVariable { id: r.get(0)?, name: r.get(1)?, value: r.get(2)? })
    })?;
    rows.collect()
}

fn insert_variable(conn: &Connection, v: &CustomVariable) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO variables (id, name, value) VALUES (?1, ?2, ?3)",
        params![v.id, v.name, v.value],
    )?;
    Ok(())
}

pub fn save_variable(conn: &Connection, v: &CustomVariable) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO variables (id, name, value) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, value = excluded.value",
        params![v.id, v.name, v.value],
    )?;
    Ok(())
}

pub fn delete_variable(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM variables WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Stats ─────────────────────────────────────────────────────────────────

fn load_stats(conn: &Connection) -> rusqlite::Result<GlobalStats> {
    // Per-day counts
    let mut expansions_per_day: HashMap<String, u64> = HashMap::new();
    let mut stmt = conn.prepare("SELECT date, count FROM stats_per_day")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64)))?;
    for row in rows {
        let (date, count) = row?;
        expansions_per_day.insert(date, count);
    }

    // Per-expansion counts
    let mut expansion_counts: HashMap<String, u64> = HashMap::new();
    let mut stmt = conn.prepare("SELECT expansion_id, count FROM stats_per_expansion")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64)))?;
    for row in rows {
        let (id, count) = row?;
        expansion_counts.insert(id, count);
    }

    Ok(GlobalStats { expansions_per_day, expansion_counts })
}

pub fn save_stats(conn: &Connection, stats: &GlobalStats) -> rusqlite::Result<()> {
    for (date, count) in &stats.expansions_per_day {
        conn.execute(
            "INSERT INTO stats_per_day (date, count) VALUES (?1, ?2)
             ON CONFLICT(date) DO UPDATE SET count = excluded.count",
            params![date, *count as i64],
        )?;
    }
    for (expansion_id, count) in &stats.expansion_counts {
        conn.execute(
            "INSERT INTO stats_per_expansion (expansion_id, count) VALUES (?1, ?2)
             ON CONFLICT(expansion_id) DO UPDATE SET count = excluded.count",
            params![expansion_id, *count as i64],
        )?;
    }
    Ok(())
}

/// Increment a single expansion's stats atomically.
/// Used by the engine's fire-and-forget stats recording.
pub fn increment_stats(
    conn: &Connection,
    expansion_id: &str,
    date: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO stats_per_day (date, count) VALUES (?1, 1)
         ON CONFLICT(date) DO UPDATE SET count = count + 1",
        params![date],
    )?;
    conn.execute(
        "INSERT INTO stats_per_expansion (expansion_id, count) VALUES (?1, 1)
         ON CONFLICT(expansion_id) DO UPDATE SET count = count + 1",
        params![expansion_id],
    )?;
    Ok(())
}