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
//! knows its terminal says so ([`set_super_delivered`]).

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
/// proves Cmd chords are delivered.
pub fn observe(input: &Input) {
    if let Input::Key(k) = input
        && k.mods.contains(KeyModifiers::SUPER)
        && SUPER.load(Ordering::SeqCst) != 2
    {
        SUPER.store(3, Ordering::SeqCst);
    }
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
}
