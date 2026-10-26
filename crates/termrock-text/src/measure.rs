use ratatui_core::buffer::CellWidth;
use unicode_segmentation::UnicodeSegmentation;

pub fn graphemes(s: &str) -> impl DoubleEndedIterator<Item = (usize, &str)> {
    s.grapheme_indices(true)
}

pub fn grapheme_width(g: &str) -> u16 {
    if g.contains(char::is_control) {
        0
    } else {
        g.cell_width()
    }
}

pub fn width(s: &str) -> u16 {
    if s.len() == 1 {
        return grapheme_width(s);
    }
    s.graphemes(true)
        .fold(0u16, |acc, g| acc.saturating_add(grapheme_width(g)))
}

pub(crate) fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

pub(crate) fn is_word_grapheme(g: &str) -> bool {
    g.chars().any(is_word_char)
}

pub fn truncate_parts(s: &str, max: u16) -> (&str, &str) {
    if width(s) <= max {
        return (s, "");
    }
    if max == 0 {
        return ("", "");
    }
    let budget = max.saturating_sub(1);
    let mut head_end = 0;
    let mut w = 0u16;
    for (i, g) in s.grapheme_indices(true) {
        let gw = grapheme_width(g);
        if w.saturating_add(gw) > budget {
            break;
        }
        head_end = i.saturating_add(g.len());
        w = w.saturating_add(gw);
    }
    (s.get(..head_end).unwrap_or(""), "…")
}

pub fn truncate(s: &str, max: u16) -> String {
    let (head, ellipsis) = truncate_parts(s, max);
    if ellipsis.is_empty() {
        return head.to_owned();
    }
    let mut out = String::with_capacity(head.len() + ellipsis.len());
    out.push_str(head);
    out.push_str(ellipsis);
    out
}

pub fn truncate_middle(s: &str, max: u16) -> String {
    let parts = middle_parts(s, max);
    let mut out = String::with_capacity(parts.iter().map(|part| part.len()).sum());
    for part in parts {
        out.push_str(part);
    }
    out
}

pub fn middle_parts(s: &str, max: u16) -> [&str; 3] {
    if width(s) <= max {
        return [s, "", ""];
    }
    if max == 0 {
        return ["", "", ""];
    }
    let keep_end = if max < 5 {
        0
    } else {
        max.saturating_sub(1) / 3
    };
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
    if keep_end > 0 {
        for (i, g) in s.grapheme_indices(true).rev() {
            let gw = grapheme_width(g);
            if w.saturating_add(gw) > keep_end {
                break;
            }
            tail_start = i;
            w = w.saturating_add(gw);
        }
    }
    [
        s.get(..head_end).unwrap_or(""),
        "…",
        s.get(tail_start..).unwrap_or(""),
    ]
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WrapPiece<'a> {
    Text(&'a str),
    Space,
    Break,
}

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
        let projected = if grapheme_width(g) > w { "…" } else { g };
        let gw = grapheme_width(projected);
        if lw.saturating_add(gw) > w {
            f(WrapPiece::Break);
            *lw = 0;
        }
        f(WrapPiece::Text(projected));
        *lw = lw.saturating_add(gw);
    }
}

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

pub fn wrapped_rows(s: &str, w: u16) -> u16 {
    let mut rows = 1u16;
    wrap_walk(s, w, &mut |p| {
        if p == WrapPiece::Break {
            rows = rows.saturating_add(1);
        }
    });
    rows
}
