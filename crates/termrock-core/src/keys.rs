//! Kitty keyboard protocol: capability query, push/pop sequences, CSI-u
//! encode/decode, and the delivery answers behind "hints never advertise
//! keys the terminal cannot deliver".
//!
//! Ported from PR #26 `src/core/keyboard.rs` (commit `f4867e9b5`). The
//! frozen apps never negotiate the protocol at startup (the baseline
//! captures contain no `CSI >` push and the boot mode check pins only
//! 1049/1003/1006/2004), so negotiation is opt-in: [`keyboard::KeyboardProtocol`]
//! defaults to `Auto` detection but the session crate only pushes when an
//! app asks for it.

pub mod keyboard {
    use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

    use crate::event::{Input, Key};
    use crate::keys::{KeyCode, KeyModifiers};
    use crate::platform::ClientPlatform;

    /// Disambiguate escape codes (kitty keyboard protocol flag 1).
    pub const DISAMBIGUATE: u8 = 0b1;
    /// Report alternate keys (kitty keyboard protocol flag 4).
    pub const REPORT_ALTERNATE_KEYS: u8 = 0b100;

    /// The flag set this library pushes: disambiguate + alternate keys.
    pub const FLAGS: u8 = DISAMBIGUATE | REPORT_ALTERNATE_KEYS;

    /// Flags query: the terminal replies `CSI ? <flags> u`.
    pub const QUERY: &str = "\x1b[?u";
    /// Pop the most recently pushed flag set.
    pub const POP: &str = "\x1b[<u";

    /// Push a flag set (`CSI > <flags> u`).
    pub fn push(flags: u8) -> String {
        format!("\x1b[>{flags}u")
    }

    /// Parse a flags reply (`CSI ? <flags> u`), rejecting other CSI replies.
    pub fn parse_flags_reply(reply: &[u8]) -> Option<u8> {
        let body = reply.strip_prefix(b"\x1b[?")?.strip_suffix(b"u")?;
        if body.is_empty() || !body.iter().all(u8::is_ascii_digit) {
            return None;
        }
        std::str::from_utf8(body).ok()?.parse().ok()
    }

    /// Decode a CSI-u modifier parameter (`mods + 1`) into `KeyModifiers`.
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

    /// Encode `KeyModifiers` into a CSI-u modifier parameter (`mods + 1`).
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

    /// CSI-u encode a plain or modified character key. Non-character keys have
    /// no legacy CSI-u text encoding, so they return `None`.
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

    /// Decode a CSI-u sequence (including the alternate-key `code:shifted`
    /// form) into a [`Key`]. Queries, pushes, pops and flags replies are not
    /// keys.
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

