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

    /// Returns indices of matching items, sorted by match score (best first).
    pub fn filter(&mut self, query: &str, items: &[String]) -> Vec<usize> {
        if query.is_empty() {
            return (0..items.len()).collect();
        }

        let atom = Atom::new(
            query,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut buf = Vec::new();
        let mut scored: Vec<(usize, u16)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                let haystack = Utf32Str::new(item, &mut buf);
                let score = atom.score(haystack, &mut self.matcher)?;
                Some((idx, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(idx, _)| idx).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<String> {
        vec![
            "My App".to_string(),
            "Backend".to_string(),
            "Prod Server".to_string(),
            "Dev Box".to_string(),
        ]
    }

    #[test]
    fn empty_query_returns_all_in_order() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("", &items());
        assert_eq!(result, vec![0, 1, 2, 3]);
    }

    #[test]
    fn exact_match_returns_item() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("Backend", &items());
        assert!(result.contains(&1));
    }

    #[test]
    fn partial_match() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("dev", &items());
        assert!(!result.is_empty());
        assert!(result.contains(&3));
    }

    #[test]
    fn no_match_returns_empty() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("zzzzzzz", &items());
        assert!(result.is_empty());
    }

    #[test]
    fn fuzzy_match_skips_characters() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("prd", &items());
        assert!(result.contains(&2));
    }

    #[test]
    fn case_insensitive() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("MY APP", &items());
        assert!(result.contains(&0));
    }
}
