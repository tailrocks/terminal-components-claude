//! Ported from PR #26 `src/core/keyboard.rs` (commit `f4867e9b5`). The

pub mod keyboard {
    use std::ffi::OsString;
    use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

    use crate::event::{Input, Key};
    use crate::keys::{KeyCode, KeyModifiers};
    use crate::platform::ClientPlatform;

    pub const DISAMBIGUATE: u8 = 0b1;

    pub const REPORT_ALTERNATE_KEYS: u8 = 0b100;

    pub const FLAGS: u8 = DISAMBIGUATE | REPORT_ALTERNATE_KEYS;

    pub const QUERY: &str = "\x1b[?u";

    pub const POP: &str = "\x1b[<u";

    pub fn push(flags: u8) -> String {
        format!("\x1b[>{flags}u")
    }

    pub fn parse_flags_reply(reply: &[u8]) -> Option<u8> {
        let body = reply.strip_prefix(b"\x1b[?")?.strip_suffix(b"u")?;
        if body.is_empty() || !body.iter().all(u8::is_ascii_digit) {
            return None;
        }
        std::str::from_utf8(body).ok()?.parse().ok()
    }

    pub fn modifiers(param: u16) -> KeyModifiers {
        let bits = param.saturating_sub(1);
        let mut m = KeyModifiers::NONE;
        for (bit, flag) in [
            (1, KeyModifiers::SHIFT),
            (2, KeyModifiers::ALT),
            (4, KeyModifiers::CONTROL),
            (8, KeyModifiers::SUPER),
        ] {
            if bits & bit != 0 {
                m |= flag;
            }
        }
        m
    }

    pub fn modifier_param(mods: KeyModifiers) -> u16 {
        let mut bits = 0;
        for (bit, flag) in [
            (1, KeyModifiers::SHIFT),
            (2, KeyModifiers::ALT),
            (4, KeyModifiers::CONTROL),
            (8, KeyModifiers::SUPER),
        ] {
            if mods.contains(flag) {
                bits |= bit;
            }
        }
        bits + 1
    }

    pub fn encode(key: &Key) -> Option<String> {
        let KeyCode::Char(c) = key.code else {
            return None;
        };
        let lower = c.to_lowercase().next().unwrap_or(c);
        let param = modifier_param(key.mods);
        Some(if param == 1 {
            format!("\x1b[{}u", lower as u32)
        } else {
            format!("\x1b[{};{param}u", lower as u32)
        })
    }

