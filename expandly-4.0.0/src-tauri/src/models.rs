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
    #[serde(default)]
    pub expansions_per_day: HashMap<String, u64>,

    #[serde(default)]
    pub expansion_counts: HashMap<String, u64>,
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

    #[serde(default)]
    pub launch_minimised: bool,

    #[serde(default = "default_track_stats")]
    pub track_stats: bool,

    #[serde(default = "default_expansion_delay_ms")]
    pub expansion_delay_ms: u64,

    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,

    #[serde(default = "default_hotkey_delay")]
    pub hotkey_delay_ms: u64,

    #[serde(default = "default_clear_buffer_on_switch")]
    pub clear_buffer_on_switch: bool,

    #[serde(default)]
    pub debug_enabled: bool,

    #[serde(default = "default_debug_level")]
    pub debug_level: String,
}

fn default_enabled() -> bool { true }
fn default_track_stats() -> bool { true }
fn default_expansion_delay_ms() -> u64 { 250 }
fn default_buffer_size() -> usize { 32 }
fn default_hotkey_delay() -> u64 { 80 }
fn default_clear_buffer_on_switch() -> bool { true }
fn default_debug_level() -> String { "errors".to_string() }
fn default_sound_path() -> String { "https://cdn.klazorix.com/expandly/default_sound.mp3".to_string() }
fn wiki_url() -> String { "https://github.com/klazorix/expandly/wiki".to_string() }

fn expansion(id: String, name: &str, text: &str) -> Expansion {
    Expansion { id, name: name.to_string(), text: text.to_string() }
}

fn variable(id: String, name: &str, value: String) -> CustomVariable {
    CustomVariable { id, name: name.to_string(), value }
}

impl Default for RootConfig {
    fn default() -> Self {
        let exp1_id     = uuid::Uuid::new_v4().to_string();
        let exp2_id     = uuid::Uuid::new_v4().to_string();
        let exp3_id     = uuid::Uuid::new_v4().to_string();
        let trigger1_id = uuid::Uuid::new_v4().to_string();
        let trigger2_id = uuid::Uuid::new_v4().to_string();
        let trigger3_id = uuid::Uuid::new_v4().to_string();
        let variable1_id = uuid::Uuid::new_v4().to_string();
        let variable2_id = uuid::Uuid::new_v4().to_string();

        let version = env!("CARGO_PKG_VERSION").to_string();
        let expansions = HashMap::from([
            (exp1_id.clone(), expansion(
                exp1_id.clone(),
                "Welcome to Expandly",
                "{greeting}, welcome to Expandly {version}! This is your first snippet. Try editing me or creating your own!",
            )),
            (exp2_id.clone(), expansion(
                exp2_id.clone(),
                "Current Date & Time",
                "The date today is {date} and the time is {time}.",
            )),
            (exp3_id.clone(), expansion(
                exp3_id.clone(),
                "Expandly Assistance",
                "Need help with Expandly? Check out the documentation at {wiki}.",
            )),
        ]);

        Self {
            version:                version.clone(),
            enabled:                true,
            expansion_delay_ms:     250,
            buffer_size:            32,
            hotkey_delay_ms:        80,
            clear_buffer_on_switch: true,
            sound_enabled:          false,
            sound_path:             Some(default_sound_path()),
            launch_at_startup:      false,
            launch_minimised:       false,
            minimise_to_tray:       true,
            theme:                  "starry-blue".to_string(),
            expansions,
            triggers: vec![
                Trigger { id: trigger1_id, key: "/hello".to_string(), expansion_id: exp1_id, word_boundary: true },
                Trigger { id: trigger2_id, key: "/time".to_string(),  expansion_id: exp2_id, word_boundary: true },
                Trigger { id: trigger3_id, key: "/help".to_string(),  expansion_id: exp3_id, word_boundary: true },
            ],
            hotkeys:          vec![],
            custom_variables: vec![
                variable(variable1_id, "version", version),
                variable(variable2_id, "wiki", wiki_url()),
            ],
            stats:         GlobalStats::default(),
            track_stats:   true,
            debug_enabled: false,
            debug_level:   "warnings".to_string(),
        }
    }
}
