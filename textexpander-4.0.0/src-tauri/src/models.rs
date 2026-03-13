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

impl Default for RootConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            enabled: true,
            expansions: HashMap::new(),
            triggers: Vec::new(),
            hotkeys: Vec::new(),
            custom_variables: Vec::new(),
            stats: GlobalStats::default(),
        }
    }
}