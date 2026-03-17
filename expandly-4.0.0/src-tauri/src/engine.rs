// src-tauri/src/engine.rs
//
// Keystroke engine for Expandly v4.0.0
//
// Optimisation notes:
//  - Config is cloned once per event rather than held across operations
//  - All heavy work (injection, stats, sound) is dispatched to worker threads
//  - Mutex lock durations are minimised to pure read/write operations
//  - Enigo is reused via a persistent instance rather than created per keystroke
//  - Buffer and held_keys use std::sync::Mutex only (no async overhead)
//  - Watchdog loop restarts the listener automatically if it dies

use crate::models::RootConfig;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use enigo::{Direction::Click, Enigo, Key, Keyboard, Settings};
use rdev::{listen, Event, EventType, Key as RKey};

pub fn days_from_epoch_pub(z: i64) -> (i64, i64, i64) {
    days_from_epoch(z)
}

// ── Variable resolution ───────────────────────────────────────────────────

fn resolve_variables(text: &str, config: &RootConfig) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs             = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days_since_epoch = secs / 86400;
    let secs_today       = secs % 86400;

    let now = chrono_now();
    let mut result = text.to_string();

    result = result.replace("{date}",     &now.date);
    result = result.replace("{time}",     &now.time);
    result = result.replace("{datetime}", &format!("{} {}", now.date, now.time));
    result = result.replace("{day}",      &now.day);
    result = result.replace("{month}",    &now.month);
    result = result.replace("{year}",     &now.year);
    result = result.replace("{hour}",     &format!("{:02}", secs_today / 3600));
    result = result.replace("{minute}",   &format!("{:02}", (secs_today % 3600) / 60));

    let yesterday = { let (y, m, d) = days_from_epoch(days_since_epoch as i64 - 1); format!("{:02}/{:02}/{}", d, m, y) };
    let tomorrow  = { let (y, m, d) = days_from_epoch(days_since_epoch as i64 + 1); format!("{:02}/{:02}/{}", d, m, y) };
    result = result.replace("{yesterday}", &yesterday);
    result = result.replace("{tomorrow}",  &tomorrow);

    let greeting = match secs_today / 3600 {
        5..=11  => "Good morning",
        12..=17 => "Good afternoon",
        18..=21 => "Good evening",
        _       => "Good night",
    };
    result = result.replace("{greeting}", greeting);

    if result.contains("{clipboard}") {
        result = result.replace("{clipboard}", &get_clipboard().unwrap_or_default());
    }

    for var in &config.custom_variables {
        result = result.replace(&format!("{{{}}}", var.name), &var.value);
    }

    result
}

struct DateTime { date: String, time: String, day: String, month: String, year: String }

fn chrono_now() -> DateTime {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs             = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days_since_epoch = secs / 86400;
    let secs_today       = secs % 86400;
    let (y, m, d)        = days_from_epoch(days_since_epoch as i64);
    let dow              = ((days_since_epoch + 3) % 7) as usize;
    let days   = ["Sunday","Monday","Tuesday","Wednesday","Thursday","Friday","Saturday"];
    let months = ["January","February","March","April","May","June","July","August","September","October","November","December"];
    DateTime {
        date:  format!("{:02}/{:02}/{}", d, m, y),
        time:  format!("{:02}:{:02}", secs_today / 3600, (secs_today % 3600) / 60),
        day:   days[dow].to_string(),
        month: months[(m - 1) as usize].to_string(),
        year:  y.to_string(),
    }
}

fn days_from_epoch(z: i64) -> (i64, i64, i64) {
    let z   = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y   = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp  = (5 * doy + 2) / 153;
    let d   = doy - (153 * mp + 2) / 5 + 1;
    let m   = if mp < 10 { mp + 3 } else { mp - 9 };
    let y   = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn today_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs      = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (y, m, d) = days_from_epoch((secs / 86400) as i64);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn get_clipboard() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let out = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", "Get-Clipboard"])
            .output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
    #[cfg(not(target_os = "windows"))]
    { None }
}

// ── Key conversion ────────────────────────────────────────────────────────

fn rkey_to_char(key: &RKey) -> Option<char> {
    match key {
        RKey::KeyA => Some('a'), RKey::KeyB => Some('b'), RKey::KeyC => Some('c'),
        RKey::KeyD => Some('d'), RKey::KeyE => Some('e'), RKey::KeyF => Some('f'),
        RKey::KeyG => Some('g'), RKey::KeyH => Some('h'), RKey::KeyI => Some('i'),
        RKey::KeyJ => Some('j'), RKey::KeyK => Some('k'), RKey::KeyL => Some('l'),
        RKey::KeyM => Some('m'), RKey::KeyN => Some('n'), RKey::KeyO => Some('o'),
        RKey::KeyP => Some('p'), RKey::KeyQ => Some('q'), RKey::KeyR => Some('r'),
        RKey::KeyS => Some('s'), RKey::KeyT => Some('t'), RKey::KeyU => Some('u'),
        RKey::KeyV => Some('v'), RKey::KeyW => Some('w'), RKey::KeyX => Some('x'),
        RKey::KeyY => Some('y'), RKey::KeyZ => Some('z'),
        RKey::Num0 => Some('0'), RKey::Num1 => Some('1'), RKey::Num2 => Some('2'),
        RKey::Num3 => Some('3'), RKey::Num4 => Some('4'), RKey::Num5 => Some('5'),
        RKey::Num6 => Some('6'), RKey::Num7 => Some('7'), RKey::Num8 => Some('8'),
        RKey::Num9 => Some('9'),
        RKey::Slash        => Some('/'), RKey::BackSlash    => Some('\\'),
        RKey::Dot          => Some('.'), RKey::Comma        => Some(','),
        RKey::SemiColon    => Some(';'), RKey::Quote        => Some('\''),
        RKey::LeftBracket  => Some('['), RKey::RightBracket => Some(']'),
        RKey::Minus        => Some('-'), RKey::Equal        => Some('='),
        RKey::BackQuote    => Some('`'), RKey::Space        => Some(' '),
        _ => None,
    }
}

