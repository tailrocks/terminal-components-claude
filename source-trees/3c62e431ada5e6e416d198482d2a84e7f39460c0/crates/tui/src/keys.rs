//! Backend-neutral keyboard vocabulary. Values describe keys, never terminal I/O.

/// A media control key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediaKeyCode {
    /// Play key.
    Play,
    /// Pause key.
    Pause,
    /// Play pause key.
    PlayPause,
    /// Reverse key.
    Reverse,
    /// Stop key.
    Stop,
    /// Fast forward key.
    FastForward,
    /// Rewind key.
    Rewind,
    /// Track next key.
    TrackNext,
    /// Track previous key.
    TrackPrevious,
    /// Record key.
    Record,
    /// Lower volume key.
    LowerVolume,
    /// Raise volume key.
    RaiseVolume,
    /// Mute volume key.
    MuteVolume,
}

/// A modifier key pressed as a key in its own right.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModifierKeyCode {
    /// Left shift key.
    LeftShift,
    /// Left control key.
    LeftControl,
    /// Left alt key.
    LeftAlt,
    /// Left super key.
    LeftSuper,
    /// Left hyper key.
    LeftHyper,
    /// Left meta key.
    LeftMeta,
    /// Right shift key.
    RightShift,
    /// Right control key.
    RightControl,
    /// Right alt key.
    RightAlt,
    /// Right super key.
    RightSuper,
    /// Right hyper key.
    RightHyper,
    /// Right meta key.
    RightMeta,
    /// Iso level3 shift key.
    IsoLevel3Shift,
    /// Iso level5 shift key.
    IsoLevel5Shift,
}

/// A keyboard key, independent of the terminal backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCode {
    /// Backspace key.
    Backspace,
    /// Enter key.
    Enter,
    /// Left key.
    Left,
    /// Right key.
    Right,
    /// Up key.
    Up,
    /// Down key.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page up key.
    PageUp,
    /// Page down key.
    PageDown,
    /// Tab key.
    Tab,
    /// Back tab key.
    BackTab,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Null key.
    Null,
    /// Esc key.
    Esc,
    /// Caps lock key.
    CapsLock,
    /// Scroll lock key.
    ScrollLock,
    /// Num lock key.
    NumLock,
    /// Print screen key.
    PrintScreen,
    /// Pause key.
    Pause,
    /// Menu key.
    Menu,
    /// Keypad begin key.
    KeypadBegin,
    /// Function key number.
    F(u8),
    /// Unicode character.
    Char(char),
    /// Media control.
    Media(MediaKeyCode),
    /// Physical modifier key.
    Modifier(ModifierKeyCode),
}

bitflags::bitflags! {
    /// Keyboard modifiers held during a key or pointer event.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct KeyModifiers: u8 {
        /// Shift modifier.
        const SHIFT = 1;
        /// Control modifier.
        const CONTROL = 2;
        /// Alt modifier.
        const ALT = 4;
        /// Super modifier.
        const SUPER = 8;
        /// Hyper modifier.
        const HYPER = 16;
        /// Meta modifier.
        const META = 32;
        /// No modifiers.
        const NONE = 0;
    }
}
