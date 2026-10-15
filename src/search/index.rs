//! In-memory file index, built on a background thread.
//!
//! User folders are walked first (deep) so they are searchable within a
//! second or two; drive roots follow (shallower), skipping anything already
//! indexed. Results become searchable as each root finishes.
//!
//! Each folder path is stored once, and file names live in two flat string
//! buffers (original and lowercase) that entries index into. Half a million
//! files fit in a few tens of megabytes.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

const USER_DIR_DEPTH: usize = 10;
const DRIVE_DEPTH: usize = 6;

/// Folder names never worth walking, anywhere (lowercase).
const SKIP_ANYWHERE: &[&str] = &["node_modules", "target", "__pycache__", "venv", "appdata"];

/// System folders skipped at the top of a drive (lowercase). Apps are found
/// through the Start Menu instead of Program Files.
const SKIP_AT_DRIVE_ROOT: &[&str] = &[
    "windows",
    "windows.old",
    "program files",
    "program files (x86)",
    "programdata",
    "recovery",
    "perflogs",
    "system volume information",
];

/// A search hit.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

struct Folder {
    path: Box<str>,
    lower: Box<str>,
}

struct Entry {
    folder: u32,
    name: u32,
    lower: u32,
    name_len: u16,
    lower_len: u16,
    is_dir: bool,
}

#[derive(Default)]
struct Store {
    folders: Vec<Folder>,
    entries: Vec<Entry>,
    names: String,
    lowers: String,
}

impl Store {
    fn add_folder(&mut self, path: &Path) -> u32 {
        let path = path.to_string_lossy();
        self.folders.push(Folder {
            lower: path.to_lowercase().into(),
            path: path.into(),
        });
        (self.folders.len() - 1) as u32
    }

    fn add_entry(&mut self, folder: u32, name: &str, lower: &str, is_dir: bool) {
        self.entries.push(Entry {
            folder,
            name: self.names.len() as u32,
            lower: self.lowers.len() as u32,
            name_len: name.len() as u16,
            lower_len: lower.len() as u16,
            is_dir,
        });
        self.names.push_str(name);
        self.lowers.push_str(lower);
    }

    fn name(&self, entry: &Entry) -> &str {
        &self.names[entry.name as usize..][..entry.name_len as usize]
    }

    fn lower(&self, entry: &Entry) -> &str {
        &self.lowers[entry.lower as usize..][..entry.lower_len as usize]
    }

    fn append(&mut self, mut other: Store) {
        let folders = self.folders.len() as u32;
        let names = self.names.len() as u32;
        let lowers = self.lowers.len() as u32;
        for entry in &mut other.entries {
            entry.folder += folders;
            entry.name += names;
            entry.lower += lowers;
        }
        self.folders.append(&mut other.folders);
        self.entries.append(&mut other.entries);
        self.names.push_str(&other.names);
        self.lowers.push_str(&other.lowers);
    }

    fn shrink(&mut self) {
        self.folders.shrink_to_fit();
        self.entries.shrink_to_fit();
        self.names.shrink_to_fit();
        self.lowers.shrink_to_fit();
    }

    fn hit(&self, entry: &Entry) -> FileEntry {
        let name = self.name(entry);
        FileEntry {
            name: name.to_string(),
            path: Path::new(&*self.folders[entry.folder as usize].path).join(name),
            is_dir: entry.is_dir,
        }
    }
}

pub struct FileIndex {
    store: Arc<RwLock<Store>>,
    indexing: Arc<AtomicBool>,
    count: Arc<AtomicUsize>,
}

impl FileIndex {
    pub fn start(roots: Vec<PathBuf>) -> Self {
        let index = Self {
            store: Arc::default(),
            indexing: Arc::new(AtomicBool::new(true)),
            count: Arc::default(),
        };
        let store = index.store.clone();
        let indexing = index.indexing.clone();
        let count = index.count.clone();

        std::thread::spawn(move || {
            let (drives, mut folders): (Vec<_>, Vec<_>) =
                roots.into_iter().partition(|p| p.parent().is_none());
            // Deepest first, so `Desktop` is indexed before the home folder
            // that contains it and the home walk skips it.
            folders.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

            let mut done: HashSet<String> = HashSet::new();
            let plan = folders
                .iter()
                .map(|p| (p, USER_DIR_DEPTH))
                .chain(drives.iter().map(|p| (p, DRIVE_DEPTH)));
            for (root, depth) in plan {
                let mut found = Store::default();
                walk(root, depth, &done, &mut found);
                done.insert(lower(root));

                let mut all = store.write().unwrap();
                all.append(found);
                count.store(all.entries.len(), Ordering::Relaxed);
            }
            store.write().unwrap().shrink();
            indexing.store(false, Ordering::Relaxed);
        });

        index
    }

