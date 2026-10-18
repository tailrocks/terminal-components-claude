use unicode_segmentation::UnicodeSegmentation;

fn eq_fold(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    a.chars()
        .flat_map(char::to_lowercase)
        .map(|c| if c == 'ς' { 'σ' } else { c })
        .eq(b
            .chars()
            .flat_map(char::to_lowercase)
            .map(|c| if c == 'ς' { 'σ' } else { c }))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FuzzyBoundary {
    #[default]
    Word,
    Identifier,
}
impl FuzzyBoundary {
    fn contains(self, grapheme: &str) -> bool {
        matches!(grapheme, "_" | ".") || (self == Self::Word && matches!(grapheme, " " | "-"))
    }
}

pub fn fuzzy(label: &str, word: &str) -> Option<(u32, Vec<usize>)> {
    fuzzy_with_boundary(label, word, FuzzyBoundary::default())
}

pub fn fuzzy_with_boundary(
    label: &str,
    word: &str,
    boundary: FuzzyBoundary,
) -> Option<(u32, Vec<usize>)> {
    if word.is_empty() {
        return Some((0, Vec::new()));
    }
    let lg: Vec<&str> = label.graphemes(true).collect();
    let wg: Vec<&str> = word.graphemes(true).collect();
    if wg.len() > lg.len() {
        return subsequence(&lg, &wg);
    }

    let last_start = lg.len().saturating_sub(wg.len());
    for start in 0..=last_start {
        let hit = wg.iter().enumerate().all(|(k, w)| {
            lg.get(start.saturating_add(k))
                .is_some_and(|l| eq_fold(l, w))
        });
        if hit {
            let idx: Vec<usize> = (start..start.saturating_add(wg.len())).collect();
            if start == 0 {
                return Some((0, idx));
            }
            let at_boundary = lg
                .get(start.saturating_sub(1))
                .is_some_and(|grapheme| boundary.contains(grapheme));
            return Some((if at_boundary { 10 } else { 30 }, idx));
        }
    }
    subsequence(&lg, &wg)
}

fn subsequence(lg: &[&str], wg: &[&str]) -> Option<(u32, Vec<usize>)> {
    let mut matched = Vec::with_capacity(wg.len());
    let mut li = 0usize;
    for w in wg {
        loop {
            let l = lg.get(li)?;
            if eq_fold(l, w) {
                matched.push(li);
                li = li.saturating_add(1);
                break;
            }
            li = li.saturating_add(1);
        }
    }
    let last = matched.last().copied().unwrap_or(0) as u32;
    Some((60u32.saturating_add(last), matched))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_returns_grapheme_indices_into_the_original_label() {
        let label = "İstanbul_Ünïted";
        let (score, idx) = fuzzy(label, "ün").unwrap_or((u32::MAX, Vec::new()));
        assert_eq!(score, 10);
        assert_eq!(idx, vec![9, 10]);
        let graphemes: Vec<&str> = label.graphemes(true).collect();
        assert_eq!(graphemes.get(9).copied(), Some("Ü"));
        let fam = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}";
        let (_, idx) = fuzzy(&format!("{fam}x"), "x").unwrap_or((0, Vec::new()));
        assert_eq!(idx, vec![1]);
    }

    #[test]
    fn fuzzy_ranks_prefix_before_boundary_before_substring_before_subsequence() {
        let p = fuzzy("orders", "ord").map(|m| m.0);
        let b = fuzzy("my_orders", "ord").map(|m| m.0);
        let s = fuzzy("reorders", "ord").map(|m| m.0);
        let q = fuzzy("o r d", "ord").map(|m| m.0);
        assert_eq!(p, Some(0));
        assert_eq!(b, Some(10));
        assert_eq!(s, Some(30));
        assert!(q.is_some_and(|v| v >= 60));
        assert!(p < b && b < s && s < q);
        assert_eq!(fuzzy("abc", "z"), None);
        assert_eq!(fuzzy("abc", ""), Some((0, Vec::new())));
        assert_eq!(fuzzy("ab", "abc"), None);
    }

    #[test]
    fn fuzzy_matches_only_whole_graphemes_and_folds_sigma_variants() {
        assert_eq!(fuzzy("İ", "i"), None);
        assert_eq!(fuzzy("e\u{301}", "e"), None);
        assert_eq!(fuzzy("İx", "ix"), None);
        assert_eq!(fuzzy("İi", "i"), Some((30, vec![1])));
        assert_eq!(fuzzy("ΟΣ", "οσ"), Some((0, vec![0, 1])));
        assert_eq!(fuzzy("ΟΣ", "ος"), Some((0, vec![0, 1])));
    }
}
