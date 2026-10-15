//! Result types shared by every source, and the final ordering.

use std::path::PathBuf;

use crate::actions::system::Command;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub subtitle: String,
    pub kind: ResultKind,
    pub score: u32,
    pub action: ResultAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultKind {
    App,
    File,
    Folder,
    Clipboard,
    Calculator,
    Action,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResultAction {
    Launch(PathBuf),
    Open(PathBuf),
    Paste(String),
    Copy(String),
    Run(Command),
}

/// Sorts best first. Each source scores within its own range; these offsets
/// decide how sources interleave.
pub fn rank(results: &mut [SearchResult]) {
    for r in results.iter_mut() {
        let boost = match r.kind {
            ResultKind::Calculator => 2000,
            ResultKind::Action => 1000,
            ResultKind::App => 500,
            _ => 0,
        };
        r.score = r.score.saturating_add(boost);
    }
    results.sort_by_key(|r| std::cmp::Reverse(r.score));
}
