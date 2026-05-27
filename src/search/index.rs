//! Filesystem indexer - scans drives and maintains a searchable file index.
//!
//! Phase 1: Index user directories (fast, <2s)
//! Phase 2: Index remaining drives in background (can take longer)
//! Skips system dirs, hidden files, and known slow directories.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub ext: String,
    pub name_lower: String,
}

pub struct FileIndex {
    entries: Arc<RwLock<Vec<FileEntry>>>,
    indexing: Arc<AtomicBool>,
    count: Arc<AtomicUsize>,
}

impl FileIndex {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            indexing: Arc::new(AtomicBool::new(false)),
            count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Index in two phases: user dirs first (fast), then all drives.
    pub fn start_indexing(&self, _dirs: Vec<PathBuf>) {
        let entries = self.entries.clone();
        let indexing = self.indexing.clone();
        let count = self.count.clone();

        indexing.store(true, Ordering::SeqCst);

        std::thread::spawn(move || {
            let start = Instant::now();
            let mut all = Vec::with_capacity(200_000);

            // Phase 1: User directories (fast)
            let priority_dirs = get_priority_dirs();
            for dir in &priority_dirs {
                walk_dir(dir, &mut all, 0, 10); // Max depth 10 for user dirs
            }

            // Commit phase 1 results immediately
            let phase1_count = all.len();
            all.sort_by(|a, b| a.name_lower.cmp(&b.name_lower));
            if let Ok(mut lock) = entries.write() {
                *lock = all.clone();
            }
            count.store(phase1_count, Ordering::SeqCst);

            // Phase 2: All drive roots (skip already-indexed user dirs)
            let drive_roots = get_drive_roots();
            for root in &drive_roots {
                if priority_dirs.iter().any(|p| root.starts_with(p) || p.starts_with(root)) {
                    continue; // Already indexed
                }
                walk_dir(root, &mut all, 0, 6); // Shallower for drive roots
            }

            all.sort_by(|a, b| a.name_lower.cmp(&b.name_lower));
            let total = all.len();
            if let Ok(mut lock) = entries.write() {
                *lock = all;
            }
            count.store(total, Ordering::SeqCst);
            indexing.store(false, Ordering::SeqCst);

            let _ = start; // Suppress unused warning in release
        });
    }

    pub fn is_indexing(&self) -> bool {
        self.indexing.load(Ordering::Relaxed)
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<FileEntry> {
        let query_lower = query.to_lowercase();
        let query_parts: Vec<&str> = query_lower.split_whitespace().collect();
        let entries = self.entries.read().unwrap();

        let mut results: Vec<(usize, &FileEntry)> = entries
            .iter()
            .filter_map(|entry| {
                let score = score_match(entry, &query_lower, &query_parts)?;
                Some((score, entry))
            })
            .collect();

        results.sort_by(|a, b| b.0.cmp(&a.0));
        results.truncate(limit);
        results.into_iter().map(|(_, e)| e.clone()).collect()
    }
}

/// Priority directories - indexed first, deeper.
fn get_priority_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = dirs::home_dir() {
        // Desktop, Documents, Downloads, Projects
        for sub in ["Desktop", "Documents", "Downloads", "Projects", "dev", "code", "repos", "src"] {
            let p = home.join(sub);
            if p.exists() { dirs.push(p); }
        }
        dirs.push(home); // Home itself (shallow)
    }

    dirs
}

/// All drive roots that exist.
fn get_drive_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive = PathBuf::from(format!("{}:\\", letter as char));
        if drive.exists() {
            roots.push(drive);
        }
    }
    roots
}

// Directories to always skip
const SKIP_DIRS: &[&str] = &[
    "node_modules", ".git", "target", "__pycache__", ".venv", "venv",
    ".cargo", ".rustup", ".npm", ".cache", ".local", "AppData",
    "$Recycle.Bin", "$WinREAgent", "System Volume Information",
    "ProgramData", "Recovery", "PerfLogs",
];

const SKIP_PATHS: &[&str] = &[
    "\\windows\\",
    "\\windows.old\\",
    "\\program files\\windowsapps",
    "\\appdata\\local\\temp",
    "\\appdata\\local\\microsoft",
    "\\appdata\\local\\packages",
];

fn walk_dir(dir: &Path, entries: &mut Vec<FileEntry>, depth: usize, max_depth: usize) {
    if depth > max_depth { return; }

    let Ok(read) = std::fs::read_dir(dir) else { return };

    for entry in read.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden/system
        if name.starts_with('.') || name.starts_with('$') { continue; }
        if SKIP_DIRS.contains(&name.as_str()) { continue; }

        let path = entry.path();
        let path_lower = path.to_string_lossy().to_lowercase();

        // Skip known slow paths
        if SKIP_PATHS.iter().any(|s| path_lower.contains(s)) { continue; }

        let name_lower = name.to_lowercase();
        let ext = path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        entries.push(FileEntry {
            name: name.clone(),
            path: path.clone(),
            is_dir: meta.is_dir(),
            size: meta.len(),
            ext,
            name_lower,
        });

        if meta.is_dir() {
            walk_dir(&path, entries, depth + 1, max_depth);
        }
    }
}

/// Score a file entry against a query. Smarter matching with extension and path awareness.
fn score_match(entry: &FileEntry, query: &str, parts: &[&str]) -> Option<usize> {
    if query.is_empty() { return None; }

    let name = &entry.name_lower;

    // Multi-word: all parts must match somewhere (name or path)
    if parts.len() > 1 {
        let path_lower = entry.path.to_string_lossy().to_lowercase();
        let all_match = parts.iter().all(|p| name.contains(p) || path_lower.contains(p));
        if !all_match { return None; }
        return Some(800 + (100 - name.len().min(100)));
    }

    // Single word matching
    // Exact match
    if name == query { return Some(10000); }

    // Extension match: ".rs", ".py", ".txt"
    if query.starts_with('.') && entry.ext == query[1..] {
        return Some(3000 + (100 - name.len().min(100)));
    }

    // Starts with
    if name.starts_with(query) { return Some(5000 + (100 - name.len().min(100))); }

    // Contains
    if name.contains(query) { return Some(1000 + (100 - name.len().min(100))); }

    // Fuzzy
    let mut query_chars = query.chars().peekable();
    let mut consecutive = 0;
    let mut max_consecutive = 0;
    let mut matched = 0;

    for c in name.chars() {
        if query_chars.peek() == Some(&c) {
            query_chars.next();
            matched += 1;
            consecutive += 1;
            max_consecutive = max_consecutive.max(consecutive);
        } else {
            consecutive = 0;
        }
    }

    if query_chars.peek().is_none() {
        Some(100 + max_consecutive * 50 + matched * 10 + (100 - name.len().min(100)))
    } else {
        None
    }
}
