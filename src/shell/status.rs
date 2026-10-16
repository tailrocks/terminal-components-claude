//! The status line: one transient message that reaches the user without
//! taking focus (WCAG 2.2 SC 4.1.3, Status Messages).
//!
//! - An information message expires [`StatusLine::DEFAULT_TTL_MS`] (5 s)
//!   after it is set, measured on the shell's [`Clock`](crate::clock::Clock)
//!   (so a paused capture never ages it out).
//! - An error ([`Tone::Error`]) never times out (SC 2.2.1, Timing
//!   Adjustable): it stays until the user's next action
//!   ([`StatusLine::on_input`]) or until another message replaces it.
//! - Priority: while an error is showing, a non-error message from the
//!   same action does not hide it.
//!
//! The application draws [`StatusLine::get`] in its footer; the line holds
//! state only.

use crate::clock::Moment;
use crate::core::event::{Input, MouseKind};
use crate::theme::Tone;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Message {
    text: String,
    tone: Tone,
    at: Moment,
}

/// One transient status message with its expiry rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLine {
    message: Option<Message>,
    ttl_ms: u64,
    /// The last time the shell's clock was read ([`StatusLine::observe`],
    /// [`StatusLine::expire`]); a message set now starts here.
    now: Moment,
}

impl Default for StatusLine {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusLine {
    /// How long an information message stays: 5 s.
    pub const DEFAULT_TTL_MS: u64 = 5_000;

    pub const fn new() -> Self {
        Self {
            message: None,
            ttl_ms: Self::DEFAULT_TTL_MS,
            now: Moment::ZERO,
        }
    }

    /// Information messages stay `ms` instead of
    /// [`DEFAULT_TTL_MS`](Self::DEFAULT_TTL_MS).
    pub const fn ttl_ms(mut self, ms: u64) -> Self {
        self.ttl_ms = ms;
        self
    }

    /// Record the clock's current reading; a message set afterwards starts
    /// its time here.
    pub fn observe(&mut self, now: Moment) {
        self.now = self.now.max(now);
    }

    /// Show `text` in `tone`. An error stays until the next action; any
    /// other tone expires after the TTL. A non-error message does not
    /// replace an error that is still showing.
    pub fn set(&mut self, text: impl Into<String>, tone: Tone) {
        if tone != Tone::Error && self.is_error() {
            return;
        }
        self.message = Some(Message {
            text: text.into(),
            tone,
            at: self.now,
        });
    }

    /// [`set`](Self::set) after [`observe`](Self::observe)`(now)`.
    pub fn set_at(&mut self, text: impl Into<String>, tone: Tone, now: Moment) {
        self.observe(now);
        self.set(text, tone);
    }

    /// Show an error ([`Tone::Error`]).
    pub fn error(&mut self, text: impl Into<String>) {
        self.set(text, Tone::Error);
    }

    /// Remove the message.
    pub fn clear(&mut self) -> bool {
        self.message.take().is_some()
    }

    /// The message and its tone, if one is showing.
    pub fn get(&self) -> Option<(&str, Tone)> {
        self.message.as_ref().map(|m| (m.text.as_str(), m.tone))
    }

    /// The message text, if one is showing.
    pub fn text(&self) -> Option<&str> {
        self.get().map(|(text, _)| text)
    }

    pub fn is_some(&self) -> bool {
        self.message.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.message.is_none()
    }

    /// Whether an error is showing.
    pub fn is_error(&self) -> bool {
        self.message.as_ref().is_some_and(|m| m.tone == Tone::Error)
    }

    /// Read the clock and drop an information message older than the TTL.
    /// Returns whether the line changed (a repaint is due).
    pub fn expire(&mut self, now: Moment) -> bool {
        self.observe(now);
        let due = self
            .message
            .as_ref()
            .is_some_and(|m| m.tone != Tone::Error && now.since(m.at) >= self.ttl_ms);
        if due {
            self.message = None;
        }
        due
    }

    /// The user acted: an error has been seen and goes. A key, a paste or a
    /// button press counts; pointer moves, wheel, ticks and resizes do not.
    /// Call before the input is handled, so a new message the action sets
    /// stays. Returns whether the line changed.
    pub fn on_input(&mut self, input: &Input) -> bool {
        let acts = match input {
            Input::Key(_) | Input::Paste(_) => true,
            Input::Mouse(m) => matches!(
                m.kind,
                MouseKind::Down | MouseKind::Secondary | MouseKind::Middle
            ),
            Input::Tick | Input::Resize(..) => false,
        };
        if acts && self.is_error() {
            self.message = None;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{Key, Mouse};
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    use ratatui::layout::Position;

    fn ms(n: u64) -> Moment {
        Moment::from_millis(n)
    }

    #[test]
    fn information_expires_after_five_seconds_on_the_clock() {
        let mut s = StatusLine::new();
        s.observe(ms(1_000));
        s.set("Saved", Tone::Secondary);
        assert!(!s.expire(ms(5_999)));
        assert_eq!(s.get(), Some(("Saved", Tone::Secondary)));
        assert!(s.expire(ms(6_000)));
        assert!(s.is_none());
    }

    #[test]
    fn a_clock_that_does_not_move_never_ages_a_message() {
        let mut s = StatusLine::new();
        s.set_at("Paused capture", Tone::Normal, ms(80));
        for _ in 0..100 {
            assert!(!s.expire(ms(80)));
        }
        assert_eq!(s.text(), Some("Paused capture"));
    }

    #[test]
    fn errors_persist_until_the_next_action() {
        let mut s = StatusLine::new();
        s.error("Connection refused");
        assert!(!s.expire(ms(60_000)));
        assert!(s.is_error());
        // moves, wheel, ticks and resizes are not actions
        assert!(!s.on_input(&Input::Tick));
        assert!(!s.on_input(&Input::Resize(80, 24)));
        assert!(!s.on_input(&Input::Mouse(Mouse::new(
            MouseKind::Move,
            Position::new(1, 1)
        ))));
        assert!(s.is_error());
        // an information message from the same action does not hide it
        s.set("Retrying", Tone::Secondary);
        assert_eq!(s.text(), Some("Connection refused"));
        // a key press is the next action
        let key = Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        };
        assert!(s.on_input(&Input::Key(key)));
        assert!(s.is_none());
        s.set("Retrying", Tone::Secondary);
        assert_eq!(s.text(), Some("Retrying"));
    }

    #[test]
    fn a_new_error_replaces_an_error_and_the_ttl_is_configurable() {
        let mut s = StatusLine::new().ttl_ms(1_000);
        s.error("first");
        s.error("second");
        assert_eq!(s.text(), Some("second"));
        assert!(s.clear());
        s.set_at("info", Tone::Normal, ms(10));
        assert!(s.expire(ms(1_010)));
    }
}
