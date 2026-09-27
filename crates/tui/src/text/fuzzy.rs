//! Fuzzy matching over graphemes of the original label
//! (`COMPONENT_ARCHITECTURE.md` §22.2 item 4, §20.10 item 7f).

use core::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Case-insensitive grapheme equality without allocating.
struct Folded<'a> {
    text: String,
    spans: Vec<(Range<usize>, usize)>,
    _source: core::marker::PhantomData<&'a str>,
}

impl<'a> Folded<'a> {
    fn new(source: &'a str) -> Self {
        let text = source.to_lowercase();
        let mut folded = text.char_indices().peekable();
        let mut spans = Vec::new();
        for (ordinal, (_, grapheme)) in source.grapheme_indices(true).enumerate() {
            let start = folded.peek().map_or(text.len(), |(byte, _)| *byte);
            let count: usize = grapheme.chars().map(|c| c.to_lowercase().count()).sum();
            for _ in 0..count {
                folded.next();
            }
            let end = folded.peek().map_or(text.len(), |(byte, _)| *byte);
            spans.push((start..end, ordinal));
        }
        Self {
            text,
            spans,
            _source: core::marker::PhantomData,
        }
    }

    fn ordinals(&self, range: Range<usize>) -> Vec<usize> {
        let mut result = Vec::new();
        for (span, ordinal) in &self.spans {
            if span.start < range.end && range.start < span.end && result.last() != Some(ordinal) {
                result.push(*ordinal);
            }
        }
        result
    }

    fn complete_clusters(&self, range: &Range<usize>) -> bool {
        self.spans.iter().any(|(span, _)| span.start == range.start)
            && self.spans.iter().any(|(span, _)| span.end == range.end)
    }
}

/// Which separators give a substring the word-boundary ranking bonus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FuzzyBoundary {
    /// Underscore, dot, space and hyphen; the default collection-search policy.
    #[default]
    Word,
    /// Underscore and dot only, for identifier-oriented completion/ranking.
    Identifier,
}
impl FuzzyBoundary {
    fn contains(self, grapheme: &str) -> bool {
        matches!(grapheme, "_" | ".") || (self == Self::Word && matches!(grapheme, " " | "-"))
    }
}

/// Match `word` against `label`: a prefix wins (0), then a substring on a
/// `_`/`.`/` `/`-` boundary (10), then any substring (30), then a subsequence
/// (60 + position of the last match). Returns the penalty (lower is better)
/// and the **grapheme ordinals in the original label** that matched, so a
/// list can bold them while walking the label's graphemes.
///
/// **Allocates three `Vec`s per call** (the grapheme index, the lowercase
/// fold and the match ordinals). That is fine for a Slice-3 primitive and a
/// dialog-sized candidate set; a 100 k-item `Picker` filter would make
/// 300 000 allocations per keystroke, so 4F must either take a scratch buffer
/// or filter incrementally (MI-10). Recorded here so it is not discovered
/// under a profiler.
pub fn fuzzy(label: &str, word: &str) -> Option<(u32, Vec<usize>)> {
    fuzzy_with_boundary(label, word, FuzzyBoundary::default())
}

/// Match with an explicit boundary-bonus policy.
///
/// Case folding, prefix/substring/subsequence matching and original-label
/// grapheme ordinals are identical to [`fuzzy`]; only the boundary bonus varies.
pub fn fuzzy_with_boundary(
    label: &str,
    word: &str,
    boundary: FuzzyBoundary,
) -> Option<(u32, Vec<usize>)> {
    if word.is_empty() {
        return Some((0, Vec::new()));
    }
    let label_folded = Folded::new(label);
    let word_folded = word.to_lowercase();
    if label_folded.text.starts_with(&word_folded) {
        if !label_folded.complete_clusters(&(0..word_folded.len())) {
            return None;
        }
        return Some((0, label_folded.ordinals(0..word_folded.len())));
    }
    if let Some(start) = label_folded.text.find(&word_folded) {
        let range = start..start.saturating_add(word_folded.len());
        if !label_folded.complete_clusters(&range) {
            return None;
        }
        let ordinal_start = label_folded
            .spans
            .iter()
            .find(|(span, _)| span.start <= start && start < span.end)
            .map_or(0, |(_, i)| *i);
        let at_boundary = ordinal_start == 0
            || label
                .graphemes(true)
                .nth(ordinal_start.saturating_sub(1))
                .is_some_and(|g| boundary.contains(g));
        return Some((
            if at_boundary { 10 } else { 30 },
            label_folded.ordinals(range),
        ));
    }
    subsequence(&label_folded, &word_folded)
}

fn subsequence(label: &Folded<'_>, word: &str) -> Option<(u32, Vec<usize>)> {
    let mut matched = Vec::new();
    let mut cursor = 0;
    let mut last = 0;
    for wanted in word.chars() {
        let suffix = label.text.get(cursor..)?;
        let (byte, _) = suffix.char_indices().find(|(_, c)| *c == wanted)?;
        cursor = cursor
            .saturating_add(byte)
            .saturating_add(wanted.len_utf8());
        let ordinal = label
            .spans
            .iter()
            .find(|(span, _)| {
                span.start <= cursor.saturating_sub(wanted.len_utf8())
                    && cursor.saturating_sub(wanted.len_utf8()) < span.end
            })
            .map(|(_, i)| *i)?;
        if matched.last() != Some(&ordinal) {
            matched.push(ordinal);
        }
        last = ordinal;
    }
    Some((60u32.saturating_add(last as u32), matched))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_returns_grapheme_indices_into_the_original_label() {
        // `İ` lowercases to two code points, so byte offsets into a lowercased
        // copy would drift; grapheme ordinals of the original do not
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
    fn fuzzy_handles_contextual_case_and_never_matches_inside_a_grapheme() {
        assert_eq!(fuzzy("ΟΣ", "ος"), Some((0, vec![0, 1])));
        assert_eq!(fuzzy("İ", "i"), None);
        assert_eq!(fuzzy("e\u{301}", "e"), None);
    }
}
