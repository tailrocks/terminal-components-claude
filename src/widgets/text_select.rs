use std::ops::Range;

use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Position;
use unicode_segmentation::UnicodeSegmentation;

use crate::core::event::Outcome;
use crate::widgets::field_common::PointerSelect;

#[derive(Debug, Clone)]
pub(crate) struct TextSelect<P> {
    pub sel: Option<(P, P)>,

    pub press: Option<P>,

    pub word: Option<(P, P)>,

    pub dragged: Option<Position>,

    extended: bool,
}

impl<P> Default for TextSelect<P> {
    fn default() -> Self {
        Self {
            sel: None,
            press: None,
            word: None,
            dragged: None,
            extended: false,
        }
    }
}

impl<P: Copy + Ord> TextSelect<P> {
    pub fn range(&self) -> Option<(P, P)> {
        let (a, b) = self.sel?;
        (a != b).then(|| (a.min(b), a.max(b)))
    }

    pub fn by_word(&self) -> bool {
        self.word.is_some()
    }

    pub fn set(&mut self, a: P, b: P) {
        self.sel = Some((a, b));
        self.press = None;
        self.word = None;
        self.extended = false;
    }

    pub fn clear(&mut self) -> Outcome {
        self.word = None;
        if self.sel.take().is_some() {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn press(
        &mut self,
        policy: PointerSelect,
        at: Option<P>,
        count: u8,
        mods: KeyModifiers,
        word: Option<(P, P)>,
        line: Option<(P, P)>,
    ) -> Outcome {
        self.dragged = None;
        self.extended = false;
        self.word = None;
        let Some(at) = at else {
            self.press = None;
            return match self.sel.take() {
                Some(_) => Outcome::Changed,
                None => Outcome::Consumed,
            };
        };

        if mods.contains(KeyModifiers::SHIFT)
            && let Some(anchor) = self.sel.map(|s| s.0).or(self.press)
        {
            self.sel = Some((anchor, at));
            self.press = Some(anchor);
            self.extended = true;
            return Outcome::Changed;
        }
        if count >= 3
            && policy.line
            && let Some(l) = line
        {
            self.sel = Some(l);
            self.press = None;
            return Outcome::Changed;
        }
        if count == 2
            && policy.word
            && policy.drag_by_word
            && let Some(w) = word
        {
            self.sel = Some(w);
            self.press = Some(w.0);
            self.word = Some(w);
            return Outcome::Changed;
        }
        let had = self.sel.take().is_some();
        self.press = Some(at);
        if had {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    pub fn drag(
        &mut self,
        policy: PointerSelect,
        head: Option<P>,
        pos: Position,
        word: Option<(P, P)>,
    ) -> Outcome {
        let Some(press) = self.press else {
            return Outcome::Ignored;
        };
        let Some(head) = head.filter(|_| policy.drag) else {
            return Outcome::Consumed;
        };
        self.sel = Some(match self.word {
            Some(w) => {
                let to = word.unwrap_or((head, head));
                if to.0 < w.0 {
                    (w.1, to.0)
                } else {
                    (w.0, to.1.max(w.1))
                }
            }
            None => (press, head),
        });
        self.dragged = Some(pos);
        Outcome::Changed
    }

    pub fn click(
        &mut self,
        policy: PointerSelect,
        pos: Position,
        count: u8,
        word: Option<(P, P)>,
        line: Option<(P, P)>,
    ) -> Outcome {
        let dragged = self.dragged.take() == Some(pos) && self.sel.is_some();
        let extended = std::mem::take(&mut self.extended);
        self.word = None;
        if dragged || extended {
            return Outcome::Consumed;
        }
        let pick = if count >= 3 && policy.line {
            line
        } else if count == 2 && policy.word {
            word
        } else {
            None
        };
        match pick {
            Some((a, b)) => {
                self.sel = Some((a, b));
                self.press = None;
                Outcome::Changed
            }
            None => Outcome::Consumed,
        }
    }
}

pub(crate) fn is_word(g: &str) -> bool {
    g.chars()
        .all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '-' | '/' | '.'))
}

pub(crate) fn word_bounds<T>(
    items: &[T],
    i: usize,
    word: impl Fn(&T) -> bool,
) -> Option<(usize, usize)> {
    if items.is_empty() {
        return None;
    }
    let i = i.min(items.len() - 1);
    if !word(&items[i]) {
        return None;
    }
    let mut s = i;
    while s > 0 && word(&items[s - 1]) {
        s -= 1;
    }
    let mut e = i + 1;
    while e < items.len() && word(&items[e]) {
        e += 1;
    }
    Some((s, e))
}

#[derive(Debug, Clone)]
pub(crate) struct PlainCell<'a> {
    pub g: &'a str,
    pub bytes: Range<usize>,

    pub col: usize,
}

pub(crate) fn plain_cells(s: &str) -> Vec<PlainCell<'_>> {
    let mut col = 0;
    s.grapheme_indices(true)
        .map(|(at, g)| {
            let c = PlainCell {
                g,
                bytes: at..at + g.len(),
                col,
            };
            col += crate::ui::text::width(g);
            c
        })
        .collect()
}

