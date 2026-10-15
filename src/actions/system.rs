//! Built-in commands offered in the palette.

use crate::search::ranker::{ResultAction, ResultKind, SearchResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Sleep,
    Lock,
    Restart,
    Shutdown,
    EmptyRecycleBin,
    CopyIp,
    Terminal,
    Screenshot,
    Quit,
}

impl Command {
    const ALL: [Command; 9] = [
        Command::Sleep,
        Command::Lock,
        Command::Restart,
        Command::Shutdown,
        Command::EmptyRecycleBin,
        Command::CopyIp,
        Command::Terminal,
        Command::Screenshot,
        Command::Quit,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Command::Sleep => "Sleep",
            Command::Lock => "Lock",
            Command::Restart => "Restart",
            Command::Shutdown => "Shut down",
            Command::EmptyRecycleBin => "Empty Recycle Bin",
            Command::CopyIp => "Copy IP address",
            Command::Terminal => "Open terminal",
            Command::Screenshot => "Screenshot",
            Command::Quit => "Quit Reflex",
        }
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            Command::Sleep => &["sleep", "suspend"],
            Command::Lock => &["lock"],
            Command::Restart => &["restart", "reboot"],
            Command::Shutdown => &["shutdown", "shut", "power", "off"],
            Command::EmptyRecycleBin => &["empty", "recycle", "bin", "trash"],
            Command::CopyIp => &["ip", "address", "network"],
            Command::Terminal => &["terminal", "cmd", "console", "shell", "powershell"],
            Command::Screenshot => &["screenshot", "snip", "capture"],
            Command::Quit => &["quit", "exit", "reflex"],
        }
    }

    /// Commands that end the session need a second Enter.
    pub fn needs_confirmation(self) -> bool {
        matches!(self, Command::Restart | Command::Shutdown)
    }
}

/// Commands whose title or a keyword starts with every word of the query.
pub fn search(query: &str) -> Vec<SearchResult> {
    let query = query.to_lowercase();
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }

    Command::ALL
        .into_iter()
        .filter_map(|command| {
            let title = command.title().to_lowercase();
            let prefixes = || {
                title
                    .split_whitespace()
                    .chain(command.keywords().iter().copied())
            };
            if !words.iter().all(|w| prefixes().any(|p| p.starts_with(w))) {
                return None;
            }
            let score = if title.starts_with(&query) {
                3000
            } else {
                2000
            };
            Some(SearchResult {
                title: command.title().to_string(),
                subtitle: String::new(),
                kind: ResultKind::Action,
                score,
                action: ResultAction::Run(command),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(query: &str) -> Vec<String> {
        search(query).into_iter().map(|r| r.title).collect()
    }

    #[test]
    fn matches_title_and_keyword_prefixes() {
        assert_eq!(titles("reboot"), ["Restart"]);
        assert_eq!(titles("shut"), ["Shut down"]);
        assert!(titles("trash").contains(&"Empty Recycle Bin".to_string()));
    }

    #[test]
    fn single_letters_inside_words_do_not_match() {
        assert!(!titles("p").contains(&"Sleep".to_string()));
    }

    #[test]
    fn only_restart_and_shutdown_need_confirmation() {
        let confirmed: Vec<_> = Command::ALL
            .into_iter()
            .filter(|c| c.needs_confirmation())
            .collect();
        assert_eq!(confirmed, [Command::Restart, Command::Shutdown]);
    }
}
