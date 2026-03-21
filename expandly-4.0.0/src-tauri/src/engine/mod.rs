pub mod keys;
pub mod injection;
pub mod sound;
pub mod stats;
pub mod variables;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use rdev::{listen, Event, EventType, Key as RKey};

use crate::models::RootConfig;
use keys::{rkey_to_char, rkey_to_modifier_str, rkey_to_name};
use injection::{delete_chars, inject_text};
use sound::play_sound;
use stats::record_stats;
use variables::resolve_variables;

// Public re-export used by commands/stats.rs
pub use variables::days_from_epoch as days_from_epoch_pub;

// ── Snapshot ──────────────────────────────────────────────────────────────

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

// Builds a minimal RootConfig from a snapshot, used only for resolve_variables
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
    }
}

// ── Entry point ───────────────────────────────────────────────────────────

pub fn start(config: Arc<Mutex<RootConfig>>, config_file_path: PathBuf) {
    thread::spawn(move || {
        loop {
            let buffer:    Arc<Mutex<Vec<char>>>   = Arc::new(Mutex::new(Vec::new()));
            let held_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

            let buffer_clone = Arc::clone(&buffer);
            let held_clone   = Arc::clone(&held_keys);
            let config_clone = Arc::clone(&config);
            let path_clone   = config_file_path.clone();

            let restart_delay: u64 = 1000;

            let result = listen(move |event: Event| {
                match event.event_type {

                    // ── Key pressed ───────────────────────────────────────
                    EventType::KeyPress(key) => {

                        // Track modifiers — minimal lock scope
                        if let Some(modifier) = rkey_to_modifier_str(&key) {
                            let mut held = held_clone.lock().unwrap();
                            if !held.contains(&modifier.to_string()) {
                                held.push(modifier.to_string());
                            }
                            return;
                        }

                        // Snapshot config — single short lock, no work inside
                        let snap = {
                            let cfg = config_clone.lock().unwrap();
                            if !cfg.enabled { return; }
                            EngineSnapshot::from(&*cfg)
                        };

                        // ── Clear buffer on window switch (Alt+Tab / Super+Tab) ──
                        // Only triggers when Tab specifically is pressed, not on every keystroke while Alt/Super is held
                        if snap.clear_buffer_on_switch && matches!(key, RKey::Tab) {
                            let should_clear = {
                                let held = held_clone.lock().unwrap();
                                held.contains(&"Alt".to_string()) || held.contains(&"Super".to_string())
                            };
                            if should_clear {
                                buffer_clone.lock().unwrap().clear();
                                return;
                            }
                        }

                        // ── Hotkey check ──────────────────────────────────
                        {
                            let held = held_clone.lock().unwrap();
                            if !held.is_empty() {
                                if let Some(key_name) = rkey_to_name(&key) {
                                    let mut parts = held.clone();
                                    parts.push(key_name);
                                    let combo = parts.join("+");
                                    drop(held);

                                    for hotkey in &snap.hotkeys {
                                        if hotkey.keys.eq_ignore_ascii_case(&combo) {
                                            if let Some(expansion) = snap.expansions.get(&hotkey.expansion_id) {
                                                let text     = resolve_variables(&expansion.text, &snap_cfg_ref(&snap));
                                                let exp_id   = hotkey.expansion_id.clone();
                                                let cc       = Arc::clone(&config_clone);
                                                let pc       = path_clone.clone();
                                                let hk_delay = snap.hotkey_delay;
                                                thread::spawn(move || {
                                                    thread::sleep(Duration::from_millis(hk_delay));
                                                    inject_text(&text);
                                                    record_stats(&cc, &pc, &exp_id);
                                                });
                                                return;
                                            }
                                        }
                                    }
                                }
                                return;
                            }
                        }

                        // ── Buffer update ─────────────────────────────────
                        let matched = {
                            let mut buf = buffer_clone.lock().unwrap();

                            match key {
                                RKey::Backspace             => { buf.pop(); }
                                RKey::Return | RKey::Escape => { buf.clear(); }
                                _ => match rkey_to_char(&key) {
                                    Some(c) => {
                                        buf.push(c);
                                        while buf.len() > snap.buffer_size { buf.remove(0); }
                                    }
                                    None => { buf.clear(); }
                                },
                            }

                            // ── Trigger check ─────────────────────────────
                            let buf_str: String = buf.iter().collect();
                            let mut found: Option<(String, usize, String)> = None;

                            'outer: for trigger in &snap.triggers {
                                if !buf_str.to_lowercase().ends_with(&trigger.key.to_lowercase()) { continue; }
                                if trigger.word_boundary {
                                    let before = buf_str.len() - trigger.key.len();
                                    if before > 0 {
                                        let prev = buf_str.chars().nth(before - 1).unwrap_or(' ');
                                        if !prev.is_whitespace() { continue 'outer; }
                                    }
                                }
                                if let Some(expansion) = snap.expansions.get(&trigger.expansion_id) {
                                    let text   = resolve_variables(&expansion.text, &snap_cfg_ref(&snap));
                                    let del    = trigger.key.len();
                                    let exp_id = trigger.expansion_id.clone();
                                    found = Some((text, del, exp_id));
                                    break;
                                }
                            }

                            if found.is_some() { buf.clear(); }
                            found
                        };

                        // ── Expand on worker thread — never blocks the hook ─
                        if let Some((text, delete_count, expansion_id)) = matched {
                            let cc    = Arc::clone(&config_clone);
                            let pc    = path_clone.clone();
                            let delay = snap.expansion_delay;
                            let se    = snap.sound_enabled;
                            let sp    = snap.sound_path.clone();

                            thread::spawn(move || {
                                thread::sleep(Duration::from_millis(delay));
                                delete_chars(delete_count);
                                inject_text(&text);
                                record_stats(&cc, &pc, &expansion_id);
                                if se { if let Some(path) = sp { play_sound(path); } }
                            });
                        }
                    }

                    // ── Key released ──────────────────────────────────────
                    EventType::KeyRelease(key) => {
                        if let Some(modifier) = rkey_to_modifier_str(&key) {
                            let mut held = held_clone.lock().unwrap();
                            held.retain(|k| k != modifier);
                        }
                    }

                    _ => {}
                }
            });

            eprintln!("[engine] listener exited ({:?}), restarting in {}ms", result, restart_delay);
            thread::sleep(Duration::from_millis(restart_delay));
        }
    });
}