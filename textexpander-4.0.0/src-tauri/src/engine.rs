// src-tauri/src/engine.rs
//
// Keystroke engine for expandly v4.0.0
//
// Responsibilities:
//  - Listen to all keystrokes system-wide via rdev
//  - Maintain a rolling buffer of recently typed characters
//  - When buffer tail matches a trigger key, delete it and inject expansion
//  - Resolve variables ({date}, {time}, {clipboard}, custom vars) at expansion time

use crate::models::RootConfig;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

use rdev::{listen, Event, EventType, Key as RKey};

// Maximum characters to keep in the rolling buffer
const BUFFER_SIZE: usize = 64;

// Delay before injecting text after a hotkey fires (ms)
const HOTKEY_INJECT_DELAY_MS: u64 = 80;

// ── Variable resolution ───────────────────────────────────────────────────

fn resolve_variables(text: &str, config: &RootConfig) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Get current local datetime components using chrono-free approach
    // We use the time crate via std primitives
    let now = chrono_now();

    let mut result = text.to_string();

    // Built-in variables
    result = result.replace("{date}",     &now.date);
    result = result.replace("{time}",     &now.time);
    result = result.replace("{datetime}", &format!("{} {}", now.date, now.time));
    result = result.replace("{day}",      &now.day);
    result = result.replace("{month}",    &now.month);
    result = result.replace("{year}",     &now.year);

    // Clipboard
    if result.contains("{clipboard}") {
        let clipboard_text = get_clipboard().unwrap_or_default();
        result = result.replace("{clipboard}", &clipboard_text);
    }

    // Custom variables
    for var in &config.custom_variables {
        let token = format!("{{{}}}", var.name);
        result = result.replace(&token, &var.value);
    }

    result
}

struct DateTime {
    date:  String,
    time:  String,
    day:   String,
    month: String,
    year:  String,
}

fn chrono_now() -> DateTime {
    // Use the `time` approach via std — no extra crate needed
    // We shell out to a simple calculation using SystemTime
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Simple epoch → date conversion (UTC, good enough for text expansion)
    let days_since_epoch = secs / 86400;
    let secs_today = secs % 86400;

    let hours   = secs_today / 3600;
    let minutes = (secs_today % 3600) / 60;

    // Zeller's-style date calculation from epoch days
    let (y, m, d) = days_from_epoch(days_since_epoch as i64);

    let day_of_week = ((days_since_epoch + 3) % 7) as usize; // 0 = Sunday
    let days   = ["Sunday","Monday","Tuesday","Wednesday","Thursday","Friday","Saturday"];
    let months = ["January","February","March","April","May","June",
                  "July","August","September","October","November","December"];

    DateTime {
        date:  format!("{:02}/{:02}/{}", d, m, y),
        time:  format!("{:02}:{:02}", hours, minutes),
        day:   days[day_of_week].to_string(),
        month: months[(m - 1) as usize].to_string(),
        year:  y.to_string(),
    }
}

fn days_from_epoch(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
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

fn get_clipboard() -> Option<String> {
    // Use enigo's clipboard reading via a temporary instance
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let out = Command::new("powershell")
            .args(["-Command", "Get-Clipboard"])
            .output()
            .ok()?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

// ── Key conversion ────────────────────────────────────────────────────────

/// Convert an rdev Key to a printable character if possible
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
        RKey::Slash => Some('/'), RKey::BackSlash => Some('\\'),
        RKey::Dot => Some('.'), RKey::Comma => Some(','),
        RKey::SemiColon => Some(';'), RKey::Quote => Some('\''),
        RKey::LeftBracket => Some('['), RKey::RightBracket => Some(']'),
        RKey::Minus => Some('-'), RKey::Equal => Some('='),
        RKey::BackQuote => Some('`'),
        RKey::Space => Some(' '),
        _ => None,
    }
}

/// Convert rdev modifier key names to our stored format
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
    // Named keys that can appear in hotkey combos
    let s = match key {
        RKey::F1  => "F1",  RKey::F2  => "F2",  RKey::F3  => "F3",
        RKey::F4  => "F4",  RKey::F5  => "F5",  RKey::F6  => "F6",
        RKey::F7  => "F7",  RKey::F8  => "F8",  RKey::F9  => "F9",
        RKey::F10 => "F10", RKey::F11 => "F11", RKey::F12 => "F12",
        RKey::Tab         => "Tab",
        RKey::Escape      => "Escape",
        RKey::Return      => "Return",
        RKey::Space       => "Space",
        RKey::UpArrow     => "Up",
        RKey::DownArrow   => "Down",
        RKey::LeftArrow   => "Left",
        RKey::RightArrow  => "Right",
        RKey::Home        => "Home",
        RKey::End         => "End",
        RKey::PageUp      => "PageUp",
        RKey::PageDown    => "PageDown",
        RKey::Insert      => "Insert",
        RKey::Delete      => "Delete",
        _ => return rkey_to_char(key).map(|c| c.to_uppercase().to_string()),
    };
    Some(s.to_string())
}

