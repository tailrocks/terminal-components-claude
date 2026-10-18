use super::event::ClientPlatform;
use super::keymap::{Actions, KeyBinding, KeyMap};
use super::text::TextBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditEntry {
    #[default]
    Immediate,

    TypeToEdit,

    Explicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusSelect {
    #[default]
    SelectAll,

    CaretEnd,

    CaretStart,
}

impl FocusSelect {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKeys {
    Emacs,

    Desktop,
}

impl FieldKeys {
    pub const fn for_platform(platform: ClientPlatform) -> Self {
        match platform {
            ClientPlatform::MacOs => Self::Emacs,
            ClientPlatform::Other => Self::Desktop,
        }
    }

    pub fn text_keymap(self) -> KeyMap<TextCmd> {
        let mut map: KeyMap<TextCmd> = CMD_TEXT_KEYS.to_vec();
        match self {
            Self::Emacs => map.extend([
                (KeyBinding::ctrl('l'), TextCmd::SelectAll),
                (KeyBinding::alt('w'), TextCmd::Copy),
                (KeyBinding::ctrl('w'), TextCmd::Cut),
                (KeyBinding::ctrl('y'), TextCmd::Paste),
            ]),
            Self::Desktop => map.extend([
                (KeyBinding::ctrl('a'), TextCmd::SelectAll),
                (KeyBinding::ctrl('c'), TextCmd::Copy),
                (KeyBinding::ctrl('x'), TextCmd::Cut),
                (KeyBinding::ctrl('w'), TextCmd::Cut),
                (KeyBinding::ctrl('v'), TextCmd::Paste),
                (KeyBinding::ctrl('y'), TextCmd::Redo),
            ]),
        }
        map.extend([
            (KeyBinding::ctrl('z').shift(), TextCmd::Redo),
            (KeyBinding::ctrl('Z'), TextCmd::Redo),
            (KeyBinding::ctrl('z'), TextCmd::Undo),
        ]);
        map
    }
}

pub const CMD_TEXT_KEYS: [(KeyBinding, TextCmd); 15] = {
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    const CMD: KeyModifiers = KeyModifiers::SUPER;
    const CMD_SHIFT: KeyModifiers = KeyModifiers::SUPER.union(KeyModifiers::SHIFT);
    [
        (KeyBinding::cmd('a'), TextCmd::SelectAll),
        (KeyBinding::cmd('c'), TextCmd::Copy),
        (KeyBinding::cmd('x'), TextCmd::Cut),
        (KeyBinding::cmd('v'), TextCmd::Paste),
        (KeyBinding::cmd('z').shift(), TextCmd::Redo),
        (KeyBinding::cmd('Z'), TextCmd::Redo),
        (KeyBinding::cmd('z'), TextCmd::Undo),
        (KeyBinding::new(KeyCode::Left, CMD), TextCmd::LineStart),
        (KeyBinding::new(KeyCode::Right, CMD), TextCmd::LineEnd),
        (KeyBinding::new(KeyCode::Up, CMD), TextCmd::DocStart),
        (KeyBinding::new(KeyCode::Down, CMD), TextCmd::DocEnd),
        (
            KeyBinding::new(KeyCode::Left, CMD_SHIFT),
            TextCmd::SelectToLineStart,
        ),
        (
            KeyBinding::new(KeyCode::Right, CMD_SHIFT),
            TextCmd::SelectToLineEnd,
        ),
        (
            KeyBinding::new(KeyCode::Up, CMD_SHIFT),
            TextCmd::SelectToDocStart,
        ),
        (
            KeyBinding::new(KeyCode::Down, CMD_SHIFT),
            TextCmd::SelectToDocEnd,
        ),
    ]
};

impl Default for FieldKeys {
    fn default() -> Self {
        Self::for_platform(ClientPlatform::current())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextCmd {
    SelectAll,

    Copy,

    Cut,

    Paste,

    Undo,

    Redo,

    LineStart,

    LineEnd,

    DocStart,

    DocEnd,

    SelectToLineStart,

    SelectToLineEnd,

    SelectToDocStart,

    SelectToDocEnd,
}

impl Actions for TextCmd {
    fn label(self) -> &'static str {
        match self {
            TextCmd::SelectAll => "Select all",
            TextCmd::Copy => "Copy",
            TextCmd::Cut => "Cut",
            TextCmd::Paste => "Paste",
            TextCmd::Undo => "Undo",
            TextCmd::Redo => "Redo",
            TextCmd::LineStart => "Line start",
            TextCmd::LineEnd => "Line end",
            TextCmd::DocStart => "Start of text",
            TextCmd::DocEnd => "End of text",
            TextCmd::SelectToLineStart => "Select to line start",
            TextCmd::SelectToLineEnd => "Select to line end",
            TextCmd::SelectToDocStart => "Select to start",
            TextCmd::SelectToDocEnd => "Select to end",
        }
    }

    fn default_keymap() -> KeyMap<Self> {
        FieldKeys::default().text_keymap()
    }
}

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
    buf.set_text(s.text);
    let at = s.cursor.min(buf.text().len());
    buf.select_range(at, at);
    buf.clear_selection();
}

pub mod clipboard {
    use std::cell::RefCell;

    type Hook = Box<dyn Fn(&str)>;

    thread_local! {
        static TEXT: RefCell<Option<String>> = const { RefCell::new(None) };
        static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
    }

    pub fn copy(text: &str) {
        TEXT.with(|t| *t.borrow_mut() = Some(text.to_owned()));
        HOOK.with(|h| {
            if let Some(f) = h.borrow().as_ref() {
                f(text);
            }
        });
    }

    pub fn text() -> Option<String> {
        TEXT.with(|t| t.borrow().clone())
    }

    pub fn set_hook(hook: impl Fn(&str) + 'static) {
        HOOK.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
    }

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
    fn field_keys_follow_the_client_platform() {
        assert_eq!(
            FieldKeys::for_platform(ClientPlatform::MacOs),
            FieldKeys::Emacs
        );
        assert_eq!(
            FieldKeys::for_platform(ClientPlatform::Other),
            FieldKeys::Desktop
        );
        assert_eq!(
            FieldKeys::default(),
            FieldKeys::for_platform(ClientPlatform::current())
        );
        let cmd_a = KeyBinding::cmd('a');
        for keys in [FieldKeys::Emacs, FieldKeys::Desktop] {
            assert_eq!(
                keys.text_keymap().first(),
                Some(&(cmd_a, TextCmd::SelectAll)),
                "Cmd+A selects all in every preset"
            );
        }
        assert!(
            FieldKeys::Emacs
                .text_keymap()
                .contains(&(KeyBinding::ctrl('l'), TextCmd::SelectAll))
        );
        assert!(
            !FieldKeys::Emacs
                .text_keymap()
                .iter()
                .any(|(b, _)| *b == KeyBinding::ctrl('a')),
            "Ctrl+A stays line start on macOS"
        );
        assert!(
            FieldKeys::Desktop
                .text_keymap()
                .contains(&(KeyBinding::ctrl('a'), TextCmd::SelectAll))
        );
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
