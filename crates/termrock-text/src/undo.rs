//! Undo / redo stack (ported from PR #26 `src/core/edit.rs`,

use crate::buffer::TextBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Snap {
    text: String,
    cursor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditKind {
    #[default]
    Typing,

    Deleting,

    Other,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoStack {
    undo: Vec<Snap>,
    redo: Vec<Snap>,
    run: Option<EditKind>,
}

impl UndoStack {
    pub const LIMIT: usize = 200;

    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn seal(&mut self) {
        self.run = None;
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.run = None;
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

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