// ── Text injection ────────────────────────────────────────────────────────

fn inject_text(text: &str) {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => { eprintln!("[engine] Failed to create Enigo: {e}"); return; }
    };

    if let Err(e) = enigo.text(text) {
        eprintln!("[engine] Failed to inject text: {e}");
    }
}

fn delete_chars(n: usize) {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => { eprintln!("[engine] Failed to create Enigo: {e}"); return; }
    };

    for _ in 0..n {
        let _ = enigo.key(Key::Backspace, Click);
    }
}

// ── Engine ────────────────────────────────────────────────────────────────

pub fn start(config: Arc<Mutex<RootConfig>>) {
    thread::spawn(move || {
        let buffer: Arc<Mutex<Vec<char>>> = Arc::new(Mutex::new(Vec::new()));
        let held_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let buffer_clone     = Arc::clone(&buffer);
        let held_clone       = Arc::clone(&held_keys);
        let config_clone     = Arc::clone(&config);

        listen(move |event: Event| {
            match event.event_type {

                // ── Key pressed ───────────────────────────────────────────
                EventType::KeyPress(key) => {

                    // Track modifier keys for hotkey detection
                    if let Some(modifier) = rkey_to_modifier_str(&key) {
                        let mut held = held_clone.lock().unwrap();
                        if !held.contains(&modifier.to_string()) {
                            held.push(modifier.to_string());
                        }
                        return;
                    }

                    let cfg = config_clone.lock().unwrap();
                    if !cfg.enabled { return; }

                    // ── Hotkey check ──────────────────────────────────────
                    {
                        let held = held_clone.lock().unwrap();
                        if !held.is_empty() {
                            if let Some(key_name) = rkey_to_name(&key) {
                                // Build the current combo string
                                let mut parts = held.clone();
                                parts.push(key_name);
                                let combo = parts.join("+");

                                // Check against registered hotkeys
                                for hotkey in &cfg.hotkeys {
                                    if hotkey.keys.eq_ignore_ascii_case(&combo) {
                                        if let Some(expansion) = cfg.expansions.get(&hotkey.expansion_id) {
                                            let text = resolve_variables(&expansion.text, &cfg);
                                            drop(cfg);
                                            drop(held);
                                            thread::sleep(Duration::from_millis(HOTKEY_INJECT_DELAY_MS));
                                            inject_text(&text);
                                            return;
                                        }
                                    }
                                }
                            }
                            return; // modifier held but no hotkey matched — ignore
                        }
                    }

                    // ── Buffer update ─────────────────────────────────────
                    {
                        let mut buf = buffer_clone.lock().unwrap();

                        match key {
                            RKey::Backspace => { buf.pop(); }
                            RKey::Return | RKey::Escape => { buf.clear(); }
                            _ => {
                                if let Some(c) = rkey_to_char(&key) {
                                    buf.push(c);
                                    if buf.len() > BUFFER_SIZE {
                                        buf.remove(0);
                                    }
                                } else {
                                    // Unknown key — clear buffer to avoid false matches
                                    buf.clear();
                                }
                            }
                        }

                        // ── Trigger check ─────────────────────────────────
                        let buf_str: String = buf.iter().collect();

                        for trigger in &cfg.triggers {
                            if buf_str.ends_with(&trigger.key) {
                                // Word boundary check
                                if trigger.word_boundary {
                                    let before = buf_str.len() - trigger.key.len();
                                    if before > 0 {
                                        let prev_char = buf_str.chars().nth(before - 1).unwrap_or(' ');
                                        if !prev_char.is_whitespace() {
                                            continue;
                                        }
                                    }
                                }

                                if let Some(expansion) = cfg.expansions.get(&trigger.expansion_id) {
                                    let text = resolve_variables(&expansion.text, &cfg);
                                    let delete_count = trigger.key.len();
                                    buf.clear();
                                    drop(buf);
                                    drop(cfg);

                                    thread::sleep(Duration::from_millis(30));
                                    delete_chars(delete_count);
                                    thread::sleep(Duration::from_millis(30));
                                    inject_text(&text);
                                    return;
                                }
                            }
                        }
                    }
                }

                // ── Key released ──────────────────────────────────────────
                EventType::KeyRelease(key) => {
                    if let Some(modifier) = rkey_to_modifier_str(&key) {
                        let mut held = held_clone.lock().unwrap();
                        held.retain(|k| k != modifier);
                    }
                }

                _ => {}
            }
        }).unwrap_or_else(|e| eprintln!("[engine] rdev listen error: {:?}", e));
    });
}