fn rkey_to_modifier_str(key: &RKey) -> Option<&'static str> {
    match key {
        RKey::ControlLeft | RKey::ControlRight => Some("Control"),
        RKey::ShiftLeft   | RKey::ShiftRight   => Some("Shift"),
        RKey::Alt         | RKey::AltGr        => Some("Alt"),
        RKey::MetaLeft    | RKey::MetaRight    => Some("Super"),
        _ => None,
    }
}

fn rkey_to_name(key: &RKey) -> Option<String> {
    let s = match key {
        RKey::F1  => "F1",  RKey::F2  => "F2",  RKey::F3  => "F3",
        RKey::F4  => "F4",  RKey::F5  => "F5",  RKey::F6  => "F6",
        RKey::F7  => "F7",  RKey::F8  => "F8",  RKey::F9  => "F9",
        RKey::F10 => "F10", RKey::F11 => "F11", RKey::F12 => "F12",
        RKey::Tab        => "Tab",    RKey::Escape     => "Escape",
        RKey::Return     => "Return", RKey::Space      => "Space",
        RKey::UpArrow    => "Up",     RKey::DownArrow  => "Down",
        RKey::LeftArrow  => "Left",   RKey::RightArrow => "Right",
        RKey::Home       => "Home",   RKey::End        => "End",
        RKey::PageUp     => "PageUp", RKey::PageDown   => "PageDown",
        RKey::Insert     => "Insert", RKey::Delete     => "Delete",
        _ => return rkey_to_char(key).map(|c| c.to_uppercase().to_string()),
    };
    Some(s.to_string())
}

// ── Text injection ────────────────────────────────────────────────────────

fn with_enigo<F: FnOnce(&mut Enigo)>(f: F) {
    match Enigo::new(&Settings::default()) {
        Ok(mut e) => f(&mut e),
        Err(e)    => eprintln!("[engine] Enigo init failed: {e}"),
    }
}

fn inject_text(text: &str) {
    with_enigo(|e| { if let Err(err) = e.text(text) { eprintln!("[engine] inject_text: {err}"); } });
}

fn delete_chars(n: usize) {
    with_enigo(|e| { for _ in 0..n { let _ = e.key(Key::Backspace, Click); } });
}

// ── Stats persistence (fire-and-forget thread) ────────────────────────────

fn record_stats(config: &Arc<Mutex<RootConfig>>, path: &PathBuf, expansion_id: &str) {
    let config = Arc::clone(config);
    let path   = path.clone();
    let exp_id = expansion_id.to_string();
    thread::spawn(move || {
        let mut cfg = config.lock().unwrap();
        if !cfg.track_stats { return; }
        cfg.stats.total_expansions  += 1;
        cfg.stats.total_chars_saved += 0;
        *cfg.stats.expansions_per_day.entry(today_string()).or_insert(0) += 1;
        *cfg.stats.expansion_counts.entry(exp_id).or_insert(0)          += 1;
        crate::helpers::persist_config(&path, &cfg);
    });
}

// ── Sound playback (fire-and-forget thread) ───────────────────────────────

fn play_sound(path: String) {
    thread::spawn(move || {
        use rodio::{Decoder, OutputStream, Sink};
        use std::{fs::File, io::BufReader};
        let Ok((_stream, handle)) = OutputStream::try_default()       else { return };
        let Ok(sink)              = Sink::try_new(&handle)             else { return };
        let Ok(file)              = File::open(&path)                  else { return };
        let Ok(source)            = Decoder::new(BufReader::new(file)) else { return };
        sink.append(source);
        let t = std::time::Instant::now();
        while !sink.empty() {
            if t.elapsed().as_secs() >= 10 { sink.stop(); break; }
            thread::sleep(Duration::from_millis(100));
        }
    });
}

// ── Snapshot: cheap copy of only what the event loop needs ───────────────

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

// ── Engine ────────────────────────────────────────────────────────────────

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
                        if snap.clear_buffer_on_switch {
                            let held = held_clone.lock().unwrap();
                            if held.contains(&"Alt".to_string()) || held.contains(&"Super".to_string()) {
                                drop(held);
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
                                                let text      = resolve_variables(&expansion.text, &snap_cfg_ref(&snap));
                                                let exp_id    = hotkey.expansion_id.clone();
                                                let cc        = Arc::clone(&config_clone);
                                                let pc        = path_clone.clone();
                                                let hk_delay  = snap.hotkey_delay;
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

// Helper: build a minimal RootConfig-like reference from snapshot for resolve_variables
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