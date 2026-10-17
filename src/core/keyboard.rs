//! The kitty keyboard protocol: progressive enhancement of key reporting.
//!
//! A terminal speaking the legacy encoding cannot report Cmd (the super
//! modifier) at all, so Cmd+A, the macOS Select All, never reaches an
//! application. Terminals that implement the kitty keyboard protocol
//! (<https://sw.kovidgoyal.net/kitty/keyboard-protocol/>) report it once the
//! application pushes the enhancement flags. The runtime
//! ([`crate::runtime::TerminalSession`]) does that on enter, pops them on
//! leave and on suspend, and pushes them again on continue, so the host
//! shell never inherits them.
//!
//! Support is detected with the protocol's own query (`CSI ? u`, answered
//! with `CSI ? flags u`) when [`KeyboardProtocol::Auto`] is in effect;
//! [`KeyboardProtocol::ENV`] overrides the decision. Decoding the reports
//! is crossterm's (the super modifier arrives as
//! [`KeyModifiers::SUPER`]); [`encode`] and [`decode`] are the reference
//! codec for PTY drivers and tests.
//!
//! Hints must not advertise a Cmd chord the terminal does not deliver
//! (AGENTS.md). Pushing the flags is not proof: a macOS terminal can still
//! keep Cmd+A for its own menu. [`super_delivered`] turns true only once a
//! super-modified key actually arrived ([`observe`]), or when an app that
//! knows its terminal says so ([`set_super_delivered`]). Alt (Option) is
//! the same question on a macOS client, whose terminals type a character
//! for Option by default: [`alt_delivered`] (A29).

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::core::event::{Input, Key};

/// `CSI > flags u` flag: report every modified key unambiguously (Esc,
/// Alt+letter and Ctrl+letter as `CSI … u`).
pub const DISAMBIGUATE: u8 = 0b1;
/// `CSI > flags u` flag: report the shifted and base-layout keys too.
pub const REPORT_ALTERNATE_KEYS: u8 = 0b100;
/// The flags the runtime pushes: disambiguate plus alternate keys. Key
/// release events (flag 2) and all-keys-as-escapes (flag 8) stay off, so
/// plain typing and Enter/Tab/Backspace keep their legacy bytes.
pub const FLAGS: u8 = DISAMBIGUATE | REPORT_ALTERNATE_KEYS;

/// The support query; a terminal that implements the protocol answers
/// `CSI ? flags u` (see [`parse_flags_reply`]).
pub const QUERY: &str = "\x1b[?u";
/// Pop the flags this application pushed (one entry of the stack).
pub const POP: &str = "\x1b[<u";

/// `CSI > flags u`: push `flags` onto the terminal's enhancement stack.
pub fn push(flags: u8) -> String {
    format!("\x1b[>{flags}u")
}

/// The flags in a query reply `CSI ? flags u`, or `None` when `reply` is
/// not one (a terminal without the protocol answers only the primary
/// device attributes query that follows it).
pub fn parse_flags_reply(reply: &[u8]) -> Option<u8> {
    let body = reply.strip_prefix(b"\x1b[?")?.strip_suffix(b"u")?;
    if body.is_empty() || !body.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(body).ok()?.parse().ok()
}

/// The protocol's modifier bits (the parameter minus one): shift 1, alt 2,
/// ctrl 4, super 8. Hyper, meta and the lock bits have no
/// [`KeyModifiers`] use here and are dropped.
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

/// The modifier parameter for `mods` (the inverse of [`modifiers`]).
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

/// `key` as the protocol reports it under [`FLAGS`]: `CSI code ; mods u`
/// for a character with a modifier (the code is the unshifted key), `None`
/// for keys this codec does not cover (named keys keep their legacy form).
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

/// Decode one `CSI code[:alternates] [; mods] u` key report into a [`Key`]
/// (the inverse of [`encode`]); `None` for anything else.
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

/// Whether the runtime enables the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyboardProtocol {
    /// Ask the terminal ([`QUERY`]) and enable the protocol when it
    /// answers. The default: progressive enhancement.
    #[default]
    Auto,
    /// Push the flags without asking (a terminal known to support it).
    On,
    /// Never push the flags: legacy key reporting only.
    Off,
}

impl KeyboardProtocol {
    /// The environment variable that overrides the default: `auto`, `on`
    /// (or `1`, `true`, `kitty`) and `off` (or `0`, `false`, `legacy`).
    pub const ENV: &'static str = "JUNIE_TUI_KEYBOARD_PROTOCOL";

    /// The policy named by [`Self::ENV`] in `get`, else `Auto`.
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

    /// [`Self::from_env`] on the process environment.
    pub fn current() -> Self {
        Self::from_env(|k| std::env::var(k).ok())
    }
}

