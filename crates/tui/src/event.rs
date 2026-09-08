//! Raw input at the runtime boundary (`COMPONENT_ARCHITECTURE.md` §6.1).
//!
//! `Input` never touches a component directly: the runtime resolves it
//! against the last frame's registry and focus ring and delivers
//! [`Intent`](crate::intent::Intent)s instead. Keyboard types are owned by
//! this crate. Backend normalization exists only with the `crossterm` feature.

use core::fmt;

pub use crate::keys::{KeyCode, KeyModifiers};
use ratatui_core::layout::Position;

/// One normalised input event.
#[derive(Clone, PartialEq, Eq)]
pub enum Input {
    /// A key press or repeat (releases are dropped at normalisation).
    Key(Key),
    /// A pointer event with modifiers.
    Mouse(Mouse),
    /// The terminal was resized to `(columns, rows)`.
    Resize(u16, u16),
    /// Bracketed paste text.
    Paste(String),
    /// Explicit update trigger at unchanged runtime time; elapsed time comes from `Runtime::advance_to`.
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
            Input::Paste(text) => f.debug_struct("Paste").field("len", &text.len()).finish(),
            Input::Tick => f.write_str("Tick"),
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        if let Input::Paste(text) = self {
            crate::secret::wipe_string(core::mem::take(text));
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
/// `PartialEq`/`Hash` are the derived structural forms over the backend-neutral
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
    #[cfg(feature = "crossterm")]
    pub fn from_crossterm(ev: ratatui_crossterm::crossterm::event::Event) -> Option<Self> {
        use ratatui_crossterm::crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
        match ev {
            Event::Key(k) => {
                if k.is_release() {
                    return None;
                }
                Some(Input::Key(Key {
                    code: k.code.into(),
                    mods: k.modifiers.into(),
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
                    mods: modifiers.into(),
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
    #[cfg(feature = "crossterm")]
    use ratatui_crossterm::crossterm::event::{
        Event, KeyEvent, KeyEventKind, KeyEventState, MouseButton, MouseEvent, MouseEventKind,
    };

    #[cfg(feature = "crossterm")]
    fn key_event(code: KeyCode, kind: KeyEventKind) -> Event {
        Event::Key(KeyEvent {
            code: code.into(),
            modifiers: KeyModifiers::NONE.into(),
            kind,
            state: KeyEventState::NONE,
        })
    }

    #[cfg(feature = "crossterm")]
    fn mouse(kind: MouseEventKind, mods: KeyModifiers) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: 3,
            row: 4,
            modifiers: mods.into(),
        })
    }

    #[test]
    #[cfg(feature = "crossterm")]
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
    fn paste_debug_redacts_the_payload() {
        let debug = format!("{:?}", Input::Paste("swordfish".to_owned()));
        assert!(debug.contains("len"));
        assert!(!debug.contains("swordfish"));
    }

    #[test]
    #[cfg(feature = "crossterm")]
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
    #[cfg(feature = "crossterm")]
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
    #[cfg(feature = "crossterm")]
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
    #[cfg(feature = "crossterm")]
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
}

#[cfg(feature = "crossterm")]
mod backend_keys {
    use crate::keys::{KeyCode, KeyModifiers, MediaKeyCode, ModifierKeyCode};
    use ratatui_crossterm::crossterm::event as backend;

    impl From<backend::MediaKeyCode> for MediaKeyCode {
        fn from(value: backend::MediaKeyCode) -> Self {
            match value {
                backend::MediaKeyCode::Play => Self::Play,
                backend::MediaKeyCode::Pause => Self::Pause,
                backend::MediaKeyCode::PlayPause => Self::PlayPause,
                backend::MediaKeyCode::Reverse => Self::Reverse,
                backend::MediaKeyCode::Stop => Self::Stop,
                backend::MediaKeyCode::FastForward => Self::FastForward,
                backend::MediaKeyCode::Rewind => Self::Rewind,
                backend::MediaKeyCode::TrackNext => Self::TrackNext,
                backend::MediaKeyCode::TrackPrevious => Self::TrackPrevious,
                backend::MediaKeyCode::Record => Self::Record,
                backend::MediaKeyCode::LowerVolume => Self::LowerVolume,
                backend::MediaKeyCode::RaiseVolume => Self::RaiseVolume,
                backend::MediaKeyCode::MuteVolume => Self::MuteVolume,
            }
        }
    }

    impl From<MediaKeyCode> for backend::MediaKeyCode {
        fn from(value: MediaKeyCode) -> Self {
            match value {
                MediaKeyCode::Play => Self::Play,
                MediaKeyCode::Pause => Self::Pause,
                MediaKeyCode::PlayPause => Self::PlayPause,
                MediaKeyCode::Reverse => Self::Reverse,
                MediaKeyCode::Stop => Self::Stop,
                MediaKeyCode::FastForward => Self::FastForward,
                MediaKeyCode::Rewind => Self::Rewind,
                MediaKeyCode::TrackNext => Self::TrackNext,
                MediaKeyCode::TrackPrevious => Self::TrackPrevious,
                MediaKeyCode::Record => Self::Record,
                MediaKeyCode::LowerVolume => Self::LowerVolume,
                MediaKeyCode::RaiseVolume => Self::RaiseVolume,
                MediaKeyCode::MuteVolume => Self::MuteVolume,
            }
        }
    }

    impl From<backend::ModifierKeyCode> for ModifierKeyCode {
        fn from(value: backend::ModifierKeyCode) -> Self {
            match value {
                backend::ModifierKeyCode::LeftShift => Self::LeftShift,
                backend::ModifierKeyCode::LeftControl => Self::LeftControl,
                backend::ModifierKeyCode::LeftAlt => Self::LeftAlt,
                backend::ModifierKeyCode::LeftSuper => Self::LeftSuper,
                backend::ModifierKeyCode::LeftHyper => Self::LeftHyper,
                backend::ModifierKeyCode::LeftMeta => Self::LeftMeta,
                backend::ModifierKeyCode::RightShift => Self::RightShift,
                backend::ModifierKeyCode::RightControl => Self::RightControl,
                backend::ModifierKeyCode::RightAlt => Self::RightAlt,
                backend::ModifierKeyCode::RightSuper => Self::RightSuper,
                backend::ModifierKeyCode::RightHyper => Self::RightHyper,
                backend::ModifierKeyCode::RightMeta => Self::RightMeta,
                backend::ModifierKeyCode::IsoLevel3Shift => Self::IsoLevel3Shift,
                backend::ModifierKeyCode::IsoLevel5Shift => Self::IsoLevel5Shift,
            }
        }
    }

    impl From<ModifierKeyCode> for backend::ModifierKeyCode {
        fn from(value: ModifierKeyCode) -> Self {
            match value {
                ModifierKeyCode::LeftShift => Self::LeftShift,
                ModifierKeyCode::LeftControl => Self::LeftControl,
                ModifierKeyCode::LeftAlt => Self::LeftAlt,
                ModifierKeyCode::LeftSuper => Self::LeftSuper,
                ModifierKeyCode::LeftHyper => Self::LeftHyper,
                ModifierKeyCode::LeftMeta => Self::LeftMeta,
                ModifierKeyCode::RightShift => Self::RightShift,
                ModifierKeyCode::RightControl => Self::RightControl,
                ModifierKeyCode::RightAlt => Self::RightAlt,
                ModifierKeyCode::RightSuper => Self::RightSuper,
                ModifierKeyCode::RightHyper => Self::RightHyper,
                ModifierKeyCode::RightMeta => Self::RightMeta,
                ModifierKeyCode::IsoLevel3Shift => Self::IsoLevel3Shift,
                ModifierKeyCode::IsoLevel5Shift => Self::IsoLevel5Shift,
            }
        }
    }

    impl From<backend::KeyCode> for KeyCode {
        fn from(value: backend::KeyCode) -> Self {
            match value {
                backend::KeyCode::Backspace => Self::Backspace,
                backend::KeyCode::Enter => Self::Enter,
                backend::KeyCode::Left => Self::Left,
                backend::KeyCode::Right => Self::Right,
                backend::KeyCode::Up => Self::Up,
                backend::KeyCode::Down => Self::Down,
                backend::KeyCode::Home => Self::Home,
                backend::KeyCode::End => Self::End,
                backend::KeyCode::PageUp => Self::PageUp,
                backend::KeyCode::PageDown => Self::PageDown,
                backend::KeyCode::Tab => Self::Tab,
                backend::KeyCode::BackTab => Self::BackTab,
                backend::KeyCode::Delete => Self::Delete,
                backend::KeyCode::Insert => Self::Insert,
                backend::KeyCode::Null => Self::Null,
                backend::KeyCode::Esc => Self::Esc,
                backend::KeyCode::CapsLock => Self::CapsLock,
                backend::KeyCode::ScrollLock => Self::ScrollLock,
                backend::KeyCode::NumLock => Self::NumLock,
                backend::KeyCode::PrintScreen => Self::PrintScreen,
                backend::KeyCode::Pause => Self::Pause,
                backend::KeyCode::Menu => Self::Menu,
                backend::KeyCode::KeypadBegin => Self::KeypadBegin,
                backend::KeyCode::F(value) => Self::F(value),
                backend::KeyCode::Char(value) => Self::Char(value),
                backend::KeyCode::Media(value) => Self::Media(value.into()),
                backend::KeyCode::Modifier(value) => Self::Modifier(value.into()),
            }
        }
    }

    impl From<KeyCode> for backend::KeyCode {
        fn from(value: KeyCode) -> Self {
            match value {
                KeyCode::Backspace => Self::Backspace,
                KeyCode::Enter => Self::Enter,
                KeyCode::Left => Self::Left,
                KeyCode::Right => Self::Right,
                KeyCode::Up => Self::Up,
                KeyCode::Down => Self::Down,
                KeyCode::Home => Self::Home,
                KeyCode::End => Self::End,
                KeyCode::PageUp => Self::PageUp,
                KeyCode::PageDown => Self::PageDown,
                KeyCode::Tab => Self::Tab,
                KeyCode::BackTab => Self::BackTab,
                KeyCode::Delete => Self::Delete,
                KeyCode::Insert => Self::Insert,
                KeyCode::Null => Self::Null,
                KeyCode::Esc => Self::Esc,
                KeyCode::CapsLock => Self::CapsLock,
                KeyCode::ScrollLock => Self::ScrollLock,
                KeyCode::NumLock => Self::NumLock,
                KeyCode::PrintScreen => Self::PrintScreen,
                KeyCode::Pause => Self::Pause,
                KeyCode::Menu => Self::Menu,
                KeyCode::KeypadBegin => Self::KeypadBegin,
                KeyCode::F(value) => Self::F(value),
                KeyCode::Char(value) => Self::Char(value),
                KeyCode::Media(value) => Self::Media(value.into()),
                KeyCode::Modifier(value) => Self::Modifier(value.into()),
            }
        }
    }

    impl From<backend::KeyModifiers> for KeyModifiers {
        fn from(value: backend::KeyModifiers) -> Self {
            // Retain reserved bits as well as the six defined modifiers.
            Self::from_bits_retain(value.bits())
        }
    }

    impl From<KeyModifiers> for backend::KeyModifiers {
        fn from(value: KeyModifiers) -> Self {
            // Retain reserved bits as well as the six defined modifiers.
            Self::from_bits_retain(value.bits())
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{Input, Key};

        #[test]
        fn every_backend_key_preserves_its_identity() {
            let cases = [
                (backend::KeyCode::Backspace, KeyCode::Backspace),
                (backend::KeyCode::Enter, KeyCode::Enter),
                (backend::KeyCode::Left, KeyCode::Left),
                (backend::KeyCode::Right, KeyCode::Right),
                (backend::KeyCode::Up, KeyCode::Up),
                (backend::KeyCode::Down, KeyCode::Down),
                (backend::KeyCode::Home, KeyCode::Home),
                (backend::KeyCode::End, KeyCode::End),
                (backend::KeyCode::PageUp, KeyCode::PageUp),
                (backend::KeyCode::PageDown, KeyCode::PageDown),
                (backend::KeyCode::Tab, KeyCode::Tab),
                (backend::KeyCode::BackTab, KeyCode::BackTab),
                (backend::KeyCode::Delete, KeyCode::Delete),
                (backend::KeyCode::Insert, KeyCode::Insert),
                (backend::KeyCode::Null, KeyCode::Null),
                (backend::KeyCode::Esc, KeyCode::Esc),
                (backend::KeyCode::CapsLock, KeyCode::CapsLock),
                (backend::KeyCode::ScrollLock, KeyCode::ScrollLock),
                (backend::KeyCode::NumLock, KeyCode::NumLock),
                (backend::KeyCode::PrintScreen, KeyCode::PrintScreen),
                (backend::KeyCode::Pause, KeyCode::Pause),
                (backend::KeyCode::Menu, KeyCode::Menu),
                (backend::KeyCode::KeypadBegin, KeyCode::KeypadBegin),
                (backend::KeyCode::F(0), KeyCode::F(0)),
                (backend::KeyCode::F(255), KeyCode::F(255)),
                (backend::KeyCode::Char('λ'), KeyCode::Char('λ')),
                (backend::KeyCode::Char('\0'), KeyCode::Char('\0')),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::Play),
                    KeyCode::Media(MediaKeyCode::Play),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::Pause),
                    KeyCode::Media(MediaKeyCode::Pause),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::PlayPause),
                    KeyCode::Media(MediaKeyCode::PlayPause),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::Reverse),
                    KeyCode::Media(MediaKeyCode::Reverse),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::Stop),
                    KeyCode::Media(MediaKeyCode::Stop),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::FastForward),
                    KeyCode::Media(MediaKeyCode::FastForward),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::Rewind),
                    KeyCode::Media(MediaKeyCode::Rewind),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::TrackNext),
                    KeyCode::Media(MediaKeyCode::TrackNext),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::TrackPrevious),
                    KeyCode::Media(MediaKeyCode::TrackPrevious),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::Record),
                    KeyCode::Media(MediaKeyCode::Record),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::LowerVolume),
                    KeyCode::Media(MediaKeyCode::LowerVolume),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::RaiseVolume),
                    KeyCode::Media(MediaKeyCode::RaiseVolume),
                ),
                (
                    backend::KeyCode::Media(backend::MediaKeyCode::MuteVolume),
                    KeyCode::Media(MediaKeyCode::MuteVolume),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::LeftShift),
                    KeyCode::Modifier(ModifierKeyCode::LeftShift),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::LeftControl),
                    KeyCode::Modifier(ModifierKeyCode::LeftControl),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::LeftAlt),
                    KeyCode::Modifier(ModifierKeyCode::LeftAlt),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::LeftSuper),
                    KeyCode::Modifier(ModifierKeyCode::LeftSuper),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::LeftHyper),
                    KeyCode::Modifier(ModifierKeyCode::LeftHyper),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::LeftMeta),
                    KeyCode::Modifier(ModifierKeyCode::LeftMeta),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::RightShift),
                    KeyCode::Modifier(ModifierKeyCode::RightShift),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::RightControl),
                    KeyCode::Modifier(ModifierKeyCode::RightControl),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::RightAlt),
                    KeyCode::Modifier(ModifierKeyCode::RightAlt),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::RightSuper),
                    KeyCode::Modifier(ModifierKeyCode::RightSuper),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::RightHyper),
                    KeyCode::Modifier(ModifierKeyCode::RightHyper),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::RightMeta),
                    KeyCode::Modifier(ModifierKeyCode::RightMeta),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::IsoLevel3Shift),
                    KeyCode::Modifier(ModifierKeyCode::IsoLevel3Shift),
                ),
                (
                    backend::KeyCode::Modifier(backend::ModifierKeyCode::IsoLevel5Shift),
                    KeyCode::Modifier(ModifierKeyCode::IsoLevel5Shift),
                ),
            ];
            for (raw, owned) in cases {
                assert_eq!(KeyCode::from(raw), owned);
                assert_eq!(backend::KeyCode::from(owned), raw);
                for kind in [backend::KeyEventKind::Press, backend::KeyEventKind::Repeat] {
                    let event =
                        backend::KeyEvent::new_with_kind(raw, backend::KeyModifiers::NONE, kind);
                    assert_eq!(
                        Input::from_crossterm(backend::Event::Key(event)),
                        Some(Input::Key(Key {
                            code: owned,
                            mods: KeyModifiers::NONE
                        }))
                    );
                }
                let release = backend::KeyEvent::new_with_kind(
                    raw,
                    backend::KeyModifiers::NONE,
                    backend::KeyEventKind::Release,
                );
                assert_eq!(Input::from_crossterm(backend::Event::Key(release)), None);
            }
        }

        #[test]
        fn modifier_names_and_every_bit_pattern_are_lossless() {
            let names = [
                (backend::KeyModifiers::SHIFT, KeyModifiers::SHIFT),
                (backend::KeyModifiers::CONTROL, KeyModifiers::CONTROL),
                (backend::KeyModifiers::ALT, KeyModifiers::ALT),
                (backend::KeyModifiers::SUPER, KeyModifiers::SUPER),
                (backend::KeyModifiers::HYPER, KeyModifiers::HYPER),
                (backend::KeyModifiers::META, KeyModifiers::META),
                (backend::KeyModifiers::NONE, KeyModifiers::NONE),
            ];
            for (raw, owned) in names {
                assert_eq!(KeyModifiers::from(raw), owned);
            }
            for bits in u8::MIN..=u8::MAX {
                let raw = backend::KeyModifiers::from_bits_retain(bits);
                let owned = KeyModifiers::from(raw);
                assert_eq!(owned.bits(), bits);
                assert_eq!(backend::KeyModifiers::from(owned), raw);
                let event = backend::KeyEvent::new(backend::KeyCode::Enter, raw);
                assert_eq!(
                    Input::from_crossterm(backend::Event::Key(event)),
                    Some(Input::Key(Key {
                        code: KeyCode::Enter,
                        mods: owned
                    }))
                );
            }
        }
    }
}
