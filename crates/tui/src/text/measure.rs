//! The ONE width function and the non-render text helpers
//! (`COMPONENT_ARCHITECTURE.md` §22.2 items 2–4, R‑1).
//!
//! [`width`] measures exactly the columns `Buffer::set_stringn` consumes:
//! graphemes are walked with `unicode-segmentation`, graphemes containing a
//! control character are skipped, and each surviving grapheme is measured
//! with `ratatui_core::buffer::CellWidth` (`unicode-width` plus one column
//! per halfwidth katakana sound mark). No other file may name
//! `unicode_width`.

use ratatui_core::buffer::CellWidth;
use unicode_segmentation::UnicodeSegmentation;

/// Graphemes of `s` with their byte offsets.
pub(crate) fn graphemes(s: &str) -> impl Iterator<Item = (usize, &str)> {
    s.grapheme_indices(true)
}

/// Columns one grapheme occupies as painted: `0` for a grapheme carrying a
/// control character, else `CellWidth::cell_width`.
pub(crate) fn grapheme_width(g: &str) -> u16 {
    if g.contains(char::is_control) {
        0
    } else {
        g.cell_width()
    }
}

/// Display width in columns, as `Buffer::set_stringn` consumes them.
pub fn width(s: &str) -> u16 {
    if s.len() == 1 {
        return grapheme_width(s);
    }
    s.graphemes(true)
        .fold(0u16, |acc, g| acc.saturating_add(grapheme_width(g)))
}

/// Whether `c` is a word character for word motion; one definition shared by
/// the editor core and the viewport.
pub(crate) fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Truncate to `max` columns, appending `…` when cut. Non-render callers
/// only: painting goes through the clipping writer (R‑2).
pub fn truncate(s: &str, max: u16) -> String {
    if width(s) <= max {
        return s.to_owned();
    }
    if max == 0 {
        return String::new();
    }
    let budget = max.saturating_sub(1);
    let mut out = String::new();
    let mut w = 0u16;
    for g in s.graphemes(true) {
        let gw = grapheme_width(g);
        if w.saturating_add(gw) > budget {
            break;
        }
        out.push_str(g);
        w = w.saturating_add(gw);
    }
    out.push('…');
    out
}

/// Truncate keeping both ends: `very_long_identifier_name` → `very_l…_name`.
pub fn truncate_middle(s: &str, max: u16) -> String {
    if width(s) <= max {
        return s.to_owned();
    }
    if max < 5 {
        return truncate(s, max);
    }
    let keep_end = max.saturating_sub(1) / 3;
    let keep_start = max.saturating_sub(1).saturating_sub(keep_end);
    let mut head_end = 0usize;
    let mut w = 0u16;
    for (i, g) in s.grapheme_indices(true) {
        let gw = grapheme_width(g);
        if w.saturating_add(gw) > keep_start {
            break;
        }
        head_end = i.saturating_add(g.len());
        w = w.saturating_add(gw);
    }
    let mut tail_start = s.len();
    let mut w = 0u16;
    for (i, g) in s.grapheme_indices(true).rev() {
        let gw = grapheme_width(g);
        if w.saturating_add(gw) > keep_end {
            break;
        }
        tail_start = i;
        w = w.saturating_add(gw);
    }
    let head = s.get(..head_end).unwrap_or("");
    let tail = s.get(tail_start..).unwrap_or("");
    let mut out = String::with_capacity(
        head.len()
            .saturating_add(tail.len())
            .saturating_add('…'.len_utf8()),
    );
    out.push_str(head);
    out.push('…');
    out.push_str(tail);
    out
}

/// One step of the wrap walk: text to append, a separating space, or a line
/// break. `wrap` and `wrapped_rows` are both written against it so the row
/// count and the rows themselves cannot drift.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WrapPiece<'a> {
    Text(&'a str),
    Space,
    Break,
}

/// Word-wrap `s` to `w` columns, hard-wrapping a word longer than the width,
/// emitting one [`WrapPiece`] per step. Allocation-free in itself.
fn wrap_walk<'a>(s: &'a str, w: u16, f: &mut dyn FnMut(WrapPiece<'a>)) {
    let w = w.max(1);
    for (i, para) in s.split('\n').enumerate() {
        if i > 0 {
            f(WrapPiece::Break);
        }
        let mut lw = 0u16;
        for word in para.split(' ') {
            let ww = width(word);
            if lw == 0 {
                if ww <= w {
                    f(WrapPiece::Text(word));
                    lw = ww;
                } else {
                    hard_wrap_walk(word, w, &mut lw, f);
                }
            } else if lw.saturating_add(1).saturating_add(ww) <= w {
                f(WrapPiece::Space);
                f(WrapPiece::Text(word));
                lw = lw.saturating_add(1).saturating_add(ww);
            } else {
                f(WrapPiece::Break);
                lw = 0;
                if ww <= w {
                    f(WrapPiece::Text(word));
                    lw = ww;
                } else {
                    hard_wrap_walk(word, w, &mut lw, f);
                }
            }
        }
    }
}

