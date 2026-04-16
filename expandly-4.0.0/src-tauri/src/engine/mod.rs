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
use keys::{rkey_to_buffer_char, rkey_to_modifier_str, rkey_to_name};
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

#[derive(Default)]
struct PendingTrigger {
    trailing_spaces: usize,
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
    pending_trigger: Arc<Mutex<Option<PendingTrigger>>>,
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
        let extra_delete_count = pending_trigger
            .lock()
            .unwrap()
            .take()
            .map_or(0, |pending| pending.trailing_spaces);
        delete_chars(delete_count + extra_delete_count);
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
    let has_non_shift_modifier = held
        .iter()
        .any(|modifier| matches!(modifier.as_str(), "Control" | "Alt" | "Super"));
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
        return has_non_shift_modifier;
    }
    has_non_shift_modifier
}

fn track_pending_trigger_space(
    key: RKey,
    pending_trigger: &Arc<Mutex<Option<PendingTrigger>>>,
) -> bool {
    if !matches!(key, RKey::Space) {
        return false;
    }

    let mut pending = pending_trigger.lock().unwrap();
    if let Some(pending) = pending.as_mut() {
        pending.trailing_spaces += 1;
        return true;
    }

    false
}

fn update_buffer_and_match_trigger(
    key: RKey,
    snap: &EngineSnapshot,
    buffer: &Arc<Mutex<Vec<char>>>,
    shift_active: bool,
    caps_lock_on: bool,
    log: &SharedLogger,
) -> Option<(String, usize, String, String)> {
    let mut buf = buffer.lock().unwrap();

    match key {
        RKey::Backspace => {
            buf.pop();
        }
        RKey::Return
        | RKey::Tab
        | RKey::UpArrow
        | RKey::DownArrow
        | RKey::LeftArrow
        | RKey::RightArrow
        | RKey::Home
        | RKey::End
        | RKey::PageUp
        | RKey::PageDown
        | RKey::Delete => {
            buf.clear();
        }
        _ => match rkey_to_buffer_char(&key, shift_active, caps_lock_on) {
            Some(c) => {
                buf.push(c);
                while buf.len() > snap.buffer_size {
                    buf.remove(0);
                }
            }
            None => {}
        },
    }

    let buf_str: String = buf.iter().collect();
    let mut found = None;

    'outer: for trigger in &snap.triggers {
        let matches_trigger = if trigger.case_sensitive {
            buf_str.ends_with(&trigger.key)
        } else {
            buf_str.to_lowercase().ends_with(&trigger.key.to_lowercase())
        };
        if !matches_trigger {
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
            let caps_lock_on: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
            let pending_trigger: Arc<Mutex<Option<PendingTrigger>>> = Arc::new(Mutex::new(None));

            let buffer_clone = Arc::clone(&buffer);
            let held_clone   = Arc::clone(&held_keys);
            let caps_lock_clone = Arc::clone(&caps_lock_on);
            let pending_trigger_clone = Arc::clone(&pending_trigger);
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

                        if matches!(key, RKey::CapsLock) {
                            let mut caps_lock = caps_lock_clone.lock().unwrap();
                            *caps_lock = !*caps_lock;
                            return;
                        }

                        // Snapshot helpers
                        let snap = {
                            let cfg = config_clone.lock().unwrap();
                            if !cfg.enabled { return; }
                            EngineSnapshot::from(&*cfg)
                        };

                        if track_pending_trigger_space(key, &pending_trigger_clone) {
                            return;
                        }

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
                        let shift_active = {
                            let held = held_clone.lock().unwrap();
                            held.contains(&"Shift".to_string())
                        };
                        let caps_lock_on = *caps_lock_clone.lock().unwrap();
                        let matched = update_buffer_and_match_trigger(
                            key,
                            &snap,
                            &buffer_clone,
                            shift_active,
                            caps_lock_on,
                            &log_clone,
                        );

                        // Expand on a worker thread so the hook stays responsive
                        if let Some((text, delete_count, expansion_id, exp_name)) = matched {
                            *pending_trigger_clone.lock().unwrap() = Some(PendingTrigger::default());
                            spawn_trigger_expansion(
                                Arc::clone(&config_clone),
                                Arc::clone(&db_clone),
                                Arc::clone(&log_clone),
                                Arc::clone(&pending_trigger_clone),
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



