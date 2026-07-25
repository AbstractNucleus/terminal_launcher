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

    /// Returns `(entry_index, match_positions)` sorted by score (best first).
    /// Positions are char indices into the haystack. Empty query returns every
    /// index with an empty position list, preserving input order.
    pub fn filter(&mut self, query: &str, items: &[String]) -> Vec<(usize, Vec<u32>)> {
        if query.is_empty() {
            return (0..items.len()).map(|idx| (idx, Vec::new())).collect();
        }

        let atom = Atom::new(
            query,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut buf = Vec::new();
        let mut scored: Vec<(usize, u16, Vec<u32>)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                let haystack = Utf32Str::new(item, &mut buf);
                let mut indices = Vec::new();
                let score = atom.indices(haystack, &mut self.matcher, &mut indices)?;
                indices.sort_unstable();
                indices.dedup();
                Some((idx, score, indices))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored
            .into_iter()
            .map(|(idx, _, indices)| (idx, indices))
            .collect()
    }
}

/// Split match positions from a `name + " " + detail` haystack into name and
/// detail char indices. The joining space (at `name_chars`) is dropped.
pub fn split_match_positions(positions: &[u32], name_chars: usize) -> (Vec<u32>, Vec<u32>) {
    let boundary = name_chars as u32;
    let mut name_pos = Vec::new();
    let mut detail_pos = Vec::new();
    for &pos in positions {
        if pos < boundary {
            name_pos.push(pos);
        } else if pos > boundary {
            detail_pos.push(pos - boundary - 1);
        }
        // pos == boundary is the joining space — drop it
    }
    (name_pos, detail_pos)
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
        assert_eq!(
            result.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert!(result.iter().all(|(_, pos)| pos.is_empty()));
    }

    #[test]
    fn exact_match_returns_item() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("Backend", &items());
        assert!(result.iter().any(|(i, _)| *i == 1));
    }

    #[test]
    fn partial_match() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("dev", &items());
        assert!(!result.is_empty());
        assert!(result.iter().any(|(i, _)| *i == 3));
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
        assert!(result.iter().any(|(i, _)| *i == 2));
    }

    #[test]
    fn fuzzy_match_returns_positions_for_prd() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("prd", &items());
        let (_, positions) = result.iter().find(|(i, _)| *i == 2).unwrap();
        // "Prod Server" — P, r, d at char indices 0, 1, 3
        assert_eq!(positions, &vec![0, 1, 3]);
    }

    #[test]
    fn case_insensitive() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("MY APP", &items());
        assert!(result.iter().any(|(i, _)| *i == 0));
    }

    #[test]
    fn split_uses_char_boundary_not_bytes() {
        // "Ünicode" is 7 chars but 8 bytes (Ü is 2 bytes in UTF-8)
        let name = "Ünicode";
        assert_eq!(name.chars().count(), 7);
        assert_eq!(name.len(), 8);
        let detail = "~/projects/ünicode";
        let haystack = format!("{} {}", name, detail);
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("üni", &[haystack]);
        assert_eq!(result.len(), 1);
        let (_, positions) = &result[0];
        let (name_pos, detail_pos) = split_match_positions(positions, name.chars().count());
        // At least the leading Ü/ü in the name should highlight
        assert!(
            !name_pos.is_empty() || !detail_pos.is_empty(),
            "expected some match positions, got name={:?} detail={:?}",
            name_pos,
            detail_pos
        );
        // Every name position must be within the name char count
        assert!(name_pos.iter().all(|&p| (p as usize) < name.chars().count()));
        // Every detail position must be within the detail char count
        assert!(detail_pos
            .iter()
            .all(|&p| (p as usize) < detail.chars().count()));
    }

    #[test]
    fn split_drops_joining_space() {
        let (name_pos, detail_pos) = split_match_positions(&[0, 7, 8, 10], 7);
        // boundary=7 dropped; 8 -> detail 0; 10 -> detail 2
        assert_eq!(name_pos, vec![0]);
        assert_eq!(detail_pos, vec![0, 2]);
    }
}
