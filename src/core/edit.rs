//! Editing policy shared by every text control (`TextInput`, `TextArea`,
//! `CodeEditor`, the prompt fields of `Dialog`): when focus starts an edit
//! ([`EditEntry`]), which control-key preset the fields use
//! ([`FieldKeys`]), the undo history ([`UndoStack`]) and the field
//! clipboard ([`clipboard`]).
//!
//! Every policy has an app-wide default on the theme
//! ([`crate::theme::Theme::edit_entry`], [`crate::theme::Theme::field_keys`])
//! and a per-widget override.

use super::text::TextBuffer;

/// How a focused text control starts editing.
///
/// The default follows macOS: a text field that receives focus takes
/// typing at once, with the caret at the end of its value (or at the click
/// point when the pointer focused it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditEntry {
    /// Focus starts the edit: the caret shows as soon as the control is
    /// focused, typing edits, Enter applies, Tab applies and moves on, Esc
    /// reverts while the value is dirty and otherwise passes on (so the
    /// page's Esc still runs). Page letter shortcuts never see the keys.
    #[default]
    Immediate,
    /// Focus shows the field; the first typed character starts the edit
    /// (landing at the end of the value). Enter and F2 start it too.
    TypeToEdit,
    /// Enter, F2 or a click start the edit; plain letters stay with the
    /// page's shortcuts until then.
    Explicit,
}

/// Where the caret lands when the keyboard (Tab, a focus move) starts an
/// [`EditEntry::Immediate`] edit. A click always places the caret at the
/// pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusSelect {
    /// Select the whole value, so typing replaces it (macOS and Windows
    /// text boxes reached by Tab).
    #[default]
    SelectAll,
    /// Caret after the last character, nothing selected.
    CaretEnd,
    /// Caret before the first character, nothing selected.
    CaretStart,
}

impl FocusSelect {
    /// Place the caret (or the selection) in `buf` per this policy.
    pub fn apply(self, buf: &mut TextBuffer) {
        match self {
            FocusSelect::SelectAll => buf.select_all(),
            FocusSelect::CaretEnd => {
                buf.move_doc_end(false);
                buf.clear_selection();
            }
            FocusSelect::CaretStart => {
                buf.move_doc_start(false);
                buf.clear_selection();
            }
        }
    }
}

/// The control-key preset of text fields.
///
/// Both presets share the platform-neutral keys: arrows, Home/End,
/// Backspace/Delete, Ctrl/Alt+arrows by word, Ctrl+Z undo and
/// Ctrl+Shift+Z redo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldKeys {
    /// The Cocoa text-system bindings macOS fields answer to: Ctrl+A/E
    /// line start/end, Ctrl+K kill to end, Ctrl+U kill to start, Ctrl+W
    /// delete the word (cut the selection when there is one), Ctrl+Y yank,
    /// Alt+B/F by word, Ctrl+L select all.
    #[default]
    Emacs,
    /// Windows and Linux desktop bindings: Ctrl+A select all, Ctrl+C copy,
    /// Ctrl+X cut, Ctrl+V paste, Ctrl+Y redo.
    Desktop,
}

/// One state the undo history can return to.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Snap {
    text: String,
    cursor: usize,
}

/// What kind of edit was recorded last, so a run of typed characters (or
/// of deletions) undoes as one step, the way desktop fields group typing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditKind {
    /// A typed character that continues a word.
    #[default]
    Typing,
    /// A Backspace or Delete.
    Deleting,
    /// Anything else (a paste, a cut, a kill, an indent): its own step.
    Other,
}

/// Undo and redo history for one text buffer.
///
/// The owner records the buffer *before* each change with [`Self::record`];
/// consecutive typing (or deleting) folds into one step until a different
/// kind of edit, a word break or [`Self::seal`] ends the run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoStack {
    undo: Vec<Snap>,
    redo: Vec<Snap>,
    run: Option<EditKind>,
}

impl UndoStack {
    /// The default history depth.
    pub const LIMIT: usize = 200;

    pub fn new() -> Self {
        Self::default()
    }

    /// Record `buf` as it is before an edit of `kind`. A run of the same
    /// kind (typing or deleting) folds into the step that started it.
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

    /// End the current typing run: the next edit starts a new step.
    pub fn seal(&mut self) {
        self.run = None;
    }

    /// Forget the whole history (a value set from outside, a new document).
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

    /// Step `buf` back one edit; `false` when there is nothing to undo.
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

    /// Re-apply the edit [`Self::undo`] stepped back; `false` when there is
    /// nothing to redo.
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
    buf.set_text(s.text);
    let at = s.cursor.min(buf.text().len());
    buf.select_range(at, at);
    buf.clear_selection();
}

/// The field clipboard: what Copy and Cut put away and Paste (or Ctrl+Y)
/// brings back.
///
/// A terminal application cannot read the system clipboard, so the text
/// is kept per thread; an application that can reach the system clipboard
/// (OSC 52, a platform crate) installs a hook with [`clipboard::set_hook`]
/// and every copy or cut in every field is handed to it as well.
pub mod clipboard {
    use std::cell::RefCell;

    type Hook = Box<dyn Fn(&str)>;

    thread_local! {
        static TEXT: RefCell<Option<String>> = const { RefCell::new(None) };
        static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
    }

    /// Put `text` on the field clipboard (and hand it to the hook).
    pub fn copy(text: &str) {
        TEXT.with(|t| *t.borrow_mut() = Some(text.to_owned()));
        HOOK.with(|h| {
            if let Some(f) = h.borrow().as_ref() {
                f(text);
            }
        });
    }

    /// The text last copied or cut, if any.
    pub fn text() -> Option<String> {
        TEXT.with(|t| t.borrow().clone())
    }

    /// Hand every copy and cut to `hook` too (the system clipboard).
    pub fn set_hook(hook: impl Fn(&str) + 'static) {
        HOOK.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
    }

    /// Remove the hook installed with [`set_hook`].
    pub fn clear_hook() {
        HOOK.with(|h| *h.borrow_mut() = None);
    }
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
    fn the_clipboard_hook_sees_every_copy() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
        let s = seen.clone();
        clipboard::set_hook(move |t| s.borrow_mut().push_str(t));
        clipboard::copy("abc");
        assert_eq!(clipboard::text().as_deref(), Some("abc"));
        assert_eq!(seen.borrow().as_str(), "abc");
        clipboard::clear_hook();
    }
}
