//! Stub platform implementation for non-Windows builds.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::path::PathBuf;

pub fn register_hotkey(_triggered: Arc<AtomicBool>) {}
pub fn apply_rounded_corners() {}

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub path: PathBuf,
    pub launch_count: u32,
}

pub fn discover_apps() -> Vec<AppEntry> { Vec::new() }
pub fn launch_app(_path: &PathBuf, _as_admin: bool) {}
pub fn kill_process(_name: &str) -> Result<(), String> { Err("Not supported".into()) }
pub fn system_action(_action: &str) -> Result<(), String> { Err("Not supported".into()) }
pub fn get_ip() -> String { "unknown".into() }