/// What the terminal session pushes and pops, as a pure state machine so
/// the enter / leave / suspend / continue sequence is testable without a
/// terminal. The runtime keeps one per process ([`session`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProtocolState {
    /// The protocol was chosen for this session (on enter).
    pub enabled: bool,
    /// The flags are on the terminal's stack now.
    pub pushed: bool,
}

impl ProtocolState {
    /// Decide on enter: `policy`, with `detect` asked only under
    /// [`KeyboardProtocol::Auto`]. Returns the bytes to write, if any.
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

    /// Leave (exit, panic, suspend): pop once, whatever path calls it.
    pub fn leave(&mut self) -> Option<&'static str> {
        std::mem::take(&mut self.pushed).then_some(POP)
    }

    /// Continue after a suspension: push again when the session uses it.
    pub fn resume(&mut self) -> Option<String> {
        (self.enabled && !self.pushed).then(|| {
            self.pushed = true;
            push(FLAGS)
        })
    }
}

static ENABLED: AtomicBool = AtomicBool::new(false);
static PUSHED: AtomicBool = AtomicBool::new(false);
/// 0 unknown, 1 confirmed by an app, 2 refused by an app, 3 observed.
static SUPER: AtomicU8 = AtomicU8::new(0);
/// The same states as `SUPER`, for Alt.
static ALT: AtomicU8 = AtomicU8::new(0);
/// The installed [`AltKeys`] (0: not detected yet).
static ALT_KEYS: AtomicU8 = AtomicU8::new(0);

/// The process's protocol state (the terminal session is one per
/// process; the panic hook restores through it too).
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

    /// [`ProtocolState::enter`] on the process state.
    pub fn enter(policy: KeyboardProtocol, detect: impl FnOnce() -> bool) -> Option<String> {
        let mut s = load();
        let out = s.enter(policy, detect);
        store(s);
        out
    }

    /// [`ProtocolState::leave`] on the process state.
    pub fn leave() -> Option<&'static str> {
        let mut s = load();
        let out = s.leave();
        store(s);
        out
    }

    /// [`ProtocolState::resume`] on the process state.
    pub fn resume() -> Option<String> {
        let mut s = load();
        let out = s.resume();
        store(s);
        out
    }
}

/// True while the enhancement flags are pushed (the terminal reports keys
/// in the protocol's encoding).
pub fn active() -> bool {
    PUSHED.load(Ordering::SeqCst)
}

/// True when Cmd (super) chords are known to reach the application: a
/// super-modified key arrived ([`observe`]) or the app said so
/// ([`set_super_delivered`]). Hints name a Cmd chord only then.
pub fn super_delivered() -> bool {
    matches!(SUPER.load(Ordering::SeqCst), 1 | 3)
}

/// An app that knows its terminal delivers Cmd (or does not) says so;
/// `false` also forgets an observation.
pub fn set_super_delivered(on: bool) {
    SUPER.store(if on { 1 } else { 2 }, Ordering::SeqCst);
}

/// Note an input the runtime decoded: a key carrying the super modifier
/// proves Cmd chords are delivered; an Alt-modified key proves Alt chords
/// are ([`alt_delivered`]) when it cannot be a stray Esc prefix: under
/// the protocol any key, under the legacy encoding only a named key
/// (an arrow, Home, a function key), whose modifier rides in its own
/// escape sequence. A legacy Alt+letter is `Esc` then the letter, which a
/// quick Esc, letter also produces, and macOS Terminal sends Option+←
/// as Esc b, so neither proves Option reaches the app as Alt.
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

/// Whether the terminal delivers Alt (Option) chords, as detected from the
/// environment ([`AltKeys::from_env`]). macOS Terminal and iTerm2 type a
/// character for Option+letter (Option+Z is `Ω`) and send Option+← as a
/// word move unless "Use Option as Meta key" (Terminal) or "Option key
/// acts as Esc+" (iTerm2) is on, so on a macOS client an Alt chord is
/// not advertised until it is known to arrive (AGENTS.md: hints never
/// advertise keys the terminal cannot deliver).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltKeys {
    /// Alt chords arrive: a Linux, Windows or BSD client, a macOS
    /// terminal that sends Option as Alt by default (WezTerm), or
    /// [`AltKeys::ENV`] `on`.
    Delivered,
    /// Alt chords do not arrive: [`AltKeys::ENV`] `off`.
    Refused,
    /// A macOS terminal that keeps Option for typing by default (macOS
    /// Terminal, iTerm2): delivered only once an Alt key arrives
    /// ([`observe`]).
    UntilSeen,
    /// Another terminal on a macOS client (Ghostty, kitty): delivered
    /// while the kitty keyboard protocol is on ([`active`]), whose
    /// terminals report the Alt modifier unambiguously, or once an Alt
    /// key arrives. Ghostty and kitty keep Option for composing unless
    /// configured (`macos-option-as-alt`, `macos_option_as_alt`); set
    /// [`AltKeys::ENV`] `off` where neither is.
    WithProtocol,
}

