//! Small text helpers: truncation with ellipsis, padding, width.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// TextViewport renders each tab as four spaces, independent of column.
pub(crate) const TAB_SPACES: &str = "    ";

/// Match viewport tab geometry before constructing fixed-width styled spans.
pub(crate) fn expand_tabs(text: &str) -> std::borrow::Cow<'_, str> {
    if text.contains('\t') {
        text.replace('\t', TAB_SPACES).into()
    } else {
        text.into()
    }
}

pub fn width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Truncate to `max` display cells, appending `…` when cut.
pub fn truncate(s: &str, max: usize) -> String {
    if width(s) <= max {
        return s.to_owned();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut w = 0;
    for g in s.graphemes(true) {
        let gw = width(g);
        if w + gw > max - 1 {
            break;
        }
        out.push_str(g);
        w += gw;
    }
    out.push('…');
    out
}

/// Show a horizontal window addressed in display cells, never splitting a
/// grapheme. Clipped edges use `…`; a partly hidden wide grapheme leaves padding
/// so subsequent glyphs keep their original cell positions. The result occupies
/// at most `max` cells, making cursor position `column - offset` reusable for
/// rendering and mouse placement. A window past the text returns an empty string.
///
/// ```
/// use junie_tui::ui::text::slice_cells;
/// assert_eq!(slice_cells("ab日本cd", 3, 5), "…本cd");
/// assert_eq!(slice_cells("日本語", 0, 5), "日本…");
/// ```
pub fn slice_cells(s: &str, offset: usize, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let clipped_right = width(s) > offset.saturating_add(max);
    let visible = max - usize::from(clipped_right);
    if visible == 0 {
        return "…".to_owned();
    }
    let end = offset.saturating_add(visible);
    let mut out = String::new();
    let mut col = 0;
    let mut out_width = 0;
    for g in s.graphemes(true) {
        let start = col;
        col += width(g);
        if col <= offset {
            continue;
        }
        if start >= end {
            break;
        }
        if start <= offset && offset > 0 {
            let cells = col.min(end) - offset;
            out.push('…');
            out.push_str(&" ".repeat(cells - 1));
            out_width += cells;
        } else if col <= end {
            out.push_str(g);
            out_width += col - start;
        } else {
            break;
        }
    }
    if clipped_right {
        out.push_str(&" ".repeat(visible.saturating_sub(out_width)));
        out.push('…');
    }
    out
}

/// Truncate keeping both ends: `very_long_identifier_name` → `very_l…_name`.
pub fn truncate_middle(s: &str, max: usize) -> String {
    if width(s) <= max {
        return s.to_owned();
    }
    if max < 5 {
        return truncate(s, max);
    }
    let keep_end = (max - 1) / 3;
    let keep_start = max - 1 - keep_end;
    let gs: Vec<&str> = s.graphemes(true).collect();
    let mut head = String::new();
    let mut w = 0;
    for g in &gs {
        let gw = width(g);
        if w + gw > keep_start {
            break;
        }
        head.push_str(g);
        w += gw;
    }
    let mut tail = String::new();
    let mut w = 0;
    for g in gs.iter().rev() {
        let gw = width(g);
        if w + gw > keep_end {
            break;
        }
        tail.insert_str(0, g);
        w += gw;
    }
    format!("{head}…{tail}")
}

/// `1203338` → `1,203,338`.
pub fn thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// Left-align in `w` cells (pad or truncate).
pub fn fit(s: &str, w: usize) -> String {
    let t = truncate(s, w);
    let pad = w.saturating_sub(width(&t));
    format!("{t}{}", " ".repeat(pad))
}

/// Right-align in `w` cells.
pub fn fit_right(s: &str, w: usize) -> String {
    let t = truncate(s, w);
    let pad = w.saturating_sub(width(&t));
    format!("{}{t}", " ".repeat(pad))
}

/// Word-wrap into lines of at most `w.max(1)` cells. Hard-wraps overlong
/// words; a grapheme wider than the entire line is represented by `…`.
pub fn wrap(s: &str, w: usize) -> Vec<String> {
    let w = w.max(1);
    let mut lines = Vec::new();
    for para in s.split('\n') {
        let mut line = String::new();
        let mut lw = 0;
        for word in para.split(' ') {
            let ww = width(word);
            if lw == 0 {
                if ww <= w {
                    line.push_str(word);
                    lw = ww;
                } else {
                    hard_wrap(word, w, &mut lines, &mut line, &mut lw);
                }
            } else if lw + 1 + ww <= w {
                line.push(' ');
                line.push_str(word);
                lw += 1 + ww;
            } else {
                lines.push(std::mem::take(&mut line));
                lw = 0;
                if ww <= w {
                    line.push_str(word);
                    lw = ww;
                } else {
                    hard_wrap(word, w, &mut lines, &mut line, &mut lw);
                }
            }
        }
        lines.push(line);
    }
    lines
}

fn hard_wrap(word: &str, w: usize, lines: &mut Vec<String>, line: &mut String, lw: &mut usize) {
    for g in word.graphemes(true) {
        let g = if width(g) > w { "…" } else { g };
        let gw = width(g);
        if *lw + gw > w {
            lines.push(std::mem::take(line));
            *lw = 0;
        }
        line.push_str(g);
        *lw += gw;
    }
}

/// Search text retains source graphemes when lowercase expands a scalar.
struct SearchText {
    text: String,
    origins: Vec<(Range<usize>, Range<usize>)>,
}

impl SearchText {
    fn new(source: &str, lowercase: bool) -> Self {
        // Lowercase the whole string so contextual forms (Greek final sigma)
        // retain Rust's string-casing semantics. Context changes the scalar's
        // form; scalar expansion counts still identify its source grapheme.
        let text = if lowercase {
            source.to_lowercase()
        } else {
            source.to_owned()
        };
        let mut chars = text.char_indices().peekable();
        let mut origins = Vec::new();
        for (start, grapheme) in source.grapheme_indices(true) {
            let mapped_start = chars.peek().map_or(text.len(), |(offset, _)| *offset);
            let count = if lowercase {
                grapheme.chars().map(|c| c.to_lowercase().count()).sum()
            } else {
                grapheme.chars().count()
            };
            for _ in 0..count {
                chars.next();
            }
            let mapped_end = chars.peek().map_or(text.len(), |(offset, _)| *offset);
            origins.push((mapped_start..mapped_end, start..start + grapheme.len()));
        }
        Self { text, origins }
    }

    fn original_range(&self, range: Range<usize>) -> Range<usize> {
        let first = self
            .origins
            .partition_point(|(mapped, _)| mapped.end <= range.start);
        let end = self
            .origins
            .partition_point(|(mapped, _)| mapped.start < range.end);
        self.origins[first].1.start..self.origins[end - 1].1.end
    }

    fn matched_offsets(&self, range: Range<usize>) -> Vec<usize> {
        self.origins
            .iter()
            .filter(|(mapped, _)| mapped.start < range.end && range.start < mapped.end)
            .map(|(_, original)| original.start)
            .collect()
    }
}

/// Match ranges always refer to complete graphemes in the original document.
pub fn find_ranges(text: &str, needle: &str, case_sensitive: bool) -> Vec<Range<usize>> {
    if needle.is_empty() {
        return Vec::new();
    }
    let hay = SearchText::new(text, !case_sensitive);
    let needle = if case_sensitive {
        needle.to_owned()
    } else {
        needle.to_lowercase()
    };
    let mut ranges: Vec<_> = hay
        .text
        .match_indices(&needle)
        .map(|(start, matched)| hay.original_range(start..start + matched.len()))
        .collect();
    // Multiple scalar matches may belong to one combining/emoji grapheme.
    ranges.dedup();
    ranges
}

/// Fuzzy match `word` against `label`: a prefix wins, then a substring
/// (bonus on a `_`/`.` boundary), then a subsequence. Returns a penalty
/// (lower is better) and unique byte offsets of matched original grapheme
/// starts so a list can bold them. Matching compares Unicode scalars after
/// lowercasing; offsets never address transformed text or split UTF-8.
pub fn fuzzy(label: &str, word: &str) -> Option<(u32, Vec<usize>)> {
    if word.is_empty() {
        return Some((0, vec![]));
    }
    let l = SearchText::new(label, true);
    let w = word.to_lowercase();
    if l.text.starts_with(&w) {
        return Some((0, l.matched_offsets(0..w.len())));
    }
    if let Some(p) = l.text.find(&w) {
        let boundary = p == 0 || matches!(l.text.as_bytes()[p - 1], b'_' | b'.');
        return Some((
            if boundary { 10 } else { 30 },
            l.matched_offsets(p..p + w.len()),
        ));
    }
    let mut matched = Vec::new();
    let mut chars = l.text.char_indices();
    for wc in w.chars() {
        let (offset, ch) = chars.find(|(_, ch)| *ch == wc)?;
        let original = l.original_range(offset..offset + ch.len_utf8()).start;
        if matched.last() != Some(&original) {
            matched.push(original);
        }
    }
    Some((60 + (matched.last().copied().unwrap_or(0) as u32), matched))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_windows_preserve_display_positions_and_whole_graphemes() {
        assert_eq!(slice_cells("ab日本cd", 3, 5), "…本cd");
        assert_eq!(slice_cells("ab日本cd", 2, 5), "… 本…");
        assert_eq!(slice_cells("日本語", 0, 5), "日本…");
        assert_eq!(slice_cells("a👩‍💻e\u{301}z", 1, 5), "… e\u{301}z");
        assert_eq!(slice_cells("abc", 3, 5), "");
        assert_eq!(slice_cells("日本", 1, 1), "…");
        for text in ["日本語abc", "👩‍💻🙂e\u{301}終", "abc", ""] {
            for offset in 0..=width(text) + 1 {
                for cells in 0..12 {
                    assert!(width(&slice_cells(text, offset, cells)) <= cells);
                }
            }
        }
    }

    #[test]
    fn truncates_with_ellipsis() {
        assert_eq!(truncate("hello world", 5), "hell…");
        assert_eq!(truncate("hi", 5), "hi");
        assert_eq!(fit("hi", 4), "hi  ");
        assert_eq!(fit_right("hi", 4), "  hi");
    }

    #[test]
    fn middle_truncation_and_thousands() {
        assert_eq!(
            truncate_middle("very_long_identifier_name", 12),
            "very_lon…ame"
        );
        assert_eq!(truncate_middle("short", 12), "short");
        assert_eq!(thousands(1_203_338), "1,203,338");
        assert_eq!(thousands(999), "999");
    }

    #[test]
    fn wraps_words_and_hard_wraps_long_tokens() {
        assert_eq!(wrap("aa bb cc", 5), vec!["aa bb", "cc"]);
        assert_eq!(wrap("abcdefgh", 3), vec!["abc", "def", "gh"]);
        assert_eq!(wrap("a\nb", 10), vec!["a", "b"]);
        assert_eq!(wrap("", 10), vec![""]);
    }

    #[test]
    fn wrapping_never_splits_or_overflows_a_wide_grapheme() {
        assert_eq!(wrap("日👩‍💻e\u{301}", 1), vec!["…", "…", "e\u{301}"]);
        assert_eq!(wrap("日👩‍💻e\u{301}", 2), vec!["日", "👩‍💻", "e\u{301}"]);
        for w in 0..8 {
            let lines = wrap("日本 👩‍💻 cafe\u{301}", w);
            assert!(lines.iter().all(|line| width(line) <= w.max(1)));
        }
    }

    #[test]
    fn fuzzy_returns_original_grapheme_offsets() {
        assert_eq!(fuzzy("İé", "é"), Some((30, vec![2])));
        assert_eq!(fuzzy("éclair", "é"), Some((0, vec![0])));
        assert_eq!(fuzzy("a\u{301}b", "\u{301}"), Some((30, vec![0])));
        assert_eq!(fuzzy("🙂_界", "🙂界"), Some((65, vec![0, 5])));
    }

    #[test]
    fn fuzzy_subsequence_matches_scalars_not_utf8_bytes() {
        assert_eq!(fuzzy("Ã©", "é"), None);
        assert_eq!(fuzzy("ΟΣ", "ος"), Some((0, vec![0, 2])));
    }

    #[test]
    fn find_preserves_smart_case_and_whole_source_graphemes() {
        assert_eq!(find_ranges("Aa", "A", true), vec![0..1]);
        assert_eq!(find_ranges("Aa", "a", false), vec![0..1, 1..2]);
        assert_eq!(find_ranges("İ", "i", false), vec![0..2]);
        assert_eq!(find_ranges("a\u{301}\u{301}", "\u{301}", true), vec![0..5]);
        assert!(find_ranges("text", "", false).is_empty());
        assert!(find_ranges("", "a", false).is_empty());
    }

    #[test]
    fn fuzzy_preserves_ascii_ranking() {
        assert_eq!(fuzzy("alpha", "al"), Some((0, vec![0, 1])));
        assert_eq!(fuzzy("a_beta", "be"), Some((10, vec![2, 3])));
        assert_eq!(fuzzy("alpha", "ph"), Some((30, vec![2, 3])));
        assert_eq!(fuzzy("alpha", "aa"), Some((64, vec![0, 4])));
    }
}
