#![doc = r" Backend-neutral keyboard vocabulary. Values describe keys, never terminal I/O."]

#[doc = r" A media control key."]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediaKeyCode {
    #[doc = r" Play key."]
    Play,
    #[doc = r" Pause key."]
    Pause,
    #[doc = r" Play pause key."]
    PlayPause,
    #[doc = r" Reverse key."]
    Reverse,
    #[doc = r" Stop key."]
    Stop,
    #[doc = r" Fast forward key."]
    FastForward,
    #[doc = r" Rewind key."]
    Rewind,
    #[doc = r" Track next key."]
    TrackNext,
    #[doc = r" Track previous key."]
    TrackPrevious,
    #[doc = r" Record key."]
    Record,
    #[doc = r" Lower volume key."]
    LowerVolume,
    #[doc = r" Raise volume key."]
    RaiseVolume,
    #[doc = r" Mute volume key."]
    MuteVolume,
}

#[doc = r" A modifier key pressed as a key in its own right."]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModifierKeyCode {
    #[doc = r" Left shift key."]
    LeftShift,
    #[doc = r" Left control key."]
    LeftControl,
    #[doc = r" Left alt key."]
    LeftAlt,
    #[doc = r" Left super key."]
    LeftSuper,
    #[doc = r" Left hyper key."]
    LeftHyper,
    #[doc = r" Left meta key."]
    LeftMeta,
    #[doc = r" Right shift key."]
    RightShift,
    #[doc = r" Right control key."]
    RightControl,
    #[doc = r" Right alt key."]
    RightAlt,
    #[doc = r" Right super key."]
    RightSuper,
    #[doc = r" Right hyper key."]
    RightHyper,
    #[doc = r" Right meta key."]
    RightMeta,
    #[doc = r" Iso level3 shift key."]
    IsoLevel3Shift,
    #[doc = r" Iso level5 shift key."]
    IsoLevel5Shift,
}

#[doc = r" A keyboard key, independent of the terminal backend."]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCode {
    #[doc = r" Backspace key."]
    Backspace,
    #[doc = r" Enter key."]
    Enter,
    #[doc = r" Left key."]
    Left,
    #[doc = r" Right key."]
    Right,
    #[doc = r" Up key."]
    Up,
    #[doc = r" Down key."]
    Down,
    #[doc = r" Home key."]
    Home,
    #[doc = r" End key."]
    End,
    #[doc = r" Page up key."]
    PageUp,
    #[doc = r" Page down key."]
    PageDown,
    #[doc = r" Tab key."]
    Tab,
    #[doc = r" Back tab key."]
    BackTab,
    #[doc = r" Delete key."]
    Delete,
    #[doc = r" Insert key."]
    Insert,
    #[doc = r" Null key."]
    Null,
    #[doc = r" Esc key."]
    Esc,
    #[doc = r" Caps lock key."]
    CapsLock,
    #[doc = r" Scroll lock key."]
    ScrollLock,
    #[doc = r" Num lock key."]
    NumLock,
    #[doc = r" Print screen key."]
    PrintScreen,
    #[doc = r" Pause key."]
    Pause,
    #[doc = r" Menu key."]
    Menu,
    #[doc = r" Keypad begin key."]
    KeypadBegin,
    #[doc = r" Function key number."]
    F(u8),
    #[doc = r" Unicode character."]
    Char(char),
    #[doc = r" Media control."]
    Media(MediaKeyCode),
    #[doc = r" Physical modifier key."]
    Modifier(ModifierKeyCode),
}

bitflags::bitflags! {
    #[doc = r" Keyboard modifiers held during a key or pointer event."]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct KeyModifiers: u8 {
        #[doc = r" Shift modifier."]
        const SHIFT = 1;
        #[doc = r" Control modifier."]
        const CONTROL = 2;
        #[doc = r" Alt modifier."]
        const ALT = 4;
        #[doc = r" Super modifier."]
        const SUPER = 8;
        #[doc = r" Hyper modifier."]
        const HYPER = 16;
        #[doc = r" Meta modifier."]
        const META = 32;
        #[doc = r" No modifiers."]
        const NONE = 0;
    }
}

#[doc = r" The phase in which an application chord is matched (§3.3 steps 2 and 8)."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum KeyPhase {
    #[doc = r" Before dispatch; a bare `Char` chord is skipped while the focused"]
    #[doc = r" control or an admitted fallback editor swallows typing."]
    Capture,
    #[doc = r" After dispatch, for keys no component consumed."]
    Bubble,
}