impl AltKeys {
    /// The environment variable that overrides detection: `on` (or `1`,
    /// `true`, `yes`) and `off` (or `0`, `false`, `no`), any case. Set it
    /// to `on` after turning on "Use Option as Meta key" in macOS
    /// Terminal.
    pub const ENV: &'static str = "JUNIE_TUI_ALT_KEYS";

    /// `TERM_PROGRAM` values of macOS terminals that type a character for
    /// Option unless configured otherwise.
    pub const OPTION_TYPES_TERM_PROGRAMS: [&'static str; 2] = ["Apple_Terminal", "iTerm.app"];

    /// `TERM_PROGRAM` values (any case) of terminals that send Option as
    /// Alt on macOS by default.
    pub const OPTION_IS_ALT_TERM_PROGRAMS: [&'static str; 1] = ["wezterm"];

    /// Detect from environment lookups `get` for a client on `platform`,
    /// in order: [`Self::ENV`]; a client that is not macOS delivers;
    /// `LC_TERMINAL` `iTerm2` (forwarded over SSH) or a `TERM_PROGRAM` in
    /// [`Self::OPTION_TYPES_TERM_PROGRAMS`] waits to see an Alt key; a
    /// `TERM_PROGRAM` in [`Self::OPTION_IS_ALT_TERM_PROGRAMS`] delivers;
    /// any other macOS terminal delivers with the kitty keyboard
    /// protocol.
    pub fn from_env(
        platform: crate::core::event::ClientPlatform,
        get: impl Fn(&str) -> Option<String>,
    ) -> Self {
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

    /// [`Self::from_env`] on the process environment and
    /// [`crate::core::event::ClientPlatform::current`], or what
    /// [`set_alt_keys`] installed.
    pub fn current() -> Self {
        match ALT_KEYS.load(Ordering::SeqCst) {
            1 => Self::Delivered,
            2 => Self::Refused,
            3 => Self::UntilSeen,
            4 => Self::WithProtocol,
            _ => {
                let d = Self::from_env(crate::core::event::ClientPlatform::current(), |k| {
                    std::env::var(k).ok()
                });
                set_alt_keys(d);
                d
            }
        }
    }
}

/// Install the Alt detection (an app or a test that knows its terminal;
/// [`AltKeys::current`] detects from the environment otherwise).
pub fn set_alt_keys(keys: AltKeys) {
    let v = match keys {
        AltKeys::Delivered => 1,
        AltKeys::Refused => 2,
        AltKeys::UntilSeen => 3,
        AltKeys::WithProtocol => 4,
    };
    ALT_KEYS.store(v, Ordering::SeqCst);
}

/// True when Alt (Option) chords are known to reach the application: an
/// app said so ([`set_alt_delivered`]), else the detection
/// ([`AltKeys::current`]) says so, else an Alt key arrived ([`observe`]).
/// Hints name an Alt chord only then; a key map's stand-in for an Alt
/// chord ([`crate::core::keymap::KeyBinding::alt_stand_in`]) is active
/// only while this is false.
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

/// Whether a key label (a hint's key, a help row, a menu shortcut) names
/// keys the terminal delivers: a Cmd chord only once Cmd is confirmed
/// ([`super_delivered`]); an Alt (Option) key chord only while
/// [`alt_delivered`] (A29; an Alt or Option click is a pointer modifier,
/// not a key chord); never Shift+click or Shift+drag, which terminals
/// keep for their own text selection while the app captures the mouse.
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

/// An app that knows its terminal delivers Alt (or does not) says so;
/// `false` also forgets an observation.
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
        // alternate keys (flag 4) ride after a colon; the base code decides
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
    fn the_runtime_decodes_super_from_crossterm() {
        use ratatui::crossterm::event::{Event, KeyEvent};
        let ev = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::SUPER));
        let Some(input @ Input::Key(k)) = Input::from_crossterm(ev) else {
            panic!("a key");
        };
        assert!(k.cmd() && !k.ctrl() && !k.plain());
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
        // a terminal without the protocol: nothing is written either way
        let mut legacy = ProtocolState::default();
        assert_eq!(legacy.enter(KeyboardProtocol::Auto, || false), None);
        assert_eq!(legacy.leave(), None);
        assert_eq!(legacy.resume(), None);
        // the overrides never ask the terminal
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
        use crate::core::event::ClientPlatform::{MacOs, Other};
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
        // the override wins either way
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
        // a legacy Alt+letter is Esc then the letter: no proof
        observe(&Input::Key(key(KeyCode::Char('b'), KeyModifiers::ALT)));
        assert!(!alt_delivered());
        observe(&Input::Key(key(KeyCode::Enter, KeyModifiers::ALT)));
        assert!(!alt_delivered());
        // a modified arrow carries Alt in its own sequence
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
        // under the protocol even an Alt+letter is unambiguous
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
