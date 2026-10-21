//! Undo / redo stack (ported from PR #26 `src/core/edit.rs`,
//! `UndoStack` at lines 278-365, commit f4867e9b5).
//!
//! Snapshot-based undo over a [`TextBuffer`]. Consecutive records of the
//! same kind fold into one undo step, so a typing run undoes as one;
//! `seal` ends the run early.

use crate::buffer::TextBuffer;

/// A text and cursor snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Snap {
    text: String,
    cursor: usize,
}

/// The kind of edit being recorded, which decides folding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditKind {
    /// Inserting text: consecutive typing folds into one step.
    #[default]
    Typing,
    /// Deleting text: consecutive deletes fold into one step.
    Deleting,
    /// Any other edit: never folds, and breaks the current run.
    Other,
}

/// A bounded undo / redo stack over a [`TextBuffer`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoStack {
    undo: Vec<Snap>,
    redo: Vec<Snap>,
    run: Option<EditKind>,
}

impl UndoStack {
    /// The maximum number of undo steps kept.
    pub const LIMIT: usize = 200;

    pub fn new() -> Self {
        Self::default()
    }

    /// Record a snapshot before applying an edit of `kind`. Recording clears
    /// the redo stack; consecutive records of the same typing or deleting
    /// kind fold into the current step.
    pub fn record(&mut self, buf: &TextBuffer, kind: EditKind) {
        self.redo.clear();
        let folds = kind != EditKind::Other && self.run == Some(kind);
        if !folds {
            self.undo.push(Snap {
                text: buf.text().to_owned(),
                cursor: buf.cursor_offset(),
            });
            if self.undo.len() > Self::LIMIT {
                self.undo.remove(0);
            }
        }
        self.run = (kind != EditKind::Other).then_some(kind);
    }

    /// End the current run so the next record starts a new step.
    pub fn seal(&mut self) {
        self.run = None;
    }

    /// Drop every undo and redo step.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.run = None;
    }

    /// Whether there is anything to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is anything to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Restore the last snapshot, pushing the current state onto the redo
    /// stack. Returns whether anything happened.
    pub fn undo(&mut self, buf: &mut TextBuffer) -> bool {
        let Some(s) = self.undo.pop() else {
            return false;
        };
        self.redo.push(Snap {
            text: buf.text().to_owned(),
            cursor: buf.cursor_offset(),
        });
        restore(buf, s);
        self.run = None;
        true
    }

    /// Reapply the last undone state, pushing the current state onto the
    /// undo stack. Returns whether anything happened.
    pub fn redo(&mut self, buf: &mut TextBuffer) -> bool {
        let Some(s) = self.redo.pop() else {
            return false;
        };
        self.undo.push(Snap {
            text: buf.text().to_owned(),
            cursor: buf.cursor_offset(),
        });
        restore(buf, s);
        self.run = None;
        true
    }
}

fn restore(buf: &mut TextBuffer, s: Snap) {
    buf.set_text(&s.text);
    let at = s.cursor.min(buf.text().len());
    buf.select_range(at, at);
    buf.clear_selection();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_folds_into_one_step_and_redo_reapplies_it() {
        let mut b = TextBuffer::single("");
        let mut u = UndoStack::new();
        for c in "abc".chars() {
            u.record(&b, EditKind::Typing);
            b.insert_char(c);
        }
        u.record(&b, EditKind::Deleting);
        b.backspace();
        assert_eq!(b.text(), "ab");
        assert!(u.undo(&mut b));
        assert_eq!(b.text(), "abc");
        assert!(u.undo(&mut b));
        assert_eq!(b.text(), "");
        assert!(!u.undo(&mut b));
        assert!(u.redo(&mut b));
        assert_eq!(b.text(), "abc");
        assert_eq!(b.cursor_offset(), 3);
    }

    #[test]
    fn a_sealed_run_starts_a_new_step() {
        let mut b = TextBuffer::single("");
        let mut u = UndoStack::new();
        u.record(&b, EditKind::Typing);
        b.insert_char('a');
        u.seal();
        u.record(&b, EditKind::Typing);
        b.insert_char('b');
        assert!(u.undo(&mut b));
        assert_eq!(b.text(), "a");
    }

    #[test]
    fn an_other_edit_breaks_the_run_and_the_limit_holds() {
        let mut b = TextBuffer::single("");
        let mut u = UndoStack::new();
        u.record(&b, EditKind::Typing);
        b.insert_char('a');
        u.record(&b, EditKind::Other);
        b.insert_char('b');
        assert!(u.undo(&mut b));
        assert_eq!(b.text(), "a", "Other never folds with the typing run");
        u.clear();
        assert!(!u.can_undo() && !u.can_redo());

        let mut b = TextBuffer::single("");
        let mut u = UndoStack::new();
        for i in 0..(UndoStack::LIMIT + 10) {
            u.record(&b, EditKind::Other);
            b.insert_char(char::from_digit((i % 10) as u32, 10).unwrap());
        }
        let mut steps = 0;
        while u.undo(&mut b) {
            steps += 1;
        }
        assert_eq!(steps, UndoStack::LIMIT, "the stack stays bounded");
    }
}
