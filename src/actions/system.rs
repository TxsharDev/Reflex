//! Built-in system actions that can be triggered from the command palette.

use crate::search::ranker::{SearchResult, ResultKind, ResultAction};

/// All available system actions.
pub fn all_actions() -> Vec<SystemAction> {
    vec![
        SystemAction { name: "Sleep", keywords: "sleep suspend", action: "sleep" },
        SystemAction { name: "Shutdown", keywords: "shutdown power off", action: "shutdown" },
        SystemAction { name: "Restart", keywords: "restart reboot", action: "restart" },
        SystemAction { name: "Lock", keywords: "lock screen", action: "lock" },
        SystemAction { name: "Empty Recycle Bin", keywords: "empty trash recycle bin clear", action: "emptybin" },
        SystemAction { name: "IP Address", keywords: "ip address network", action: "ip" },
        SystemAction { name: "Open Terminal", keywords: "terminal cmd powershell console", action: "terminal" },
        SystemAction { name: "Screenshot", keywords: "screenshot snip capture screen", action: "screenshot" },
    ]
}

pub struct SystemAction {
    pub name: &'static str,
    pub keywords: &'static str,
    pub action: &'static str,
}

/// Search system actions.
pub fn search_actions(query: &str) -> Vec<SearchResult> {
    let q = query.to_lowercase();
    let actions = all_actions();

    actions
        .iter()
        .filter(|a| {
            a.name.to_lowercase().contains(&q) || a.keywords.contains(&q)
        })
        .map(|a| {
            let score = if a.name.to_lowercase().starts_with(&q) { 3000 } else { 2000 };
            SearchResult {
                title: a.name.to_string(),
                subtitle: "System action".to_string(),
                kind: ResultKind::Action,
                score,
                action: ResultAction::SystemAction(a.action.to_string()),
            }
        })
        .collect()
}
