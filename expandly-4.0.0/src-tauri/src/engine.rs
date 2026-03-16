// src-tauri/src/engine.rs
//
// Keystroke engine for Expandly v4.0.0

use crate::models::RootConfig;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use enigo::{
    Direction::Click,
    Enigo, Key, Keyboard, Settings,
};

use rdev::{listen, Event, EventType, Key as RKey};

const BUFFER_SIZE: usize = 16;
const HOTKEY_INJECT_DELAY_MS: u64 = 80;

pub fn days_from_epoch_pub(z: i64) -> (i64, i64, i64) {
    days_from_epoch(z)
}

// ── Variable resolution ───────────────────────────────────────────────────

fn resolve_variables(text: &str, config: &RootConfig) -> String {
    let now = chrono_now();
    let mut result = text.to_string();
    result = result.replace("{date}",     &now.date);
    result = result.replace("{time}",     &now.time);
    result = result.replace("{datetime}", &format!("{} {}", now.date, now.time));
    result = result.replace("{day}",      &now.day);
    result = result.replace("{month}",    &now.month);
    result = result.replace("{year}",     &now.year);
    if result.contains("{clipboard}") {
        let clipboard_text = get_clipboard().unwrap_or_default();
        result = result.replace("{clipboard}", &clipboard_text);
    }
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
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs             = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days_since_epoch = secs / 86400;
    let secs_today       = secs % 86400;
    let hours            = secs_today / 3600;
    let minutes          = (secs_today % 3600) / 60;
    let (y, m, d)        = days_from_epoch(days_since_epoch as i64);
    let day_of_week      = ((days_since_epoch + 3) % 7) as usize;
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
    let days      = secs / 86400;
    let (y, m, d) = days_from_epoch(days as i64);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn get_clipboard() -> Option<String> {
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
        RKey::Tab        => "Tab",
        RKey::Escape     => "Escape",
        RKey::Return     => "Return",
        RKey::Space      => "Space",
        RKey::UpArrow    => "Up",
        RKey::DownArrow  => "Down",
        RKey::LeftArrow  => "Left",
        RKey::RightArrow => "Right",
        RKey::Home       => "Home",
        RKey::End        => "End",
        RKey::PageUp     => "PageUp",
        RKey::PageDown   => "PageDown",
        RKey::Insert     => "Insert",
        RKey::Delete     => "Delete",
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

fn record_stats(
    config: &Arc<Mutex<RootConfig>>,
    path: &PathBuf,
    expansion_id: &str,
) {
    let mut cfg = config.lock().unwrap();
    if !cfg.track_stats { return; }
    cfg.stats.total_expansions += 1;
    *cfg.stats.expansions_per_day.entry(today_string()).or_insert(0) += 1;
    *cfg.stats.expansion_counts.entry(expansion_id.to_string()).or_insert(0) += 1;
    crate::helpers::persist_config(path, &cfg);
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

            let result = listen(move |event: Event| {
                match event.event_type {

                    // ── Key pressed ───────────────────────────────────────────
                    EventType::KeyPress(key) => {

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
                                    let mut parts = held.clone();
                                    parts.push(key_name);
                                    let combo = parts.join("+");

                                    for hotkey in &cfg.hotkeys {
                                        if hotkey.keys.eq_ignore_ascii_case(&combo) {
                                            if let Some(expansion) = cfg.expansions.get(&hotkey.expansion_id) {
                                                let text   = resolve_variables(&expansion.text, &cfg);
                                                let exp_id = hotkey.expansion_id.clone();
                                                drop(cfg);
                                                drop(held);
                                                thread::sleep(Duration::from_millis(HOTKEY_INJECT_DELAY_MS));
                                                inject_text(&text);
                                                record_stats(&config_clone, &path_clone, &exp_id);
                                                return;
                                            }
                                        }
                                    }
                                }
                                return;
                            }
                        }

                        // ── Buffer update ─────────────────────────────────────
                        {
                            let mut buf = buffer_clone.lock().unwrap();

                            match key {
                                RKey::Backspace             => { buf.pop(); }
                                RKey::Return | RKey::Escape => { buf.clear(); }
                                _ => {
                                    if let Some(c) = rkey_to_char(&key) {
                                        buf.push(c);
                                        if buf.len() > BUFFER_SIZE { buf.remove(0); }
                                    } else {
                                        buf.clear();
                                    }
                                }
                            }

                            // ── Trigger check ─────────────────────────────────
                            let buf_str: String = buf.iter().collect();
                            let mut matched: Option<(String, usize, String)> = None;

                            for trigger in &cfg.triggers {
                                if buf_str.ends_with(&trigger.key) {
                                    if trigger.word_boundary {
                                        let before = buf_str.len() - trigger.key.len();
                                        if before > 0 {
                                            let prev = buf_str.chars().nth(before - 1).unwrap_or(' ');
                                            if !prev.is_whitespace() { continue; }
                                        }
                                    }
                                    if let Some(expansion) = cfg.expansions.get(&trigger.expansion_id) {
                                        let text   = resolve_variables(&expansion.text, &cfg);
                                        let del    = trigger.key.len();
                                        let exp_id = trigger.expansion_id.clone();
                                        matched = Some((text, del, exp_id));
                                        break;
                                    }
                                }
                            }

                            if matched.is_some() { buf.clear(); }
                            drop(buf);
                            drop(cfg);

                            if let Some((text, delete_count, expansion_id)) = matched {
                                let (sound_enabled, sound_path, delay) = {
                                    let cfg = config_clone.lock().unwrap();
                                    (cfg.sound_enabled, cfg.sound_path.clone(), cfg.expansion_delay_ms)
                                };

                                thread::sleep(Duration::from_millis(delay));
                                delete_chars(delete_count);
                                inject_text(&text);
                                record_stats(&config_clone, &path_clone, &expansion_id);

                                if sound_enabled {
                                    if let Some(path) = sound_path {
                                        thread::spawn(move || {
                                            use rodio::{Decoder, OutputStream, Sink};
                                            use std::fs::File;
                                            use std::io::BufReader;

                                            let Ok((_stream, handle)) = OutputStream::try_default()       else { return };
                                            let Ok(sink)              = Sink::try_new(&handle)             else { return };
                                            let Ok(file)              = File::open(&path)                  else { return };
                                            let Ok(source)            = Decoder::new(BufReader::new(file)) else { return };

                                            sink.append(source);
                                            let start = std::time::Instant::now();
                                            while !sink.empty() {
                                                if start.elapsed().as_secs() >= 10 {
                                                    sink.stop();
                                                    break;
                                                }
                                                thread::sleep(Duration::from_millis(100));
                                            }
                                        });
                                    }
                                }

                                return;
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
            });

            eprintln!("[engine] rdev listener stopped: {:?} — restarting in 1s", result);
            thread::sleep(Duration::from_secs(1));
        }
    });
}