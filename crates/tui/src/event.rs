//! Raw input at the runtime boundary (`COMPONENT_ARCHITECTURE.md` §6.1).
//!
//! `Input` never touches a component directly: the runtime resolves it
//! against the last frame's registry and focus ring and delivers
//! [`Intent`](crate::intent::Intent)s instead. Key vocabulary is crossterm's
//! `KeyCode`/`KeyModifiers`, reached only through the `ratatui_crossterm`
//! re-export (§22 R‑14).

use core::fmt;

use ratatui_core::layout::Position;
pub use ratatui_crossterm::crossterm::event::{KeyCode, KeyModifiers};

use crate::secret::zeroize_string;

/// One normalised input event.
#[derive(PartialEq, Eq)]
pub enum Input {
    /// A key press or repeat (releases are dropped at normalisation).
    Key(Key),
    /// A pointer event with modifiers.
    Mouse(Mouse),
    /// The terminal was resized to `(columns, rows)`.
    Resize(u16, u16),
    /// Bracketed paste text.
    Paste(String),
    /// The runtime clock advanced by one `design.motion.tick_ms`.
    Tick,
}

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Input::Key(key) => f.debug_tuple("Key").field(key).finish(),
            Input::Mouse(mouse) => f.debug_tuple("Mouse").field(mouse).finish(),
            Input::Resize(width, height) => {
                f.debug_tuple("Resize").field(width).field(height).finish()
            }
            Input::Paste(text) => f
                .debug_struct("Paste")
                .field("len", &text.len())
                .field("text", &"[redacted]")
                .finish(),
            Input::Tick => f.write_str("Tick"),
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        if let Input::Paste(text) = self {
            zeroize_string(text);
        }
    }
}

/// A key press: code plus modifiers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Key {
    /// The key code.
    pub code: KeyCode,
    /// The modifiers held.
    pub mods: KeyModifiers,
}

impl Key {
    /// The chord this key press is, for matching against binding tables.
    pub const fn chord(&self) -> Chord {
        Chord {
            code: self.code,
            mods: self.mods,
        }
    }

    /// `code` matches and no modifier other than `SHIFT` is held.
    pub fn is(&self, c: KeyCode) -> bool {
        self.code == c && self.mods.difference(KeyModifiers::SHIFT).is_empty()
    }

    /// Whether `CONTROL` is held.
    pub const fn ctrl(&self) -> bool {
        self.mods.contains(KeyModifiers::CONTROL)
    }

    /// Whether `ALT` is held.
    pub const fn alt(&self) -> bool {
        self.mods.contains(KeyModifiers::ALT)
    }

    /// Whether `SHIFT` is held.
    pub const fn shift(&self) -> bool {
        self.mods.contains(KeyModifiers::SHIFT)
    }

    /// `Char(c)` with no modifier other than `SHIFT`: a typing key.
    pub const fn bare_char(&self) -> Option<char> {
        match self.code {
            KeyCode::Char(c) if self.mods.difference(KeyModifiers::SHIFT).is_empty() => Some(c),
            _ => None,
        }
    }
}

/// A key chord as written in a binding table.
///
/// `PartialEq`/`Hash` are the derived structural forms over crossterm's
/// `KeyCode` and `KeyModifiers`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Chord {
    /// The key code.
    pub code: KeyCode,
    /// The modifiers required.
    pub mods: KeyModifiers,
}

impl Chord {
    /// A chord with no modifiers.
    pub const fn key(c: KeyCode) -> Chord {
        Chord {
            code: c,
            mods: KeyModifiers::NONE,
        }
    }

    /// A chord with modifiers.
    pub const fn with(c: KeyCode, m: KeyModifiers) -> Chord {
        Chord { code: c, mods: m }
    }

    /// Whether this chord is a bare `Char` (skipped by the capture phase
    /// while the focused control swallows typing, §3.3 step 2).
    pub const fn is_bare_char(&self) -> bool {
        matches!(self.code, KeyCode::Char(_))
            && self.mods.difference(KeyModifiers::SHIFT).is_empty()
    }

