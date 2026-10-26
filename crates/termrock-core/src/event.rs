use core::fmt;

pub use crate::keys::{KeyCode, KeyModifiers};
use ratatui_core::layout::Position;

#[derive(Clone, PartialEq, Eq)]
pub enum Input {
    Key(Key),
    Mouse(Mouse),
    Resize(u16, u16),
    Paste(String),
    Tick,
}

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq)]
pub enum RoutedInput {
    Input(Input),

    MiddleClick { pos: Position, mods: KeyModifiers },
}

impl From<Input> for RoutedInput {
    fn from(input: Input) -> Self {
        Self::Input(input)
    }
}

impl fmt::Debug for RoutedInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(input) => f.debug_tuple("Input").field(input).finish(),
            Self::MiddleClick { pos, mods } => f
                .debug_struct("MiddleClick")
                .field("pos", pos)
                .field("mods", mods)
                .finish(),
        }
    }
}

impl RoutedInput {
    #[cfg(feature = "crossterm")]
    pub fn from_crossterm(ev: ratatui_crossterm::crossterm::event::Event) -> Option<Self> {
        use ratatui_crossterm::crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
        match ev {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Middle),
                column,
                row,
                modifiers,
            }) => Some(Self::MiddleClick {
                pos: Position::new(column, row),
                mods: modifiers.into(),
            }),
            other => Input::from_crossterm(other).map(Self::Input),
        }
    }
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Key {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

impl Key {
    pub const fn chord(&self) -> Chord {
        Chord {
            code: self.code,
            mods: self.mods,
        }
    }

    pub fn is(&self, c: KeyCode) -> bool {
        self.code == c && self.mods.difference(KeyModifiers::SHIFT).is_empty()
    }

    pub const fn ctrl(&self) -> bool {
        self.mods.contains(KeyModifiers::CONTROL)
    }

    pub const fn alt(&self) -> bool {
        self.mods.contains(KeyModifiers::ALT)
    }

    pub const fn shift(&self) -> bool {
        self.mods.contains(KeyModifiers::SHIFT)
    }

    pub const fn bare_char(&self) -> Option<char> {
        match self.code {
            KeyCode::Char(c) if self.mods.difference(KeyModifiers::SHIFT).is_empty() => Some(c),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Chord {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

impl Chord {
    pub const fn key(c: KeyCode) -> Chord {
        Chord {
            code: c,
            mods: KeyModifiers::NONE,
        }
    }

    pub const fn with(c: KeyCode, m: KeyModifiers) -> Chord {
        Chord { code: c, mods: m }
    }

    pub const fn is_bare_char(&self) -> bool {
        matches!(self.code, KeyCode::Char(_))
            && self.mods.difference(KeyModifiers::SHIFT).is_empty()
    }

    pub fn effective_eq(&self, other: &Chord) -> bool {
        if self.code != other.code {
            return false;
        }
        match self.code {
            KeyCode::Char(_) | KeyCode::BackTab => {
                self.mods.difference(KeyModifiers::SHIFT)
                    == other.mods.difference(KeyModifiers::SHIFT)
            }
            _ => self.mods == other.mods,
        }
    }

    pub fn matches(&self, k: &Key) -> bool {
        self.effective_eq(&k.chord())
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mouse {
    pub kind: MouseKind,
    pub pos: Position,
    pub mods: KeyModifiers,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseKind {
    Move,
    Down,
    Up,
    Drag,
    Secondary,
    SecondaryUp,
    Wheel(Axis, i16),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Axis {
    V,
    H,
}

impl Input {
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
            Self::from_bits_retain(value.bits())
        }
    }

    impl From<KeyModifiers> for backend::KeyModifiers {
        fn from(value: KeyModifiers) -> Self {
            Self::from_bits_retain(value.bits())
        }
    }
}
