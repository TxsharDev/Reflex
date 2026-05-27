//! Configuration - persisted to disk as JSON.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_max_results")]
    pub max_results: usize,

    #[serde(default = "default_index_dirs")]
    pub index_dirs: Vec<String>,

    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_max_results() -> usize { 12 }
fn default_theme() -> String { "dark".into() }

fn default_index_dirs() -> Vec<String> {
    // Index common user directories + drive roots
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.to_string_lossy().to_string());
    }
    // Add drive roots that exist
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        if PathBuf::from(&drive).exists() {
            dirs.push(drive);
        }
    }
    dirs
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
            index_dirs: default_index_dirs(),
            theme: default_theme(),
        }
    }
}

impl Config {
    fn path() -> Option<PathBuf> {
        Some(dirs::config_dir()?.join("reflex").join("config.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::path() else { return Self::default() };
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let Some(path) = Self::path() else { return };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }
}