    /// Whether a key press matches this chord (`SHIFT` on a `Char` is
    /// already folded into the character).
    pub fn matches(&self, k: &Key) -> bool {
        if self.code != k.code {
            return false;
        }
        if matches!(self.code, KeyCode::Char(_)) {
            self.mods.difference(KeyModifiers::SHIFT) == k.mods.difference(KeyModifiers::SHIFT)
        } else {
            self.mods == k.mods
        }
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mods.contains(KeyModifiers::CONTROL) {
            f.write_str("Ctrl+")?;
        }
        if self.mods.contains(KeyModifiers::ALT) {
            f.write_str("Alt+")?;
        }
        if self.mods.contains(KeyModifiers::SHIFT) && !matches!(self.code, KeyCode::Char(_)) {
            f.write_str("Shift+")?;
        }
        match self.code {
            KeyCode::Char(' ') => f.write_str("Space"),
            KeyCode::Char(c) => write!(f, "{c}"),
            KeyCode::Enter => f.write_str("Enter"),
            KeyCode::Esc => f.write_str("Esc"),
            KeyCode::Tab => f.write_str("Tab"),
            KeyCode::BackTab => f.write_str("Shift+Tab"),
            KeyCode::Backspace => f.write_str("Backspace"),
            KeyCode::Delete => f.write_str("Del"),
            KeyCode::Left => f.write_str("←"),
            KeyCode::Right => f.write_str("→"),
            KeyCode::Up => f.write_str("↑"),
            KeyCode::Down => f.write_str("↓"),
            KeyCode::Home => f.write_str("Home"),
            KeyCode::End => f.write_str("End"),
            KeyCode::PageUp => f.write_str("PgUp"),
            KeyCode::PageDown => f.write_str("PgDn"),
            KeyCode::F(n) => write!(f, "F{n}"),
            other => write!(f, "{other:?}"),
        }
    }
}

/// A pointer event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mouse {
    /// What happened.
    pub kind: MouseKind,
    /// Where, in terminal cells.
    pub pos: Position,
    /// The modifiers held.
    pub mods: KeyModifiers,
}

/// Pointer event kinds. Primary button is `Down`/`Up`/`Drag`; the secondary
/// button is `Secondary`/`SecondaryUp`; the middle button is dropped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseKind {
    /// Pointer moved with no button held.
    Move,
    /// Primary button pressed.
    Down,
    /// Primary button released.
    Up,
    /// Pointer moved with the primary button held.
    Drag,
    /// Secondary button pressed.
    Secondary,
    /// Secondary button released.
    SecondaryUp,
    /// Wheel motion on an axis; positive is down / right.
    Wheel(Axis, i16),
}

/// A scroll axis.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Axis {
    /// Vertical.
    V,
    /// Horizontal.
    H,
}

