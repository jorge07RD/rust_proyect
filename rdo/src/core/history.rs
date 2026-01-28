use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RestoreHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub database_name: String,
    pub backup_folder: String,
    pub success: bool,
    pub error_message: Option<String>,
}

impl RestoreHistoryEntry {
    pub fn new(
        database_name: String,
        backup_folder: String,
        success: bool,
        error_message: Option<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            database_name,
            backup_folder,
            success,
            error_message,
        }
    }
}

const HISTORY_FILE: &str = "restore_history.json";

pub struct HistoryManager {
    entries: Vec<RestoreHistoryEntry>,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Load history from file
    pub fn load() -> anyhow::Result<Self> {
        if !std::path::Path::new(HISTORY_FILE).exists() {
            return Ok(Self::new());
        }

        let mut file = fs::File::open(HISTORY_FILE)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let entries: Vec<RestoreHistoryEntry> = serde_json::from_str(&contents)?;
        Ok(Self { entries })
    }

    /// Save history to file
    pub fn save(&self) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        let mut file = fs::File::create(HISTORY_FILE)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Add a new entry to history
    pub fn add_entry(&mut self, entry: RestoreHistoryEntry) -> anyhow::Result<()> {
        self.entries.push(entry);
        self.save()?;
        Ok(())
    }

    /// Get the n most recent entries
    pub fn get_recent(&self, n: usize) -> Vec<RestoreHistoryEntry> {
        let mut entries = self.entries.clone();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.into_iter().take(n).collect()
    }
}
