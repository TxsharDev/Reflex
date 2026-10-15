//! Text clipboard history for the current session.
//!
//! A background thread polls the clipboard sequence number, which is cheap
//! and never opens the clipboard, and reads the text only when it changes.
//! Content the owner marked private (password managers do this) is skipped.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::platform;

const MAX_HISTORY: usize = 200;

pub struct ClipboardHistory {
    entries: Arc<RwLock<Vec<String>>>,
    version: Arc<AtomicU64>,
}

impl ClipboardHistory {
    pub fn start() -> Self {
        let history = Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            version: Arc::new(AtomicU64::new(0)),
        };
        let entries = history.entries.clone();
        let version = history.version.clone();

        std::thread::spawn(move || {
            let mut last_sequence = 0;
            loop {
                let sequence = platform::clipboard_sequence();
                if sequence != last_sequence {
                    last_sequence = sequence;
                    if let Some(text) = read_text() {
                        remember(&mut entries.write().unwrap(), text);
                        version.fetch_add(1, Ordering::Relaxed);
                    }
                }
                std::thread::sleep(Duration::from_millis(250));
            }
        });

        history
    }

    /// Bumped whenever the history changes.
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    pub fn recent(&self, limit: usize) -> Vec<String> {
        self.entries
            .read()
            .unwrap()
            .iter()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        let query = query.to_lowercase();
        self.entries
            .read()
            .unwrap()
            .iter()
            .filter(|text| text.to_lowercase().contains(&query))
            .take(limit)
            .cloned()
            .collect()
    }
}

fn read_text() -> Option<String> {
    if platform::clipboard_is_private() {
        return None;
    }
    let text = arboard::Clipboard::new().ok()?.get_text().ok()?;
    (!text.trim().is_empty()).then_some(text)
}

/// Moves `text` to the front, dropping any older copy and the oldest overflow.
fn remember(entries: &mut Vec<String>, text: String) {
    entries.retain(|existing| *existing != text);
    entries.insert(0, text);
    entries.truncate(MAX_HISTORY);
}

/// Puts `text` on the clipboard. The watcher then moves it to the front.
pub fn copy(text: &str) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remember_dedupes_and_moves_to_front() {
        let mut entries = vec!["b".to_string(), "a".to_string()];
        remember(&mut entries, "a".to_string());
        assert_eq!(entries, ["a", "b"]);
    }

    #[test]
    fn remember_caps_history() {
        let mut entries: Vec<String> = (0..MAX_HISTORY).map(|i| i.to_string()).collect();
        remember(&mut entries, "new".to_string());
        assert_eq!(entries.len(), MAX_HISTORY);
        assert_eq!(entries[0], "new");
    }
}