    /// Negotiation policy for the kitty keyboard protocol.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum KeyboardProtocol {
        /// Ask the terminal (via the caller's detect probe) before pushing.
        #[default]
        Auto,
        /// Always push, without asking.
        On,
        /// Never push.
        Off,
    }

    impl KeyboardProtocol {
        /// Environment override for the policy.
        pub const ENV: &'static str = "TERMROCK_KEYBOARD_PROTOCOL";

        /// Read the policy from an environment lookup.
        pub fn from_env(get: impl Fn(&str) -> Option<String>) -> Self {
            match get(Self::ENV)
                .map(|v| v.trim().to_ascii_lowercase())
                .as_deref()
            {
                Some("on" | "1" | "true" | "kitty") => Self::On,
                Some("off" | "0" | "false" | "legacy") => Self::Off,
                _ => Self::Auto,
            }
        }

        /// Read the policy from the real environment.
        pub fn current() -> Self {
            Self::from_env(|k| std::env::var(k).ok())
        }
    }

    /// Whether the protocol is enabled and currently pushed.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ProtocolState {
        /// The negotiated answer.
        pub enabled: bool,
        /// Whether we pushed a flag set that still needs its pop.
        pub pushed: bool,
    }

    impl ProtocolState {
        /// Negotiate: resolve `Auto` through `detect`, then push once.
        /// Returns the push sequence to write, if any.
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

        /// Suspend: pop what we pushed, once.
        pub fn leave(&mut self) -> Option<&'static str> {
            std::mem::take(&mut self.pushed).then_some(POP)
        }

        /// Continue after a suspend: push again, once.
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

    /// Process-global protocol state for the session lifetime.
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

        /// Negotiate for the whole process (see [`ProtocolState::enter`]).
        pub fn enter(policy: KeyboardProtocol, detect: impl FnOnce() -> bool) -> Option<String> {
            let mut s = load();
            let out = s.enter(policy, detect);
            store(s);
            out
        }

        /// Suspend for the whole process (see [`ProtocolState::leave`]).
        pub fn leave() -> Option<&'static str> {
            let mut s = load();
            let out = s.leave();
            store(s);
            out
        }

        /// Continue for the whole process (see [`ProtocolState::resume`]).
        pub fn resume() -> Option<String> {
            let mut s = load();
            let out = s.resume();
            store(s);
            out
        }
    }

    /// Is a flag set currently pushed (process-global)?
    pub fn active() -> bool {
        PUSHED.load(Ordering::SeqCst)
    }

    /// Does the terminal deliver the Super (Cmd) modifier?
    pub fn super_delivered() -> bool {
        matches!(SUPER.load(Ordering::SeqCst), 1 | 3)
    }

    /// Record the terminal's answer for Super from a probe or capability reply.
    pub fn set_super_delivered(on: bool) {
        SUPER.store(if on { 1 } else { 2 }, Ordering::SeqCst);
    }

    /// Learn from real input: a delivered Super chord, or an unambiguous Alt
    /// chord while the protocol is active, is proof of delivery.
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

    /// How Alt (Option) chords reach us, decided per platform and terminal.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AltKeys {
        /// Alt chords arrive as ALT-modified keys.
        Delivered,
        /// The user or terminal refuses them; apps may install stand-ins.
        Refused,
        /// macOS terminals that send Option as a dead key until we see one
        /// unambiguous Alt chord arrive.
        UntilSeen,
        /// Alt arrives only while the kitty protocol is active.
        WithProtocol,
    }

    impl AltKeys {
        /// Environment override for the answer.
        pub const ENV: &'static str = "TERMROCK_ALT_KEYS";

        /// Terminals whose Option key sends a composed character instead of an
        /// Alt chord (they decide per key, not per terminal).
        pub const OPTION_TYPES_TERM_PROGRAMS: [&'static str; 2] = ["Apple_Terminal", "iTerm.app"];

        /// Terminals that deliver Alt chords on macOS without the protocol.
        pub const OPTION_IS_ALT_TERM_PROGRAMS: [&'static str; 1] = ["wezterm"];

        /// Decide from an environment lookup under `platform`.
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

        /// The cached process-global answer, decided from the real environment
        /// on first use.
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

    /// Pin the process-global Alt answer (tests and terminals that answered a
    /// probe).
    pub fn set_alt_keys(keys: AltKeys) {
        let v = match keys {
            AltKeys::Delivered => 1,
            AltKeys::Refused => 2,
            AltKeys::UntilSeen => 3,
            AltKeys::WithProtocol => 4,
        };
        ALT_KEYS.store(v, Ordering::SeqCst);
    }

    /// Does the terminal deliver Alt chords right now?
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

    /// May a hint with this label be advertised? A Shift+click or Shift+drag is
    /// never keyboard-deliverable; a Cmd or Alt chord only when the terminal
    /// delivers that modifier.
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

    /// Record the terminal's answer for Alt from a probe.
    pub fn set_alt_delivered(on: bool) {
        ALT.store(if on { 1 } else { 2 }, Ordering::SeqCst);
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn key(code: KeyCode, mods: KeyModifiers) -> Key {
            Key { code, mods }
        }

        #[test]
        fn push_pop_and_query_sequences() {
            assert_eq!(FLAGS, 5, "disambiguate (1) + alternate keys (4)");
            assert_eq!(push(FLAGS), "\x1b[>5u");
            assert_eq!(POP, "\x1b[<u");
            assert_eq!(QUERY, "\x1b[?u");
            assert_eq!(parse_flags_reply(b"\x1b[?0u"), Some(0));
            assert_eq!(parse_flags_reply(b"\x1b[?15u"), Some(15));
            assert_eq!(parse_flags_reply(b"\x1b[?u"), None);
            assert_eq!(parse_flags_reply(b"\x1b[?62;22c"), None, "a DA reply");
        }

        #[test]
        fn super_a_encodes_and_decodes() {
            let cmd_a = key(KeyCode::Char('a'), KeyModifiers::SUPER);
            assert_eq!(encode(&cmd_a).as_deref(), Some("\x1b[97;9u"));
            assert_eq!(decode(b"\x1b[97;9u"), Some(cmd_a));
            let ctrl_a = key(KeyCode::Char('a'), KeyModifiers::CONTROL);
            assert_eq!(encode(&ctrl_a).as_deref(), Some("\x1b[97;5u"));
            assert_eq!(decode(b"\x1b[97;5u"), Some(ctrl_a));

            let shift_cmd = key(
                KeyCode::Char('a'),
                KeyModifiers::SUPER | KeyModifiers::SHIFT,
            );
            assert_eq!(decode(b"\x1b[97:65;10u"), Some(shift_cmd));
            assert_eq!(
                decode(b"\x1b[27u"),
                Some(key(KeyCode::Esc, KeyModifiers::NONE))
            );
            assert_eq!(decode(b"\x1b[?5u"), None, "a flags reply is no key");
            for p in 1..=16 {
                assert_eq!(modifier_param(modifiers(p)), p);
            }
        }

        #[test]
        #[cfg(feature = "crossterm")]
        fn the_runtime_decodes_super_from_crossterm() {
            use crate::event::Input;
            use ratatui_crossterm::crossterm::event::{Event, KeyEvent};
            let ev = Event::Key(KeyEvent::new(
                ratatui_crossterm::crossterm::event::KeyCode::Char('a'),
                ratatui_crossterm::crossterm::event::KeyModifiers::SUPER,
            ));
            let Some(input @ Input::Key(k)) = Input::from_crossterm(ev) else {
                panic!("a key");
            };
            assert!(k.mods.contains(KeyModifiers::SUPER));
            assert!(!k.mods.contains(KeyModifiers::CONTROL));
            assert!(!super_delivered(), "unknown until a Cmd key arrives");
            observe(&input);
            assert!(super_delivered());
            set_super_delivered(false);
            assert!(!super_delivered());
            observe(&input);
            assert!(!super_delivered(), "an app's refusal stands");
        }

        #[test]
        fn enter_leave_suspend_and_continue() {
            let mut s = ProtocolState::default();
            assert_eq!(
                s.enter(KeyboardProtocol::Auto, || true).as_deref(),
                Some("\x1b[>5u")
            );
            assert_eq!(s.leave(), Some(POP), "suspend pops");
            assert_eq!(s.leave(), None, "the exit after it pops nothing more");
            assert_eq!(s.resume().as_deref(), Some("\x1b[>5u"), "continue pushes");
            assert_eq!(s.resume(), None, "once");
            assert_eq!(s.leave(), Some(POP));

            let mut legacy = ProtocolState::default();
            assert_eq!(legacy.enter(KeyboardProtocol::Auto, || false), None);
            assert_eq!(legacy.leave(), None);
            assert_eq!(legacy.resume(), None);

            let mut on = ProtocolState::default();
            assert!(
                on.enter(KeyboardProtocol::On, || panic!("no query"))
                    .is_some()
            );
            let mut off = ProtocolState::default();
            assert!(
                off.enter(KeyboardProtocol::Off, || panic!("no query"))
                    .is_none()
            );
        }

        #[test]
        fn the_env_override() {
            let env =
                |v: &'static str| move |k: &str| (k == KeyboardProtocol::ENV).then(|| v.to_owned());
            assert_eq!(
                KeyboardProtocol::from_env(env("off")),
                KeyboardProtocol::Off
            );
            assert_eq!(
                KeyboardProtocol::from_env(env("Kitty")),
                KeyboardProtocol::On
            );
            assert_eq!(
                KeyboardProtocol::from_env(env("maybe")),
                KeyboardProtocol::Auto
            );
            assert_eq!(KeyboardProtocol::from_env(|_| None), KeyboardProtocol::Auto);
        }

        fn env<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
            move |k| {
                vars.iter()
                    .find(|(n, _)| *n == k)
                    .map(|(_, v)| (*v).to_owned())
            }
        }

        #[test]
        fn alt_detection_per_platform_and_terminal() {
            use ClientPlatform::{MacOs, Other};
            assert_eq!(AltKeys::from_env(Other, env(&[])), AltKeys::Delivered);
            assert_eq!(
                AltKeys::from_env(Other, env(&[("TERM_PROGRAM", "Apple_Terminal")])),
                AltKeys::Delivered,
                "the client decides, not the terminal's name"
            );
            let apple = [("TERM_PROGRAM", "Apple_Terminal")];
            assert_eq!(AltKeys::from_env(MacOs, env(&apple)), AltKeys::UntilSeen);
            assert_eq!(
                AltKeys::from_env(MacOs, env(&[("TERM_PROGRAM", "iTerm.app")])),
                AltKeys::UntilSeen
            );
            assert_eq!(
                AltKeys::from_env(MacOs, env(&[("LC_TERMINAL", "iTerm2")])),
                AltKeys::UntilSeen,
                "iTerm2 over SSH"
            );
            assert_eq!(
                AltKeys::from_env(MacOs, env(&[("TERM_PROGRAM", "WezTerm")])),
                AltKeys::Delivered
            );
            for vars in [
                &[("TERM_PROGRAM", "ghostty"), ("TERM", "xterm-ghostty")][..],
                &[("TERM", "xterm-kitty")],
                &[("TERM", "xterm-256color")],
            ] {
                assert_eq!(AltKeys::from_env(MacOs, env(vars)), AltKeys::WithProtocol);
            }

            let on = [("TERM_PROGRAM", "Apple_Terminal"), (AltKeys::ENV, "ON")];
            assert_eq!(AltKeys::from_env(MacOs, env(&on)), AltKeys::Delivered);
            assert_eq!(
                AltKeys::from_env(Other, env(&[(AltKeys::ENV, "off")])),
                AltKeys::Refused
            );
            assert_eq!(
                AltKeys::from_env(MacOs, env(&[(AltKeys::ENV, "maybe")])),
                AltKeys::WithProtocol,
                "an unknown value is ignored"
            );
        }

        #[test]
        fn macos_terminal_delivers_alt_only_once_an_unambiguous_alt_key_arrives() {
            set_alt_keys(AltKeys::UntilSeen);
            assert!(!alt_delivered());
            assert!(!label_deliverable("Alt+Z"));
            assert!(label_deliverable("Option+click"), "a pointer modifier");

            observe(&Input::Key(key(KeyCode::Char('b'), KeyModifiers::ALT)));
            assert!(!alt_delivered());
            observe(&Input::Key(key(KeyCode::Enter, KeyModifiers::ALT)));
            assert!(!alt_delivered());

            observe(&Input::Key(key(KeyCode::Left, KeyModifiers::ALT)));
            assert!(alt_delivered());
            assert!(label_deliverable("Alt+Shift+←→"));
            set_alt_delivered(false);
            assert!(!alt_delivered(), "an app's refusal stands");
        }

        #[test]
        fn another_macos_terminal_delivers_alt_with_the_kitty_protocol() {
            set_alt_keys(AltKeys::WithProtocol);
            assert!(!alt_delivered());
            session::enter(KeyboardProtocol::On, || false);
            assert!(alt_delivered());
            session::leave();
            assert!(!alt_delivered());

            session::enter(KeyboardProtocol::On, || false);
            observe(&Input::Key(key(KeyCode::Char('z'), KeyModifiers::ALT)));
            session::leave();
            assert!(alt_delivered(), "seen once, delivered for the session");
        }

        #[test]
        fn alt_refused_or_delivered_by_detection() {
            set_alt_keys(AltKeys::Refused);
            observe(&Input::Key(key(KeyCode::Left, KeyModifiers::ALT)));
            assert!(!alt_delivered(), "the user's off stands");
            set_alt_keys(AltKeys::Delivered);
            assert!(alt_delivered());
            assert!(label_deliverable("Alt+Enter"));
        }
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
