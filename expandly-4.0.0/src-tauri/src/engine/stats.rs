use std::sync::{Arc, Mutex};
use std::thread;

use rusqlite::Connection;

use crate::models::RootConfig;
use crate::db;
use super::variables::today_string;

pub fn record_stats(
    config: &Arc<Mutex<RootConfig>>,
    db: &Arc<Mutex<Connection>>,
    expansion_id: &str,
) {
    let (config, db, exp_id) = (Arc::clone(config), Arc::clone(db), expansion_id.to_string());
    thread::spawn(move || {
        let mut cfg = config.lock().unwrap();
        if !cfg.track_stats { return; }

        let today = today_string();
        *cfg.stats.expansions_per_day.entry(today.clone()).or_insert(0) += 1;
        *cfg.stats.expansion_counts.entry(exp_id.clone()).or_insert(0) += 1;
        drop(cfg);

        let conn = db.lock().unwrap();
        if let Err(e) = db::increment_stats(&conn, &exp_id, &today) {
            eprintln!("[engine] Failed to record stats: {e}");
        }
    });
}