    pub fn decode(seq: &[u8]) -> Option<Key> {
        let body = std::str::from_utf8(seq.strip_prefix(b"\x1b[")?.strip_suffix(b"u")?).ok()?;
        if body.starts_with(['?', '>', '<', '=']) {
            return None;
        }
        let mut fields = body.split(';');
        let code: u32 = fields.next()?.split(':').next()?.parse().ok()?;
        let param: u16 = match fields.next() {
            Some(m) => m.split(':').next()?.parse().ok()?,
            None => 1,
        };
        let code = match code {
            27 => KeyCode::Esc,
            13 => KeyCode::Enter,
            9 => KeyCode::Tab,
            127 => KeyCode::Backspace,
            c => KeyCode::Char(char::from_u32(c)?),
        };
        Some(Key {
            code,
            mods: modifiers(param),
        })
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum KeyboardProtocol {
        #[default]
        Auto,

        On,

        Off,
    }

    impl KeyboardProtocol {
        pub const ENV: &'static str = "TERMROCK_KEYBOARD_PROTOCOL";

        pub const LEGACY_ENV: &'static str = "JUNIE_TUI_KEYBOARD_PROTOCOL";

        pub fn override_from_env(get: impl Fn(&str) -> Option<OsString>) -> Option<Self> {
            let value = get(Self::ENV).or_else(|| get(Self::LEGACY_ENV))?;
            Some(match value.into_string() {
                Ok(value) => Self::parse_env_value(&value),
                Err(_) => Self::Auto,
            })
        }

        pub fn from_env(get: impl Fn(&str) -> Option<String>) -> Self {
            get(Self::ENV)
                .or_else(|| get(Self::LEGACY_ENV))
                .as_deref()
                .map(Self::parse_env_value)
                .unwrap_or(Self::Auto)
        }

        fn parse_env_value(value: &str) -> Self {
            match value.trim().to_ascii_lowercase().as_str() {
                "on" | "1" | "true" | "kitty" => Self::On,
                "off" | "0" | "false" | "legacy" => Self::Off,
                _ => Self::Auto,
            }
        }

        pub fn current_override() -> Option<Self> {
            Self::override_from_env(|key| std::env::var_os(key))
        }

        pub fn current() -> Self {
            Self::current_override().unwrap_or(Self::Auto)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ProtocolState {
        pub enabled: bool,

        pub pushed: bool,
    }

    impl ProtocolState {
        pub fn enter(
            &mut self,
            policy: KeyboardProtocol,
            detect: impl FnOnce() -> bool,
        ) -> Option<String> {
            self.enabled = match policy {
                KeyboardProtocol::On => true,
                KeyboardProtocol::Off => false,
                KeyboardProtocol::Auto => detect(),
            };
            self.resume()
        }

        pub fn leave(&mut self) -> Option<&'static str> {
            std::mem::take(&mut self.pushed).then_some(POP)
        }

        pub fn resume(&mut self) -> Option<String> {
            (self.enabled && !self.pushed).then(|| {
                self.pushed = true;
                push(FLAGS)
            })
        }
    }

    static ENABLED: AtomicBool = AtomicBool::new(false);
    static PUSHED: AtomicBool = AtomicBool::new(false);

    static SUPER: AtomicU8 = AtomicU8::new(0);

    static ALT: AtomicU8 = AtomicU8::new(0);

    static ALT_KEYS: AtomicU8 = AtomicU8::new(0);

    pub mod session {
        use super::*;

        fn load() -> ProtocolState {
            ProtocolState {
                enabled: ENABLED.load(Ordering::SeqCst),
                pushed: PUSHED.load(Ordering::SeqCst),
            }
        }

        fn store(s: ProtocolState) {
            ENABLED.store(s.enabled, Ordering::SeqCst);
            PUSHED.store(s.pushed, Ordering::SeqCst);
        }

        pub fn enter(policy: KeyboardProtocol, detect: impl FnOnce() -> bool) -> Option<String> {
            let mut s = load();
            let out = s.enter(policy, detect);
            store(s);
            out
        }

        pub fn leave() -> Option<&'static str> {
            let mut s = load();
            let out = s.leave();
            store(s);
            out
        }

        pub fn resume() -> Option<String> {
            let mut s = load();
            let out = s.resume();
            store(s);
            out
        }
    }

    pub fn active() -> bool {
        PUSHED.load(Ordering::SeqCst)
    }

    pub fn super_delivered() -> bool {
        matches!(SUPER.load(Ordering::SeqCst), 1 | 3)
    }

    pub fn set_super_delivered(on: bool) {
        SUPER.store(if on { 1 } else { 2 }, Ordering::SeqCst);
    }

    pub fn observe(input: &Input) {
        let Input::Key(k) = input else { return };
        if k.mods.contains(KeyModifiers::SUPER) && SUPER.load(Ordering::SeqCst) != 2 {
            SUPER.store(3, Ordering::SeqCst);
        }
        if k.mods.contains(KeyModifiers::ALT)
            && (active()
                || !matches!(
                    k.code,
                    KeyCode::Char(_)
                        | KeyCode::Esc
                        | KeyCode::Enter
                        | KeyCode::Backspace
                        | KeyCode::Tab
                        | KeyCode::BackTab
                ))
            && ALT.load(Ordering::SeqCst) == 0
        {
            ALT.store(3, Ordering::SeqCst);
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AltKeys {
        Delivered,

        Refused,

        UntilSeen,

        WithProtocol,
    }

    impl AltKeys {
        pub const ENV: &'static str = "TERMROCK_ALT_KEYS";

        pub const OPTION_TYPES_TERM_PROGRAMS: [&'static str; 2] = ["Apple_Terminal", "iTerm.app"];

        pub const OPTION_IS_ALT_TERM_PROGRAMS: [&'static str; 1] = ["wezterm"];

        pub fn from_env(platform: ClientPlatform, get: impl Fn(&str) -> Option<String>) -> Self {
            match get(Self::ENV)
                .map(|v| v.trim().to_ascii_lowercase())
                .as_deref()
            {
                Some("on" | "1" | "true" | "yes") => return Self::Delivered,
                Some("off" | "0" | "false" | "no") => return Self::Refused,
                _ => {}
            }
            if !platform.is_macos() {
                return Self::Delivered;
            }
            let program = get("TERM_PROGRAM").unwrap_or_default();
            if get("LC_TERMINAL").is_some_and(|v| v == "iTerm2")
                || Self::OPTION_TYPES_TERM_PROGRAMS.contains(&program.as_str())
            {
                return Self::UntilSeen;
            }
            if Self::OPTION_IS_ALT_TERM_PROGRAMS.contains(&program.to_ascii_lowercase().as_str()) {
                return Self::Delivered;
            }
            Self::WithProtocol
        }

        pub fn current() -> Self {
            match ALT_KEYS.load(Ordering::SeqCst) {
                1 => Self::Delivered,
                2 => Self::Refused,
                3 => Self::UntilSeen,
                4 => Self::WithProtocol,
                _ => {
                    let d = Self::from_env(ClientPlatform::current(), |k| std::env::var(k).ok());
                    set_alt_keys(d);
                    d
                }
            }
        }
    }

    pub fn set_alt_keys(keys: AltKeys) {
        let v = match keys {
            AltKeys::Delivered => 1,
            AltKeys::Refused => 2,
            AltKeys::UntilSeen => 3,
            AltKeys::WithProtocol => 4,
        };
        ALT_KEYS.store(v, Ordering::SeqCst);
    }

    pub fn alt_delivered() -> bool {
        match ALT.load(Ordering::SeqCst) {
            1 => return true,
            2 => return false,
            _ => {}
        }
        match AltKeys::current() {
            AltKeys::Delivered => true,
            AltKeys::Refused => false,
            AltKeys::UntilSeen => ALT.load(Ordering::SeqCst) == 3,
            AltKeys::WithProtocol => active() || ALT.load(Ordering::SeqCst) == 3,
        }
    }

    pub fn label_deliverable(key: &str) -> bool {
        let cmd = key.starts_with('⌘') || key.starts_with("Cmd+") || key.starts_with("Super+");
        let pointer = |rest: &str| rest.starts_with("click") || rest.starts_with("drag");
        let alt = ["Alt+", "Ctrl+Alt+", "Option+", "⌥"]
            .iter()
            .find_map(|p| key.strip_prefix(p))
            .is_some_and(|rest| !pointer(rest));
        let shift_pointer = key.starts_with("Shift+click")
            || key.starts_with("Shift+drag")
            || key.starts_with("Shift-click")
            || key.starts_with("⇧click");
        !shift_pointer && (!cmd || super_delivered()) && (!alt || alt_delivered())
    }

    pub fn set_alt_delivered(on: bool) {
        ALT.store(if on { 1 } else { 2 }, Ordering::SeqCst);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediaKeyCode {
    Play,
    Pause,
    PlayPause,
    Reverse,
    Stop,
    FastForward,
    Rewind,
    TrackNext,
    TrackPrevious,
    Record,
    LowerVolume,
    RaiseVolume,
    MuteVolume,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModifierKeyCode {
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    LeftHyper,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    RightHyper,
    RightMeta,
    IsoLevel3Shift,
    IsoLevel5Shift,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    Null,
    Esc,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    KeypadBegin,
    F(u8),
    Char(char),
    Media(MediaKeyCode),
    Modifier(ModifierKeyCode),
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 1;
        const CONTROL = 2;
        const ALT = 4;
        const SUPER = 8;
        const HYPER = 16;
        const META = 32;
        const NONE = 0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum KeyPhase {
    Capture,
    Bubble,
}
