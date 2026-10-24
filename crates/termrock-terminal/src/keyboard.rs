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
        let mut next = self.state;
        if let Some(sequence) = next.enter(policy, detect) {
            write_sequence(out, &sequence, || self.state = next)?;
            return Ok(true);
        }
        self.state = next;
        Ok(false)
    }

    /// Suspend: pop what we pushed, once. Returns whether a pop was
    /// written.
    pub fn leave(&mut self, out: &mut impl Write) -> io::Result<bool> {
        let mut next = self.state;
        match next.leave() {
            Some(pop) => {
                write_sequence(out, pop, || self.state = next)?;
                Ok(true)
            }
            None => {
                self.state = next;
                Ok(false)
            }
        }
    }

    /// Continue after a suspend: push again, once. Returns whether a push
    /// was written.
    pub fn resume(&mut self, out: &mut impl Write) -> io::Result<bool> {
        let mut next = self.state;
        match next.resume() {
            Some(sequence) => {
                write_sequence(out, &sequence, || self.state = next)?;
                Ok(true)
            }
            None => {
                self.state = next;
                Ok(false)
            }
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

fn write_sequence(
    out: &mut impl Write,
    sequence: &str,
    commit_after_write: impl FnOnce(),
) -> io::Result<()> {
    out.write_all(sequence.as_bytes())?;
    commit_after_write();
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

    #[derive(Debug, PartialEq, Eq)]
    struct ScriptedError(&'static str);

    impl std::fmt::Display for ScriptedError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str(self.0)
        }
    }

    impl std::error::Error for ScriptedError {}

    #[derive(Default)]
    struct ScriptedOutput {
        bytes: Vec<u8>,
        fail_after: Option<usize>,
        fail_flush_at: Option<usize>,
        flushes: usize,
    }

    impl ScriptedOutput {
        fn fail_after(accepted: usize) -> Self {
            Self {
                fail_after: Some(accepted),
                ..Self::default()
            }
        }

        fn fail_flush_at(flush: usize) -> Self {
            Self {
                fail_flush_at: Some(flush),
                ..Self::default()
            }
        }
    }

    impl Write for ScriptedOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if let Some(limit) = self.fail_after {
                if self.bytes.len() >= limit {
                    return Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        ScriptedError("scripted write failure"),
                    ));
                }
                let accepted = bytes.len().min(limit - self.bytes.len());
                self.bytes.extend_from_slice(&bytes[..accepted]);
                return Ok(accepted);
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            if self.fail_flush_at == Some(self.flushes) {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    ScriptedError("scripted flush failure"),
                ));
            }
            Ok(())
        }
    }

    fn assert_scripted_error(error: &io::Error, marker: &'static str) {
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(
            error
                .get_ref()
                .and_then(|source| source.downcast_ref::<ScriptedError>()),
            Some(&ScriptedError(marker))
        );
    }

    #[test]
    fn incomplete_push_pop_and_resume_keep_the_prior_protocol_state() {
        let push = termrock_core::keys::keyboard::push(FLAGS);
        for accepted in 0..push.len() {
            let mut negotiation = KeyboardNegotiation::default();
            let mut out = ScriptedOutput::fail_after(accepted);
            let error = negotiation
                .enter(&mut out, KeyboardProtocol::On, || false)
                .unwrap_err();
            assert_scripted_error(&error, "scripted write failure");
            assert_eq!(out.bytes.as_slice(), &push.as_bytes()[..accepted]);
            assert_eq!(out.flushes, 0);
            assert_eq!(negotiation.state(), ProtocolState::default());
        }

        let mut pushed = KeyboardNegotiation::default();
        let mut initial = Vec::new();
        assert!(
            pushed
                .enter(&mut initial, KeyboardProtocol::On, || false)
                .unwrap()
        );
        for accepted in 0..POP.len() {
            let mut negotiation = pushed;
            let mut out = ScriptedOutput::fail_after(accepted);
            let error = negotiation.leave(&mut out).unwrap_err();
            assert_scripted_error(&error, "scripted write failure");
            assert_eq!(out.bytes.as_slice(), &POP.as_bytes()[..accepted]);
            assert_eq!(out.flushes, 0);
            assert_eq!(negotiation.state(), pushed.state());
        }

        let mut suspended = pushed;
        assert!(suspended.leave(&mut initial).unwrap());
        let suspended_state = suspended.state();
        for accepted in 0..push.len() {
            let mut negotiation = suspended;
            let mut out = ScriptedOutput::fail_after(accepted);
            let error = negotiation.resume(&mut out).unwrap_err();
            assert_scripted_error(&error, "scripted write failure");
            assert_eq!(out.bytes.as_slice(), &push.as_bytes()[..accepted]);
            assert_eq!(out.flushes, 0);
            assert_eq!(negotiation.state(), suspended_state);
        }
    }

    #[test]
    fn complete_write_commits_before_flush_error_and_prevents_duplicate_sequence() {
        let push = termrock_core::keys::keyboard::push(FLAGS);
        let mut negotiation = KeyboardNegotiation::default();
        let mut out = ScriptedOutput::fail_flush_at(1);
        let error = negotiation
            .enter(&mut out, KeyboardProtocol::On, || false)
            .unwrap_err();
        assert_scripted_error(&error, "scripted flush failure");
        assert_eq!(
            negotiation.state(),
            ProtocolState {
                enabled: true,
                pushed: true
            }
        );
        assert_eq!(out.bytes.as_slice(), push.as_bytes());
        assert!(
            !negotiation
                .enter(&mut out, KeyboardProtocol::On, || false)
                .unwrap()
        );
        assert!(negotiation.leave(&mut out).unwrap());
        assert_eq!(out.bytes, [push.as_bytes(), POP.as_bytes()].concat());
        assert_eq!(
            negotiation.state(),
            ProtocolState {
                enabled: true,
                pushed: false
            }
        );

        let mut negotiation = KeyboardNegotiation::default();
        let mut setup = Vec::new();
        assert!(
            negotiation
                .enter(&mut setup, KeyboardProtocol::On, || false)
                .unwrap()
        );
        let mut out = ScriptedOutput::fail_flush_at(1);
        let error = negotiation.leave(&mut out).unwrap_err();
        assert_scripted_error(&error, "scripted flush failure");
        assert_eq!(
            negotiation.state(),
            ProtocolState {
                enabled: true,
                pushed: false
            }
        );
        assert!(!negotiation.leave(&mut out).unwrap());
        assert!(negotiation.resume(&mut out).unwrap());
        assert!(!negotiation.resume(&mut out).unwrap());
        let push = termrock_core::keys::keyboard::push(FLAGS);
        assert_eq!(out.bytes, [POP.as_bytes(), push.as_bytes()].concat());
    }
}
