//! Fuzzy matching over graphemes of the original label
//! (`COMPONENT_ARCHITECTURE.md` §22.2 item 4, §20.10 item 7f).

use core::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

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
    /// Whether the byte before a substring match earns the boundary bonus.
    /// ASCII separators are single bytes untouched by case folding, so a
    /// byte test on the lowercased text is exact (oracle `ui/text.rs`).
    fn precedes(self, byte: u8) -> bool {
        matches!(byte, b'_' | b'.') || (self == Self::Word && matches!(byte, b' ' | b'-'))
    }
}

/// Search text retains source graphemes when lowercase expands a scalar
/// (oracle `ui/text.rs::SearchText`): the whole string is lowercased once so
/// contextual forms (Greek final sigma) keep Rust's string-casing semantics,
/// scalar expansion counts still identify each source grapheme, and matches
/// project back to original-grapheme ordinals plus source byte offsets.
struct SearchText {
    text: String,
    origins: Vec<Origin>,
}

/// One source grapheme's span in the lowercased text plus its original
/// grapheme ordinal and byte offset.
struct Origin {
    mapped: Range<usize>,
    ordinal: usize,
    byte: usize,
}

impl SearchText {
    fn new(source: &str, lowercase: bool) -> Self {
        let text = if lowercase {
            source.to_lowercase()
        } else {
            source.to_owned()
        };
        let mut chars = text.char_indices().peekable();
        let mut origins = Vec::new();
        for (ordinal, (byte, grapheme)) in source.grapheme_indices(true).enumerate() {
            let mapped_start = chars.peek().map_or(text.len(), |(offset, _)| *offset);
            let count: usize = if lowercase {
                grapheme.chars().map(|c| c.to_lowercase().count()).sum()
            } else {
                grapheme.chars().count()
            };
            for _ in 0..count {
                chars.next();
            }
            let mapped_end = chars.peek().map_or(text.len(), |(offset, _)| *offset);
            origins.push(Origin {
                mapped: mapped_start..mapped_end,
                ordinal,
                byte,
            });
        }
        Self { text, origins }
    }

    /// Original-grapheme ordinals overlapped by a lowercased-text range.
    /// Multiple scalar matches may belong to one combining/emoji grapheme;
    /// each ordinal is reported once, in order.
    fn matched_ordinals(&self, range: Range<usize>) -> Vec<usize> {
        let mut ordinals = Vec::new();
        for origin in &self.origins {
            if origin.mapped.start < range.end
                && range.start < origin.mapped.end
                && ordinals.last() != Some(&origin.ordinal)
            {
                ordinals.push(origin.ordinal);
            }
        }
        ordinals
    }

    /// The origin holding a lowercased-text scalar offset.
    fn origin_at(&self, offset: usize) -> Option<&Origin> {
        self.origins
            .iter()
            .find(|o| o.mapped.start <= offset && offset < o.mapped.end)
    }
}

/// Match `word` against `label`: a prefix wins (0), then a substring on a
/// boundary (10), then any substring (30), then a subsequence (60 + the
/// source byte offset of the last matched grapheme start). Returns the
/// penalty (lower is better) and the **grapheme ordinals in the original
/// label** that matched, so a list can bold them while walking the label's
/// graphemes.
///
/// Matching compares Unicode scalars after lowercasing the whole string
/// (so Greek final sigma and expanding case folds match); offsets never
/// address transformed text or split UTF-8 (BF05).
///
/// **Allocates per call** (the lowercased fold, the origin map and the match
/// ordinals). That is fine for a Slice-3 primitive and a dialog-sized
/// candidate set; a 100 k-item `Picker` filter would make 300 000
/// allocations per keystroke, so 4F must either take a scratch buffer or
/// filter incrementally (MI-10). Recorded here so it is not discovered
/// under a profiler.
pub fn fuzzy(label: &str, word: &str) -> Option<(u32, Vec<usize>)> {
    fuzzy_with_boundary(label, word, FuzzyBoundary::default())
}

/// Match with an explicit boundary-bonus policy.
///
/// Case folding, prefix/substring/subsequence matching and original-label
/// grapheme ordinals are identical to [`fuzzy`]; only the boundary bonus
/// varies. [`FuzzyBoundary::Identifier`] is the exact oracle boundary lane
/// (underscore/dot); source-qualified callers select it explicitly rather
/// than changing the generic default.
pub fn fuzzy_with_boundary(
    label: &str,
    word: &str,
    boundary: FuzzyBoundary,
) -> Option<(u32, Vec<usize>)> {
    if word.is_empty() {
        return Some((0, Vec::new()));
    }
    let l = SearchText::new(label, true);
    let w = word.to_lowercase();
    if l.text.starts_with(&w) {
        return Some((0, l.matched_ordinals(0..w.len())));
    }
    if let Some(p) = l.text.find(&w) {
        let at_boundary = p == 0
            || l.text
                .as_bytes()
                .get(p.saturating_sub(1))
                .is_some_and(|b| boundary.precedes(*b));
        return Some((
            if at_boundary { 10 } else { 30 },
            l.matched_ordinals(p..p.saturating_add(w.len())),
        ));
    }
    subsequence(&l, &w)
}

fn subsequence(l: &SearchText, w: &str) -> Option<(u32, Vec<usize>)> {
    let mut matched = Vec::with_capacity(w.chars().count());
    let mut last_byte = 0usize;
    let mut chars = l.text.char_indices();
    for wc in w.chars() {
        let (offset, _) = chars.find(|(_, ch)| *ch == wc)?;
        if let Some(origin) = l.origin_at(offset) {
            if matched.last() != Some(&origin.ordinal) {
                matched.push(origin.ordinal);
            }
            last_byte = origin.byte;
        }
    }
    Some((60u32.saturating_add(last_byte as u32), matched))
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
        let fam = "👨‍👩‍👧‍👦";
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
}
