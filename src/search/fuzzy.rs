//! Fuzzy matching for app names, backed by nucleo.

use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};

pub struct FuzzyMatcher {
    matcher: Matcher,
    buf: Vec<char>,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
            buf: Vec::new(),
        }
    }

    /// Match quality of `pattern` against `candidate`, or None.
    pub fn score(&mut self, pattern: &str, candidate: &str) -> Option<u32> {
        if pattern.is_empty() {
            return None;
        }
        let atom = Atom::new(
            pattern,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );
        let haystack = Utf32Str::new(candidate, &mut self.buf);
        atom.score(haystack, &mut self.matcher).map(u32::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_subsequences() {
        let mut fm = FuzzyMatcher::new();
        assert!(fm.score("vsc", "Visual Studio Code").is_some());
        assert!(fm.score("xyz", "Visual Studio Code").is_none());
    }

    #[test]
    fn prefers_prefix_matches() {
        let mut fm = FuzzyMatcher::new();
        assert!(fm.score("chr", "Chrome").unwrap() > fm.score("hrm", "Chrome").unwrap());
    }
}