pub(crate) fn plain_cell_at(cells: &[PlainCell<'_>], col: usize) -> usize {
    cells
        .iter()
        .position(|c| c.col + crate::ui::text::width(c.g).max(1) > col)
        .unwrap_or(cells.len())
}

pub(crate) fn plain_col_of(s: &str, cells: &[PlainCell<'_>], i: usize) -> usize {
    cells
        .get(i)
        .map(|c| c.col)
        .unwrap_or_else(|| crate::ui::text::width(s))
}

pub(crate) fn plain_slice(s: &str, from: usize, to: usize) -> &str {
    let cells = plain_cells(s);
    let a = plain_cell_at(&cells, from);
    let b = plain_cell_at(&cells, to);
    if a >= b {
        return "";
    }
    let start = cells[a].bytes.start;
    let end = cells[b - 1].bytes.end;
    &s[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    const ON: PointerSelect = PointerSelect::DEFAULT;
    const P: Position = Position::new(0, 0);

    #[test]
    fn shift_press_extends_from_the_anchor_and_its_release_keeps_it() {
        let mut s = TextSelect::<u32>::default();
        s.press(ON, Some(2), 1, KeyModifiers::NONE, None, None);
        s.drag(ON, Some(5), P, None);
        s.click(ON, P, 1, None, None);
        assert_eq!(s.range(), Some((2, 5)));

        s.press(ON, Some(0), 1, KeyModifiers::SHIFT, None, None);
        assert_eq!(
            s.click(ON, Position::new(9, 9), 1, None, None),
            Outcome::Consumed
        );
        assert_eq!(s.sel, Some((2, 0)));
        assert_eq!(s.range(), Some((0, 2)));
    }

    #[test]
    fn a_double_press_drag_extends_by_words() {
        let mut s = TextSelect::<u32>::default();
        s.press(ON, Some(4), 2, KeyModifiers::NONE, Some((3, 6)), None);
        assert_eq!(s.range(), Some((3, 6)));
        s.drag(ON, Some(9), P, Some((8, 12)));
        assert_eq!(s.range(), Some((3, 12)));
        s.drag(ON, Some(1), P, Some((0, 2)));
        assert_eq!(s.range(), Some((0, 6)));
    }

    #[test]
    fn words_are_runs_of_word_graphemes() {
        let cells = plain_cells("ab cd/e.f  g");
        let w = |i| word_bounds(&cells, i, |c: &PlainCell| is_word(c.g));
        assert_eq!(w(4), Some((3, 9)));
        assert_eq!(w(2), None);
        assert_eq!(plain_slice("héllo wörld", 2, 8), "llo wö");
    }
}
