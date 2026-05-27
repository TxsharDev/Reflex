//! Fuzzy matching using nucleo for fast, high-quality results.

use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// Score a candidate against a pattern. Returns None if no match.
    pub fn score(&mut self, pattern: &str, candidate: &str) -> Option<u32> {
        if pattern.is_empty() { return None; }

        let atom = Atom::new(
            pattern,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut buf = Vec::new();
        let haystack = Utf32Str::new(candidate, &mut buf);
        atom.score(haystack, &mut self.matcher).map(|s| s as u32)
    }
}
