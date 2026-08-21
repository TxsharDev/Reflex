//! Result ranking - combines fuzzy score, frequency, recency, and type.

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub subtitle: String,
    pub kind: ResultKind,
    pub score: u32,
    pub action: ResultAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResultKind {
    App,
    File,
    Folder,
    Clipboard,
    Calculator,
    Action,
}

impl ResultKind {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            ResultKind::App => "App",
            ResultKind::File => "File",
            ResultKind::Folder => "Folder",
            ResultKind::Clipboard => "Clip",
            ResultKind::Calculator => "Calc",
            ResultKind::Action => "Action",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResultAction {
    Launch(std::path::PathBuf),
    OpenFile(std::path::PathBuf),
    OpenFolder(std::path::PathBuf),
    PasteClip(String),
    CopyResult(String),
    SystemAction(String),
}

/// Rank and merge results from different sources.
pub fn rank_results(results: &mut [SearchResult]) {
    // Apps get a bonus
    for r in results.iter_mut() {
        match r.kind {
            ResultKind::App => r.score = r.score.saturating_add(500),
            ResultKind::Calculator => r.score = r.score.saturating_add(2000), // calculator always on top if matched
            ResultKind::Action => r.score = r.score.saturating_add(1000),
            _ => {}
        }
    }

    results.sort_by_key(|b| std::cmp::Reverse(b.score));
}
