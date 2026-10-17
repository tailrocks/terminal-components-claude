//! Pointer text selection shared by the read-only text views
//! ([`TextViewport`](crate::widgets::viewport::TextViewport) and
//! [`ScrollPanel`](crate::widgets::panel::ScrollPanel)): one gesture model
//! over ordered positions, so every view selects the same way (macOS text).
//!
//! - A press anchors a selection; a drag extends it from the anchor.
//! - Shift with a press extends the current selection from its anchor
//!   (where it started), and a drag from there keeps extending it.
//! - A double click selects the word under the pointer; a drag that starts
//!   on its second press extends by whole words.
//! - A triple click selects the line.
//! - The release that ends a drag or a Shift press keeps the selection.
//!
//! What a drag, a double and a triple click select is the view's
//! [`PointerSelect`] policy, the one the text fields take.
//!
//! The model never measures text: the view resolves positions (a word or
//! a line under a position) and hands them in, so each view keeps its own
//! layout (a viewport's segmented cells, a panel's plain lines).

use std::ops::Range;

use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Position;
use unicode_segmentation::UnicodeSegmentation;

use crate::core::event::Outcome;
use crate::widgets::field_common::PointerSelect;

/// The selection and the pointer gesture in progress, over positions `P`
/// ordered as the text reads.
#[derive(Debug, Clone)]
pub(crate) struct TextSelect<P> {
    /// `(anchor, head)`: the anchor stays where the selection started.
    pub sel: Option<(P, P)>,
    /// Where the press a drag extends from landed.
    pub press: Option<P>,
    /// The word the second press of a double click selected: a drag from
    /// it extends by whole words.
    pub word: Option<(P, P)>,
    /// Where the last drag extended the selection: the release there (a
    /// click on the same control) keeps it.
    pub dragged: Option<Position>,
    /// A Shift press extended the selection: its release keeps it.
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
    /// The selection, ordered; `None` when nothing (or nothing wide) is
    /// selected.
    pub fn range(&self) -> Option<(P, P)> {
        let (a, b) = self.sel?;
        (a != b).then(|| (a.min(b), a.max(b)))
    }

    /// A drag extends by whole words ([`PointerSelect::drag_by_word`]):
    /// the view resolves the word under the drag for [`Self::drag`].
    pub fn by_word(&self) -> bool {
        self.word.is_some()
    }

    /// Select `a..b` outright (Select All, the keyboard): no gesture is
    /// left in progress.
    pub fn set(&mut self, a: P, b: P) {
        self.sel = Some((a, b));
        self.press = None;
        self.word = None;
        self.extended = false;
    }

    /// Drop the selection. `Changed` when there was one.
    pub fn clear(&mut self) -> Outcome {
        self.word = None;
        if self.sel.take().is_some() {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    /// Button down at `at` (`None`: no text there) with its multi-click
    /// `count` and `mods`. `word` and `line` are the word and the line at
    /// `at`, which a double and a triple press select per `policy`.
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
        // Shift extends the selection (or the last press) from its anchor
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

    /// A drag to `head` at `pos` (the button held since a press): extend
    /// the selection from the press, or by whole words from a double
    /// click's word (`word` is the word at `head` then).
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

    /// A completed click at `pos` with its multi-click `count`: a double
    /// click selects `word`, a triple click `line` (per `policy`); the
    /// release that ends a drag or a Shift press keeps the selection.
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

/// A grapheme that belongs to a word for a double click: letters, digits
/// and `_ - / .`, so a path or a dotted name selects whole.
pub(crate) fn is_word(g: &str) -> bool {
    g.chars()
        .all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '-' | '/' | '.'))
}

/// The run of word items around item `i` (clamped to the last item):
/// `start..end` item indices, `None` when item `i` is not a word item.
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

/// One grapheme of a plain line: its bytes and the cells it is drawn in.
#[derive(Debug, Clone)]
pub(crate) struct PlainCell<'a> {
    pub g: &'a str,
    pub bytes: Range<usize>,
    /// The first column it is drawn at.
    pub col: usize,
}

/// The graphemes of a plain line with their columns.
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

/// The index of the cell drawn at column `col` (`cells.len()` past the
/// end).
pub(crate) fn plain_cell_at(cells: &[PlainCell<'_>], col: usize) -> usize {
    cells
        .iter()
        .position(|c| c.col + crate::ui::text::width(c.g).max(1) > col)
        .unwrap_or(cells.len())
}

/// The column the cell `i` starts at (the line width past the end).
pub(crate) fn plain_col_of(s: &str, cells: &[PlainCell<'_>], i: usize) -> usize {
    cells
        .get(i)
        .map(|c| c.col)
        .unwrap_or_else(|| crate::ui::text::width(s))
}

/// The source text of columns `from..to` of a plain line.
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
        // Shift+press before the anchor: the anchor stays at 2
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
