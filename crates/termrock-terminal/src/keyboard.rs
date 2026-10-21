//! Write-side kitty keyboard-protocol negotiation for a terminal session.
//!
//! Ported from PR #26 `src/core/keyboard.rs` session plumbing (commit
//! `f4867e9b5`). The sequences come from
//! [`termrock_core::keys::keyboard`]; this module only writes them.
//! Negotiation is opt-in: a session that never calls
//! [`KeyboardNegotiation::enter`] emits no protocol bytes, which keeps the
//! frozen apps byte-identical (the baseline captures contain no
//! negotiation, and the boot mode check pins only 1049/1003/1006/2004).

use std::io::{self, Write};

pub use termrock_core::keys::keyboard::{
    DISAMBIGUATE, FLAGS, KeyboardProtocol, POP, ProtocolState, QUERY, REPORT_ALTERNATE_KEYS,
};

/// A session's keyboard-protocol negotiation state.
///
/// `enter` pushes once, `leave` pops once (suspend), `resume` pushes again
/// (continue). Every call writes at most one sequence and flushes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyboardNegotiation {
    state: ProtocolState,
}

impl KeyboardNegotiation {
    /// Negotiate against `policy` and push if enabled. `detect` is the
    /// caller's capability probe (a `CSI ?u` query and reply parse for a
    /// real terminal). Returns whether a flag set was pushed.
    pub fn enter(
        &mut self,
        out: &mut impl Write,
        policy: KeyboardProtocol,
        detect: impl FnOnce() -> bool,
    ) -> io::Result<bool> {
        if let Some(sequence) = self.state.enter(policy, detect) {
            write_sequence(out, &sequence)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Suspend: pop what we pushed, once. Returns whether a pop was
    /// written.
    pub fn leave(&mut self, out: &mut impl Write) -> io::Result<bool> {
        match self.state.leave() {
            Some(pop) => {
                write_sequence(out, pop)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Continue after a suspend: push again, once. Returns whether a push
    /// was written.
    pub fn resume(&mut self, out: &mut impl Write) -> io::Result<bool> {
        match self.state.resume() {
            Some(sequence) => {
                write_sequence(out, &sequence)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Is a flag set currently pushed?
    pub const fn is_pushed(&self) -> bool {
        self.state.pushed
    }

    /// Did negotiation enable the protocol?
    pub const fn is_enabled(&self) -> bool {
        self.state.enabled
    }

    /// The underlying state, for callers that need to answer questions
    /// without writing.
    pub const fn state(&self) -> ProtocolState {
        self.state
    }
}

fn write_sequence(out: &mut impl Write, sequence: &str) -> io::Result<()> {
    out.write_all(sequence.as_bytes())?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer() -> Vec<u8> {
        Vec::new()
    }

    fn written(out: &[u8]) -> String {
        String::from_utf8(out.to_vec()).expect("plain CSI bytes")
    }

    #[test]
    fn enter_pushes_once_and_leave_pops_once() {
        let mut out = buffer();
        let mut n = KeyboardNegotiation::default();
        assert!(
            n.enter(&mut out, KeyboardProtocol::On, || panic!("no query"))
                .unwrap()
        );
        assert_eq!(written(&out), "\x1b[>5u");
        assert!(n.is_pushed() && n.is_enabled());
        assert!(
            !n.enter(&mut out, KeyboardProtocol::On, || panic!("no query"))
                .unwrap()
        );
        assert_eq!(
            written(&out),
            "\x1b[>5u",
            "the second enter pushes nothing more"
        );
        assert!(n.leave(&mut out).unwrap());
        assert_eq!(written(&out), "\x1b[>5u\x1b[<u");
        assert!(
            !n.leave(&mut out).unwrap(),
            "the exit after it pops nothing more"
        );
        assert_eq!(written(&out), "\x1b[>5u\x1b[<u");
        assert!(n.resume(&mut out).unwrap());
        assert_eq!(written(&out), "\x1b[>5u\x1b[<u\x1b[>5u");
        assert!(!n.resume(&mut out).unwrap(), "once");
        assert!(n.leave(&mut out).unwrap());
        assert_eq!(written(&out), "\x1b[>5u\x1b[<u\x1b[>5u\x1b[<u");
    }

    #[test]
    fn off_and_refused_auto_write_nothing() {
        let mut out = buffer();
        let mut n = KeyboardNegotiation::default();
        assert!(
            !n.enter(&mut out, KeyboardProtocol::Off, || panic!("no query"))
                .unwrap()
        );
        assert!(!n.enter(&mut out, KeyboardProtocol::Auto, || false).unwrap());
        assert!(out.is_empty(), "the legacy path emits no protocol bytes");
        assert!(!n.is_pushed());
        assert!(!n.leave(&mut out).unwrap());
        assert!(!n.resume(&mut out).unwrap());
        assert!(out.is_empty());
    }

    #[test]
    fn auto_pushes_when_the_terminal_answers() {
        let mut out = buffer();
        let mut n = KeyboardNegotiation::default();
        assert!(n.enter(&mut out, KeyboardProtocol::Auto, || true).unwrap());
        assert_eq!(written(&out), "\x1b[>5u");
    }

    #[test]
    fn sequences_come_from_the_core_module() {
        assert_eq!(FLAGS, 5);
        assert_eq!(QUERY, "\x1b[?u");
        assert_eq!(POP, "\x1b[<u");
    }
}
