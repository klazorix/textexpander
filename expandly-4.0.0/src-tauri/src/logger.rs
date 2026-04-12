// src-tauri/src/logger.rs
//
// Debug logger for Expandly v4.0.0
//
// Writes dated log files to <app_data>/debug/YYYY-MM-DD.log
// Log level is checked before any file I/O so there's no overhead when disabled.

use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::engine::variables::today_string;

const LOG_RETENTION_SECS: u64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Errors,
    Warnings,
    Verbose,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "warnings" => LogLevel::Warnings,
            "verbose"  => LogLevel::Verbose,
            _          => LogLevel::Errors,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Errors   => "errors",
            LogLevel::Warnings => "warnings",
            LogLevel::Verbose  => "verbose",
        }
    }
}

pub struct DebugLogger {
    pub enabled: bool,
    pub level:   LogLevel,
    pub dir:     PathBuf,
}

impl DebugLogger {
    pub fn new(enabled: bool, level: LogLevel, app_data_dir: PathBuf) -> Self {
        let dir = app_data_dir.join("debug");
        if enabled { let _ = fs::create_dir_all(&dir); }
        Self { enabled, level, dir }
    }

    fn write(&self, label: &str, message: &str) {
        if !self.enabled { return; }
        let path = self.dir.join(format!("{}.log", today_string()));
        let line = format!("[{}] [{}] {}\n", chrono_time(), label, message);
        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = file.write_all(line.as_bytes());
        }
    }

    fn should_log(&self, level: LogLevel) -> bool {
        self.enabled
            && match self.level {
                LogLevel::Errors => matches!(level, LogLevel::Errors),
                LogLevel::Warnings => !matches!(level, LogLevel::Verbose),
                LogLevel::Verbose => true,
            }
    }

    pub fn error(&self, message: &str) {
        if self.should_log(LogLevel::Errors) {
            self.write("ERROR", message);
        }
    }

    pub fn warning(&self, message: &str) {
        if self.should_log(LogLevel::Warnings) {
            self.write("WARN", message);
        }
    }

    pub fn verbose(&self, message: &str) {
        if self.should_log(LogLevel::Verbose) {
            self.write("VERBOSE", message);
        }
    }
}

fn unix_now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn chrono_time() -> String {
    let secs = unix_now_secs();
    let secs_today = secs % 86400;
    format!("{:02}:{:02}:{:02}", secs_today / 3600, (secs_today % 3600) / 60, secs_today % 60)
}

/// Purge log files older than 7 days from the debug directory.
pub fn purge_old_logs(app_data_dir: &PathBuf) {
    let dir = app_data_dir.join("debug");
    if !dir.exists() { return; }

    let cutoff = unix_now_secs().saturating_sub(LOG_RETENTION_SECS);

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("log") { continue; }
            if let Ok(meta) = fs::metadata(&path) {
                if let Ok(modified) = meta.modified() {
                    let modified_secs = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if modified_secs < cutoff {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

// Global logger instance — shared across engine, commands, and lib
pub type SharedLogger = Arc<Mutex<DebugLogger>>;

pub fn make_logger(enabled: bool, level: LogLevel, app_data_dir: PathBuf) -> SharedLogger {
    Arc::new(Mutex::new(DebugLogger::new(enabled, level, app_data_dir)))
}
