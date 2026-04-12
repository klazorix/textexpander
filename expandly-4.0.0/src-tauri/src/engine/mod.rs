pub mod keys;
pub mod injection;
pub mod sound;
pub mod stats;
pub mod variables;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use rdev::{listen, Event, EventType, Key as RKey};

use crate::models::RootConfig;
use crate::logger::SharedLogger;
use keys::{rkey_to_char, rkey_to_modifier_str, rkey_to_name};
use injection::{delete_chars, inject_text};
use sound::play_sound;
use stats::record_stats;
use variables::resolve_variables;

const RESTART_DELAY_MS: u64 = 1000;

// Snapshot helpers

struct EngineSnapshot {
    enabled:                bool,
    buffer_size:            usize,
    expansion_delay:        u64,
    hotkey_delay:           u64,
    clear_buffer_on_switch: bool,
    sound_enabled:          bool,
    sound_path:             Option<String>,
    triggers:               Vec<crate::models::Trigger>,
    hotkeys:                Vec<crate::models::Hotkey>,
    expansions:             std::collections::HashMap<String, crate::models::Expansion>,
    custom_variables:       Vec<crate::models::CustomVariable>,
}

impl EngineSnapshot {
    fn from(cfg: &RootConfig) -> Self {
        Self {
            enabled:                cfg.enabled,
            buffer_size:            cfg.buffer_size,
            expansion_delay:        cfg.expansion_delay_ms,
            hotkey_delay:           cfg.hotkey_delay_ms,
            clear_buffer_on_switch: cfg.clear_buffer_on_switch,
            sound_enabled:          cfg.sound_enabled,
            sound_path:             cfg.sound_path.clone(),
            triggers:               cfg.triggers.clone(),
            hotkeys:                cfg.hotkeys.clone(),
            expansions:             cfg.expansions.clone(),
            custom_variables:       cfg.custom_variables.clone(),
        }
    }
}

// Build a minimal config snapshot for variable resolution.
fn snap_cfg_ref(snap: &EngineSnapshot) -> RootConfig {
    RootConfig {
        version:                String::new(),
        enabled:                snap.enabled,
        sound_enabled:          snap.sound_enabled,
        sound_path:             snap.sound_path.clone(),
        launch_at_startup:      false,
        launch_minimised:       false,
        minimise_to_tray:       false,
        theme:                  String::new(),
        track_stats:            false,
        expansion_delay_ms:     snap.expansion_delay,
        buffer_size:            snap.buffer_size,
        hotkey_delay_ms:        snap.hotkey_delay,
        clear_buffer_on_switch: snap.clear_buffer_on_switch,
        expansions:             snap.expansions.clone(),
        triggers:               snap.triggers.clone(),
        hotkeys:                snap.hotkeys.clone(),
        custom_variables:       snap.custom_variables.clone(),
        stats:                  crate::models::GlobalStats::default(),
        debug_enabled:          false,
        debug_level:            String::new(),
    }
}

fn spawn_hotkey_expansion(
    config: Arc<Mutex<RootConfig>>,
    db: Arc<Mutex<rusqlite::Connection>>,
    log: SharedLogger,
    combo: String,
    expansion_id: String,
    expansion_name: String,
    text: String,
    delay_ms: u64,
) {
    thread::spawn(move || {
        log.lock().unwrap().verbose(&format!("[engine] Hotkey fired: {expansion_name} ({combo})"));
        thread::sleep(Duration::from_millis(delay_ms));
        inject_text(&text);
        record_stats(&config, &db, &expansion_id);
    });
}

fn spawn_trigger_expansion(
    config: Arc<Mutex<RootConfig>>,
    db: Arc<Mutex<rusqlite::Connection>>,
    log: SharedLogger,
    expansion_id: String,
    expansion_name: String,
    text: String,
    delete_count: usize,
    delay_ms: u64,
    sound_enabled: bool,
    sound_path: Option<String>,
) {
    thread::spawn(move || {
        log.lock().unwrap().verbose(&format!("[engine] Trigger fired: {expansion_name}"));
        thread::sleep(Duration::from_millis(delay_ms));
        delete_chars(delete_count);
        inject_text(&text);
        record_stats(&config, &db, &expansion_id);
        if sound_enabled {
            if let Some(path) = sound_path {
                play_sound(path);
            }
        }
    });
}

fn handle_hotkey(
    key: RKey,
    snap: &EngineSnapshot,
    held_keys: &Arc<Mutex<Vec<String>>>,
    config: &Arc<Mutex<RootConfig>>,
    db: &Arc<Mutex<rusqlite::Connection>>,
    log: &SharedLogger,
) -> bool {
    let held = held_keys.lock().unwrap();
    if held.is_empty() {
        return false;
    }
    if let Some(key_name) = rkey_to_name(&key) {
        let mut parts = held.clone();
        parts.push(key_name);
        let combo = parts.join("+");
        drop(held);

        log.lock().unwrap().verbose(&format!("[engine] Hotkey check: combo={combo}"));

        for hotkey in &snap.hotkeys {
            if hotkey.keys.eq_ignore_ascii_case(&combo) {
                if let Some(expansion) = snap.expansions.get(&hotkey.expansion_id) {
                    let text = resolve_variables(&expansion.text, &snap_cfg_ref(snap));
                    spawn_hotkey_expansion(
                        Arc::clone(config),
                        Arc::clone(db),
                        Arc::clone(log),
                        combo,
                        hotkey.expansion_id.clone(),
                        expansion.name.clone(),
                        text,
                        snap.hotkey_delay,
                    );
                    return true;
                }

                log.lock().unwrap().warning(&format!(
                    "[engine] Hotkey matched combo={combo} but expansion_id={} not found",
                    hotkey.expansion_id
                ));
            }
        }
    }
    true
}

