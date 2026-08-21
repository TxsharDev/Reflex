//! Clipboard history manager.
//!
//! Monitors the system clipboard and stores a history of copied items.
//! Supports text and images. Searchable.

use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY: usize = 200;

#[derive(Debug, Clone)]
pub struct ClipEntry {
    pub text: String,
    #[allow(dead_code)]
    pub timestamp: u64,
    pub pinned: bool,
}

pub struct ClipboardHistory {
    entries: Arc<RwLock<Vec<ClipEntry>>>,
    last_content: Arc<RwLock<String>>,
}

impl ClipboardHistory {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            last_content: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Start watching the clipboard on a background thread.
    pub fn start_watching(&self) {
        let entries = self.entries.clone();
        let last_content = self.last_content.clone();

        std::thread::spawn(move || {
            loop {
                if let Ok(mut clipboard) = arboard::Clipboard::new()
                    && let Ok(text) = clipboard.get_text()
                    && !text.is_empty()
                {
                    let last = last_content.read().unwrap().clone();
                    if text != last {
                        *last_content.write().unwrap() = text.clone();

                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        let mut entries = entries.write().unwrap();
                        entries.retain(|e| e.text != text);
                        entries.insert(0, ClipEntry { text, timestamp, pinned: false });

                        while entries.len() > MAX_HISTORY {
                            if let Some(pos) = entries.iter().rposition(|e| !e.pinned) {
                                entries.remove(pos);
                            } else {
                                break;
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
        });
    }

    /// Get all entries.
    pub fn entries(&self) -> Vec<ClipEntry> {
        self.entries.read().unwrap().clone()
    }

    /// Search clipboard history.
    pub fn search(&self, query: &str, limit: usize) -> Vec<ClipEntry> {
        let q = query.to_lowercase();
        let entries = self.entries.read().unwrap();
        entries
            .iter()
            .filter(|e| e.text.to_lowercase().contains(&q))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Paste a specific clip to the system clipboard.
    pub fn paste(&self, text: &str) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
        // Update last_content so we don't re-add it to history
        *self.last_content.write().unwrap() = text.to_string();
    }

    /// Clear all non-pinned entries.
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.retain(|e| e.pinned);
    }

    /// Toggle pin on an entry.
    #[allow(dead_code)]
    pub fn toggle_pin(&self, index: usize) {
        let mut entries = self.entries.write().unwrap();
        if let Some(entry) = entries.get_mut(index) {
            entry.pinned = !entry.pinned;
        }
    }
}
