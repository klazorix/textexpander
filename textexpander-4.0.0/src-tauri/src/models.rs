use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expansion {
    pub id: String,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub key: String,
    pub expansion_id: String,
    pub word_boundary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub id: String,
    pub keys: String,
    pub expansion_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomVariable {
    pub id: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalStats {
    pub total_expansions: u64,
    pub total_chars_saved: u64,

    #[serde(default)]
    pub expansions_per_day: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootConfig {
    pub version: String,

    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default)]
    pub sound_enabled: bool,

    #[serde(default)]
    pub sound_path: Option<String>,

    #[serde(default = "default_show_in_taskbar")]
    pub show_in_taskbar: bool,

    #[serde(default)]
    pub launch_at_startup: bool,

    #[serde(default)]
    pub minimise_to_tray: bool,

    #[serde(default)]
    pub theme: String,

    #[serde(default)]
    pub expansions: HashMap<String, Expansion>,

    #[serde(default)]
    pub triggers: Vec<Trigger>,

    #[serde(default)]
    pub hotkeys: Vec<Hotkey>,

    #[serde(default)]
    pub custom_variables: Vec<CustomVariable>,

    #[serde(default)]
    pub stats: GlobalStats,
}

fn default_enabled() -> bool {
    true
}

fn default_show_in_taskbar() -> bool {
    true
}

fn default_theme() -> String {
    "starry-blue".to_string()
}

impl Default for RootConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            enabled: true,
            sound_enabled: false,
            sound_path: None,
            show_in_taskbar: true,
            launch_at_startup: false,
            minimise_to_tray: false,
            theme: "starry-blue".to_string(),
            expansions: HashMap::new(),
            triggers: Vec::new(),
            hotkeys: Vec::new(),
            custom_variables: Vec::new(),
            stats: GlobalStats::default(),
        }
    }
}