fn hard_wrap_walk<'a>(word: &'a str, w: u16, lw: &mut u16, f: &mut dyn FnMut(WrapPiece<'a>)) {
    for g in word.graphemes(true) {
        let gw = grapheme_width(g);
        if lw.saturating_add(gw) > w {
            f(WrapPiece::Break);
            *lw = 0;
        }
        f(WrapPiece::Text(g));
        *lw = lw.saturating_add(gw);
    }
}

/// Word-wrap into lines of at most `w` columns; hard-wraps overlong words.
pub fn wrap(s: &str, w: u16) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    wrap_walk(s, w, &mut |p| match p {
        WrapPiece::Text(t) => line.push_str(t),
        WrapPiece::Space => line.push(' '),
        WrapPiece::Break => lines.push(core::mem::take(&mut line)),
    });
    lines.push(line);
    lines
}

/// The number of rows [`wrap`] would produce, without building them: the
/// sizing half of the wrap, for a component that must state its height as a
/// pure function of its props before it draws (Adjudication N1,
/// `Dialog::measured_height`). **Zero allocations.**
pub fn wrapped_rows(s: &str, w: u16) -> u16 {
    let mut rows = 1u16;
    wrap_walk(s, w, &mut |p| {
        if p == WrapPiece::Break {
            rows = rows.saturating_add(1);
        }
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Style;

    /// Columns `Buffer::set_stringn` actually consumes for `s`.
    fn consumed(s: &str) -> u16 {
        let mut buf = Buffer::empty(Rect::new(0, 0, 200, 1));
        let (x, _) = buf.set_stringn(0, 0, s, 200, Style::new());
        x
    }

    #[test]
    fn width_matches_ratatui_cell_width() {
        let corpus: &[&str] = &[
            "",
            "a",
            "hello",
            "ｶﾞ",
            "あ",
            "a\u{FF9E}",
            "日本語",
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
            "e\u{301}",
            "\u{00E9}",
            "abc\u{FF9F}x",
        ];
        for s in corpus {
            assert_eq!(width(s), s.cell_width(), "cell_width disagrees on {s:?}");
            assert_eq!(width(s), consumed(s), "set_stringn disagrees on {s:?}");
        }
        // control-bearing strings: the writer drops them; `width` agrees
        // with the writer, which is the columns layout must reserve
        for s in ["\r\n", "\u{7}", "a\u{7}b", "x\r\ny"] {
            assert_eq!(width(s), consumed(s), "set_stringn disagrees on {s:?}");
        }
        assert_eq!(width("\u{7}"), 0);
        assert_eq!(width("\r\n"), 0);
    }

    #[test]
    fn wide_characters_count_as_two_columns() {
        assert_eq!(width("日本"), 4);
        assert_eq!(grapheme_width("漢"), 2);
    }

    #[test]
    fn combining_marks_are_one_grapheme() {
        assert_eq!(graphemes("e\u{301}x").count(), 2);
        assert_eq!(width("e\u{301}"), 1);
    }

    #[test]
    fn zwj_emoji_is_one_grapheme() {
        let fam = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}";
        assert_eq!(graphemes(fam).count(), 1);
        assert_eq!(width(fam), 2);
    }

    #[test]
    fn truncates_with_ellipsis_and_middle() {
        assert_eq!(truncate("hello world", 5), "hell…");
        assert_eq!(truncate("hi", 5), "hi");
        assert_eq!(truncate("日本語", 3), "日…");
        assert_eq!(truncate("abc", 0), "");
        assert_eq!(
            truncate_middle("very_long_identifier_name", 12),
            "very_lon…ame"
        );
        assert_eq!(truncate_middle("short", 12), "short");
        assert_eq!(truncate_middle("abcdefghij", 4), "abc…");
    }

    /// `wrapped_rows` is the sizing half of `wrap`: they must agree exactly,
    /// or `Dialog::measured_height` and `Dialog::draw` drift (Adjudication N1).
    #[test]
    fn wrapped_rows_matches_wrap() {
        let corpus: &[&str] = &[
            "",
            " ",
            "a",
            "hello world",
            "one two three four five six seven eight nine ten",
            "supercalifragilisticexpialidocious",
            "日本語のテキストは全角です",
            "a\nb\nc",
            "\n",
            "line one\n\nline three",
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466} family",
            "trailing space ",
            "double  space",
        ];
        for s in corpus {
            for w in [1u16, 2, 3, 5, 8, 13, 40] {
                assert_eq!(
                    usize::from(wrapped_rows(s, w)),
                    wrap(s, w).len(),
                    "{s:?} at width {w}"
                );
            }
        }
    }

    #[test]
    fn wraps_words_and_hard_wraps_long_tokens() {
        assert_eq!(wrap("aa bb cc", 5), vec!["aa bb", "cc"]);
        assert_eq!(wrap("abcdefgh", 3), vec!["abc", "def", "gh"]);
        assert_eq!(wrap("a\nb", 10), vec!["a", "b"]);
        assert_eq!(wrap("", 10), vec![""]);
        assert_eq!(wrap("日本語です", 4), vec!["日本", "語で", "す"]);
    }
}