fn update_buffer_and_match_trigger(
    key: RKey,
    snap: &EngineSnapshot,
    buffer: &Arc<Mutex<Vec<char>>>,
    log: &SharedLogger,
) -> Option<(String, usize, String, String)> {
    let mut buf = buffer.lock().unwrap();

    match key {
        RKey::Backspace => {
            buf.pop();
        }
        RKey::Return | RKey::Escape => {
            buf.clear();
        }
        _ => match rkey_to_char(&key) {
            Some(c) => {
                buf.push(c);
                while buf.len() > snap.buffer_size {
                    buf.remove(0);
                }
            }
            None => {
                buf.clear();
            }
        },
    }

    let buf_str: String = buf.iter().collect();
    let mut found = None;

    'outer: for trigger in &snap.triggers {
        if !buf_str.to_lowercase().ends_with(&trigger.key.to_lowercase()) {
            continue;
        }
        if trigger.word_boundary {
            let before = buf_str.len() - trigger.key.len();
            if before > 0 {
                let prev = buf_str.chars().nth(before - 1).unwrap_or(' ');
                if !prev.is_whitespace() {
                    continue 'outer;
                }
            }
        }
        if let Some(expansion) = snap.expansions.get(&trigger.expansion_id) {
            found = Some((
                resolve_variables(&expansion.text, &snap_cfg_ref(snap)),
                trigger.key.len(),
                trigger.expansion_id.clone(),
                expansion.name.clone(),
            ));
            break;
        }

        log.lock().unwrap().warning(&format!(
            "[engine] Trigger '{}' matched but expansion_id={} not found",
            trigger.key, trigger.expansion_id
        ));
    }

    if found.is_some() {
        buf.clear();
    }
    found
}

// Engine entry point

pub fn start(
    config: Arc<Mutex<RootConfig>>,
    db: Arc<Mutex<rusqlite::Connection>>,
    log: SharedLogger,
) {
    thread::spawn(move || {
        loop {
            let buffer:    Arc<Mutex<Vec<char>>>   = Arc::new(Mutex::new(Vec::new()));
            let held_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

            let buffer_clone = Arc::clone(&buffer);
            let held_clone   = Arc::clone(&held_keys);
            let config_clone = Arc::clone(&config);
            let db_clone     = Arc::clone(&db);
            let log_clone    = Arc::clone(&log);

            log.lock().unwrap().verbose("[engine] Listener starting");

            let result = listen(move |event: Event| {
                match event.event_type {

                    // Key press handling
                    EventType::KeyPress(key) => {

                        // Track held modifiers with minimal lock scope
                        if let Some(modifier) = rkey_to_modifier_str(&key) {
                            let mut held = held_clone.lock().unwrap();
                            if !held.contains(&modifier.to_string()) {
                                held.push(modifier.to_string());
                            }
                            return;
                        }

                        // Snapshot helpers
                        let snap = {
                            let cfg = config_clone.lock().unwrap();
                            if !cfg.enabled { return; }
                            EngineSnapshot::from(&*cfg)
                        };

                        // Clear the buffer on Alt+Tab or Super+Tab when enabled
                        if snap.clear_buffer_on_switch && matches!(key, RKey::Tab) {
                            let should_clear = {
                                let held = held_clone.lock().unwrap();
                                held.contains(&"Alt".to_string()) || held.contains(&"Super".to_string())
                            };
                            if should_clear {
                                buffer_clone.lock().unwrap().clear();
                                log_clone.lock().unwrap().verbose("[engine] Buffer cleared on window switch");
                                return;
                            }
                        }

                        // Hotkey check
                        if handle_hotkey(
                            key,
                            &snap,
                            &held_clone,
                            &config_clone,
                            &db_clone,
                            &log_clone,
                        ) {
                            return;
                        }

                        // Buffer update and trigger match
                        let matched = update_buffer_and_match_trigger(
                            key,
                            &snap,
                            &buffer_clone,
                            &log_clone,
                        );

                        // Expand on a worker thread so the hook stays responsive
                        if let Some((text, delete_count, expansion_id, exp_name)) = matched {
                            spawn_trigger_expansion(
                                Arc::clone(&config_clone),
                                Arc::clone(&db_clone),
                                Arc::clone(&log_clone),
                                expansion_id,
                                exp_name,
                                text,
                                delete_count,
                                snap.expansion_delay,
                                snap.sound_enabled,
                                snap.sound_path.clone(),
                            );
                        }
                    }

                    // Key release handling
                    EventType::KeyRelease(key) => {
                        if let Some(modifier) = rkey_to_modifier_str(&key) {
                            let mut held = held_clone.lock().unwrap();
                            held.retain(|k| k != modifier);
                        }
                    }

                    _ => {}
                }
            });

            log.lock().unwrap().error(&format!("[engine] Listener exited ({result:?}), restarting in {RESTART_DELAY_MS}ms"));
            eprintln!("[engine] listener exited ({:?}), restarting in {}ms", result, RESTART_DELAY_MS);
            thread::sleep(Duration::from_millis(RESTART_DELAY_MS));
        }
    });
}