impl Input {
    /// Normalise a crossterm event (§3.3 step 1): key releases and unmapped
    /// buttons are dropped; `MouseEventKind` is matched exhaustively so a
    /// new upstream variant is a compile error (§22 R‑15).
    pub fn from_crossterm(ev: ratatui_crossterm::crossterm::event::Event) -> Option<Self> {
        use ratatui_crossterm::crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
        match ev {
            Event::Key(k) => {
                if k.is_release() {
                    return None;
                }
                Some(Input::Key(Key {
                    code: k.code,
                    mods: k.modifiers,
                }))
            }
            Event::Mouse(MouseEvent {
                kind,
                column,
                row,
                modifiers,
            }) => {
                let kind = match kind {
                    MouseEventKind::Moved => MouseKind::Move,
                    MouseEventKind::Down(MouseButton::Left) => MouseKind::Down,
                    MouseEventKind::Up(MouseButton::Left) => MouseKind::Up,
                    MouseEventKind::Drag(MouseButton::Left) => MouseKind::Drag,
                    MouseEventKind::Down(MouseButton::Right) => MouseKind::Secondary,
                    MouseEventKind::Up(MouseButton::Right) => MouseKind::SecondaryUp,
                    MouseEventKind::Drag(MouseButton::Right | MouseButton::Middle)
                    | MouseEventKind::Down(MouseButton::Middle)
                    | MouseEventKind::Up(MouseButton::Middle) => return None,
                    MouseEventKind::ScrollUp => MouseKind::Wheel(Axis::V, -1),
                    MouseEventKind::ScrollDown => MouseKind::Wheel(Axis::V, 1),
                    MouseEventKind::ScrollLeft => MouseKind::Wheel(Axis::H, -1),
                    MouseEventKind::ScrollRight => MouseKind::Wheel(Axis::H, 1),
                };
                Some(Input::Mouse(Mouse {
                    kind,
                    pos: Position::new(column, row),
                    mods: modifiers,
                }))
            }
            Event::Resize(w, h) => Some(Input::Resize(w, h)),
            Event::Paste(s) => Some(Input::Paste(s)),
            Event::FocusGained | Event::FocusLost => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_crossterm::crossterm::event::{
        Event, KeyEvent, KeyEventKind, KeyEventState, MouseButton, MouseEvent, MouseEventKind,
    };

    fn key_event(code: KeyCode, kind: KeyEventKind) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind,
            state: KeyEventState::NONE,
        })
    }

    fn mouse(kind: MouseEventKind, mods: KeyModifiers) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: 3,
            row: 4,
            modifiers: mods,
        })
    }

    #[test]
    fn key_release_is_dropped() {
        // synthesised: without keyboard-enhancement flags no Unix terminal
        // ever produces a release (§22.2 item 6)
        assert_eq!(
            Input::from_crossterm(key_event(KeyCode::Enter, KeyEventKind::Release)),
            None
        );
        assert!(Input::from_crossterm(key_event(KeyCode::Enter, KeyEventKind::Press)).is_some());
        assert!(Input::from_crossterm(key_event(KeyCode::Enter, KeyEventKind::Repeat)).is_some());
    }

    #[test]
    fn unmapped_mouse_button_is_dropped() {
        assert_eq!(
            Input::from_crossterm(mouse(
                MouseEventKind::Down(MouseButton::Middle),
                KeyModifiers::NONE
            )),
            None
        );
        assert_eq!(
            Input::from_crossterm(mouse(
                MouseEventKind::Drag(MouseButton::Right),
                KeyModifiers::NONE
            )),
            None
        );
    }

    #[test]
    fn mouse_carries_modifiers() {
        let Some(Input::Mouse(m)) = Input::from_crossterm(mouse(
            MouseEventKind::Down(MouseButton::Left),
            KeyModifiers::SHIFT,
        )) else {
            panic!("expected a mouse input");
        };
        assert_eq!(m.mods, KeyModifiers::SHIFT);
        assert_eq!(m.pos, Position::new(3, 4));
        assert_eq!(m.kind, MouseKind::Down);
    }

    #[test]
    fn chord_hashes_by_code_and_mods() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Chord::key(KeyCode::Char('a')));
        set.insert(Chord::with(KeyCode::Char('a'), KeyModifiers::CONTROL));
        set.insert(Chord::key(KeyCode::Char('a')));
        assert_eq!(set.len(), 2);
        assert!(set.contains(&Chord::key(KeyCode::Char('a'))));
    }

    #[test]
    fn secondary_up_is_modelled() {
        let Some(Input::Mouse(m)) = Input::from_crossterm(mouse(
            MouseEventKind::Up(MouseButton::Right),
            KeyModifiers::NONE,
        )) else {
            panic!("expected a mouse input");
        };
        assert_eq!(m.kind, MouseKind::SecondaryUp);
    }

    #[test]
    fn wheel_carries_axis_and_delta() {
        let cases = [
            (MouseEventKind::ScrollUp, Axis::V, -1),
            (MouseEventKind::ScrollDown, Axis::V, 1),
            (MouseEventKind::ScrollLeft, Axis::H, -1),
            (MouseEventKind::ScrollRight, Axis::H, 1),
        ];
        for (kind, axis, delta) in cases {
            let Some(Input::Mouse(m)) = Input::from_crossterm(mouse(kind, KeyModifiers::NONE))
            else {
                panic!("expected a mouse input");
            };
            assert_eq!(m.kind, MouseKind::Wheel(axis, delta));
        }
    }

    #[test]
    fn chord_matches_shifted_chars_and_display_is_readable() {
        let k = Key {
            code: KeyCode::Char('A'),
            mods: KeyModifiers::SHIFT,
        };
        assert!(Chord::key(KeyCode::Char('A')).matches(&k));
        assert!(k.is(KeyCode::Char('A')));
        assert_eq!(k.bare_char(), Some('A'));
        assert_eq!(
            Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL).to_string(),
            "Ctrl+s"
        );
        assert_eq!(Chord::key(KeyCode::Esc).to_string(), "Esc");
    }

    #[test]
    fn paste_debug_is_redacted_and_input_is_droppable() {
        let input = Input::Paste("hunter2".to_owned());
        assert!(!format!("{input:?}").contains("hunter2"));
        assert!(core::mem::needs_drop::<Input>());
    }
}
