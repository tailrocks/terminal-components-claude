//! Small text helpers: truncation with ellipsis, padding, width.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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

/// Word-wrap into lines of at most `w` cells. Hard-wraps overlong words.
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
        let gw = width(g);
        if *lw + gw > w {
            lines.push(std::mem::take(line));
            *lw = 0;
        }
        line.push_str(g);
        *lw += gw;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_with_ellipsis() {
        assert_eq!(truncate("hello world", 5), "hell…");
        assert_eq!(truncate("hi", 5), "hi");
        assert_eq!(fit("hi", 4), "hi  ");
        assert_eq!(fit_right("hi", 4), "  hi");
    }

    #[test]
    fn wraps_words_and_hard_wraps_long_tokens() {
        assert_eq!(wrap("aa bb cc", 5), vec!["aa bb", "cc"]);
        assert_eq!(wrap("abcdefgh", 3), vec!["abc", "def", "gh"]);
        assert_eq!(wrap("a\nb", 10), vec!["a", "b"]);
        assert_eq!(wrap("", 10), vec![""]);
    }
}
