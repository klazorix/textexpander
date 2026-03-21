use std::{path::PathBuf, sync::{Arc, Mutex}, thread};

use crate::models::RootConfig;
use crate::helpers::persist_config;
use super::variables::today_string;

pub fn record_stats(config: &Arc<Mutex<RootConfig>>, path: &PathBuf, expansion_id: &str) {
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
        persist_config(&path, &cfg);
    });
}
