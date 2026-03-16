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

}

fn default_enabled() -> bool {
    true
}

fn default_theme() -> String {
    "starry-blue".to_string()
}

fn default_track_stats() -> bool {
    true
}

fn default_expansion_delay_ms() -> u64 {
    325
}

fn default_buffer_size() -> usize {
    16
}

impl Default for RootConfig {
    fn default() -> Self {
        let exp1_id = uuid::Uuid::new_v4().to_string();
        let exp2_id = uuid::Uuid::new_v4().to_string();
        let trigger1_id = uuid::Uuid::new_v4().to_string();
        let trigger2_id = uuid::Uuid::new_v4().to_string();
        let variable_id = uuid::Uuid::new_v4().to_string();

        let mut expansions = HashMap::new();
        expansions.insert(exp1_id.clone(), Expansion {
            id: exp1_id.clone(),
            name: "Welcome to Expandly".to_string(),
            text: "Welcome to Expandly {version}! This is your first snippet. Try editing me or creating your own!".to_string(),
        });
        expansions.insert(exp2_id.clone(), Expansion {
            id: exp2_id.clone(),
            name: "Current Date & Time".to_string(),
            text: "The date today is {date} and the time is {time}.".to_string(),
        });

        Self {
            version: String::new(),
            
            enabled: true,
            expansion_delay_ms: 325,
            buffer_size: 16,

            sound_enabled: false,
            sound_path: None,

            launch_at_startup: false,
            launch_minimised: false,
            minimise_to_tray: false,

            theme: "starry-blue".to_string(),

            expansions,
            
            triggers: vec![
                Trigger {
                    id: trigger1_id,
                    key: "/hello".to_string(),
                    expansion_id: exp1_id,
                    word_boundary: true,
                },
                Trigger {
                    id: trigger2_id,
                    key: "/time".to_string(),
                    expansion_id: exp2_id,
                    word_boundary: true,
                },
            ],
            hotkeys: vec![],

            custom_variables: vec![
                CustomVariable {
                    id: variable_id,
                    name: "version".to_string(),
                    value: env!("CARGO_PKG_VERSION").to_string(),
                }
            ],

            stats: GlobalStats::default(),
            track_stats: true,
        }
    }
}