    pub fn is_indexing(&self) -> bool {
        self.indexing.load(Ordering::Relaxed)
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<FileEntry> {
        let query = query.to_lowercase();
        let words: Vec<&str> = query.split_whitespace().collect();
        let store = self.store.read().unwrap();

        let mut hits: Vec<(usize, &Entry)> = store
            .entries
            .iter()
            .filter_map(|entry| {
                let folder = &store.folders[entry.folder as usize].lower;
                Some((score(store.lower(entry), folder, &query, &words)?, entry))
            })
            .collect();
        hits.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        hits.into_iter()
            .take(limit)
            .map(|(_, entry)| store.hit(entry))
            .collect()
    }
}

fn lower(path: &Path) -> String {
    path.to_string_lossy().to_lowercase()
}

fn walk(dir: &Path, depth: usize, skip: &HashSet<String>, out: &mut Store) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    let at_drive_root = dir.parent().is_none();
    let folder = out.add_folder(dir);

    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let name_lower = name.to_lowercase();
        let skipped_name = SKIP_ANYWHERE.contains(&name_lower.as_str())
            || (at_drive_root && SKIP_AT_DRIVE_ROOT.contains(&name_lower.as_str()));
        if name.starts_with('.') || name.starts_with('$') || skipped_name {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() && skip.contains(&lower(&path)) {
            continue;
        }

        out.add_entry(folder, &name, &name_lower, file_type.is_dir());

        // file_type() does not follow symlinks or junctions, so cycles are impossible.
        if file_type.is_dir() && depth > 0 {
            walk(&path, depth - 1, skip, out);
        }
    }
}

/// Higher is better; None means no match. Shorter names win ties.
fn score(name: &str, folder: &str, query: &str, words: &[&str]) -> Option<usize> {
    let brevity = 100 - name.len().min(100);

    if words.len() > 1 {
        let all = words.iter().all(|w| name.contains(w) || folder.contains(w));
        return all.then_some(800 + brevity);
    }
    if name == query {
        return Some(10_000);
    }
    if let Some(ext) = query.strip_prefix('.')
        && name.rsplit_once('.').is_some_and(|(_, e)| e == ext)
    {
        return Some(3000 + brevity);
    }
    if name.starts_with(query) {
        return Some(5000 + brevity);
    }
    if name.contains(query) {
        return Some(1000 + brevity);
    }

    // Subsequence match, rewarding runs of consecutive characters.
    let mut pending = query.chars().peekable();
    let (mut run, mut best_run, mut matched) = (0, 0, 0);
    for c in name.chars() {
        if pending.peek() == Some(&c) {
            pending.next();
            matched += 1;
            run += 1;
            best_run = best_run.max(run);
        } else {
            run = 0;
        }
    }
    pending
        .peek()
        .is_none()
        .then_some(100 + best_run * 50 + matched * 10 + brevity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rank(path: &str, query: &str) -> Option<usize> {
        let path = path.to_lowercase();
        let (folder, name) = path.rsplit_once('\\').unwrap();
        let query = query.to_lowercase();
        let words: Vec<&str> = query.split_whitespace().collect();
        score(name, folder, &query, &words)
    }

    #[test]
    fn exact_beats_prefix_beats_contains_beats_subsequence() {
        let exact = rank(r"C:\a\notes.md", "notes.md").unwrap();
        let prefix = rank(r"C:\a\notes.md", "notes").unwrap();
        let contains = rank(r"C:\a\mynotes.md", "notes").unwrap();
        let subsequence = rank(r"C:\a\nxoxtxexs.md", "notes").unwrap();
        assert!(exact > prefix && prefix > contains && contains > subsequence);
    }

    #[test]
    fn extension_query_matches_by_extension() {
        assert!(rank(r"C:\a\main.rs", ".rs").is_some());
        assert!(rank(r"C:\a\main.py", ".rs").is_none());
    }

    #[test]
    fn multi_word_query_matches_across_path() {
        assert!(rank(r"C:\work\reflex\readme.md", "reflex readme").is_some());
        assert!(rank(r"C:\work\other\readme.md", "reflex readme").is_none());
    }

    #[test]
    fn no_match_returns_none() {
        assert!(rank(r"C:\a\photo.jpg", "zzz").is_none());
    }

    #[test]
    fn store_resolves_full_paths_after_append() {
        let mut store = Store::default();
        let first = store.add_folder(Path::new(r"C:\other"));
        store.add_entry(first, "Old.TXT", "old.txt", false);

        let mut chunk = Store::default();
        let folder = chunk.add_folder(Path::new(r"C:\docs"));
        chunk.add_entry(folder, "A.txt", "a.txt", false);
        store.append(chunk);

        let hit = store.hit(&store.entries[1]);
        assert_eq!(hit.path, PathBuf::from(r"C:\docs\A.txt"));
        assert_eq!(store.lower(&store.entries[1]), "a.txt");
        assert_eq!(store.name(&store.entries[0]), "Old.TXT");
    }
}
