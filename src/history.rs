use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::config::config_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub text: String,
    pub timestamp: i64,
    pub word_count: usize,
}

const MAX_ENTRIES: usize = 10;
const FILENAME: &str = "history.json";

fn history_path() -> Result<PathBuf> {
    let dir = config_dir()?;
    Ok(dir.join(FILENAME))
}

pub fn load() -> Vec<HistoryEntry> {
    let path = match history_path() {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to resolve history path: {}", e);
            return Vec::new();
        }
    };

    if !path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to read history file: {}", e);
            return Vec::new();
        }
    };

    match serde_json::from_str::<Vec<HistoryEntry>>(&content) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to parse history file: {}", e);
            Vec::new()
        }
    }
}

pub fn push(text: &str) -> Result<()> {
    let path = history_path()?;
    let mut entries = load();

    let word_count = text.split_whitespace().count();
    let entry = HistoryEntry {
        text: text.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0),
        word_count,
    };

    entries.insert(0, entry);
    entries.truncate(MAX_ENTRIES);

    let content = serde_json::to_string_pretty(&entries)?;
    std::fs::write(&path, content)?;

    info!("History entry saved ({} entries)", entries.len());
    Ok(())
}

pub fn clear() -> Result<()> {
    let path = history_path()?;

    if path.exists() {
        std::fs::remove_file(&path)?;
        info!("History cleared");
    }

    Ok(())
}
