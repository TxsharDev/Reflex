//! Settings and launch counts, stored as JSON in `%APPDATA%\reflex`.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rows shown at once.
    pub max_results: usize,
    /// Folders to index. Drive roots (`D:\`) are indexed shallower.
    pub index_dirs: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_results: 8,
            index_dirs: default_index_dirs(),
        }
    }
}

fn default_index_dirs() -> Vec<String> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.to_string_lossy().into_owned());
    }
    for letter in 'A'..='Z' {
        let drive = format!("{letter}:\\");
        if PathBuf::from(&drive).exists() {
            dirs.push(drive);
        }
    }
    dirs
}

fn dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("reflex")
}

impl Config {
    /// Loads the config, writing the defaults on first run so there is a file to edit.
    pub fn load() -> Self {
        let path = dir().join("config.json");
        if let Ok(text) = std::fs::read_to_string(&path) {
            return serde_json::from_str(&text).unwrap_or_default();
        }
        let config = Self::default();
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            let _ = std::fs::create_dir_all(dir());
            let _ = std::fs::write(path, json);
        }
        config
    }
}

/// Launches per app name (lowercase), used to rank apps.
pub fn load_launch_counts() -> HashMap<String, u32> {
    std::fs::read_to_string(dir().join("launch_counts.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_launch_counts(counts: &HashMap<String, u32>) {
    if let Ok(json) = serde_json::to_string(counts) {
        let _ = std::fs::create_dir_all(dir());
        let _ = std::fs::write(dir().join("launch_counts.json"), json);
    }
}
