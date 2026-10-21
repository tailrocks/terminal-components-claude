//! Status line ownership: one message, its tone, its time to live and the
//! job that owns it.
//!
//! Ported from PR #26 `src/shell/status.rs` (commit `f4867e9b5`). The tone is
//! the shared [`Role`] vocabulary instead of the source's `Tone` enum, so the
//! status message renders through the same theme roles as every component:
//! `Tone::Error` lands on [`Role::Danger`], `Tone::Success` on
//! [`Role::Success`], `Tone::Normal` on `Role::Fg(FgStep::Primary)` and
//! `Tone::Secondary` on `Role::Fg(FgStep::Secondary)`.

use termrock_core::event::{Input, MouseKind};
use termrock_runtime::Moment;
use termrock_theme::{FgStep, Role};

/// The tone of a status message, expressed in the shared theme roles.
pub type Tone = Role;

/// The default role for an informational status message.
pub const TONE_SECONDARY: Tone = Role::Fg(FgStep::Secondary);
/// The role for a plain status message.
pub const TONE_NORMAL: Tone = Role::Fg(FgStep::Primary);
/// The role for a success status message.
pub const TONE_SUCCESS: Tone = Role::Success;
/// The role for an error status message. Errors persist until the next action.
pub const TONE_ERROR: Tone = Role::Danger;

/// Identity of a long-running job that owns the status message.
///
/// A progress message ends when its job ends, not when another message
/// happens to arrive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobKey(pub &'static str);

impl JobKey {
    /// Name a job by its static name.
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// The job's name.
    pub const fn name(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Message<P> {
    text: String,
    tone: Tone,
    at: Moment,
    payload: P,
    job: Option<JobKey>,
}

/// The shell-owned status line: at most one message with a tone, a payload,
/// a time to live and an owning job.
///
/// Informational messages expire after [`StatusLine::DEFAULT_TTL_MS`] on the
/// feedback clock; error messages persist until the next acting input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLine<P = ()> {
    message: Option<Message<P>>,
    ttl_ms: u64,
    now: Moment,
}

impl<P> Default for StatusLine<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> StatusLine<P> {
    /// Information expires after five seconds on the clock.
    pub const DEFAULT_TTL_MS: u64 = 5_000;

    /// An empty status line.
    pub const fn new() -> Self {
        Self {
            message: None,
            ttl_ms: Self::DEFAULT_TTL_MS,
            now: Moment::ZERO,
        }
    }

    /// Set the time to live for informational messages.
    pub const fn ttl_ms(mut self, ms: u64) -> Self {
        self.ttl_ms = ms;
        self
    }

    /// Observe the clock so later messages carry its moment.
    pub fn observe(&mut self, now: Moment) {
        self.now = self.now.max(now);
    }

    /// Set a message with its tone and payload at the observed moment.
    pub fn set_with(&mut self, text: impl Into<String>, tone: Tone, payload: P) {
        self.message = Some(Message {
            text: text.into(),
            tone,
            at: self.now,
            payload,
            job: None,
        });
    }

    /// End the message owned by `job`. `true` when it was showing.
    pub fn end_job(&mut self, job: JobKey) -> bool {
        let showing = self.message.as_ref().is_some_and(|m| m.job == Some(job));
        if showing {
            self.message = None;
        }
        showing
    }

    /// The job that owns the current message.
    pub fn job(&self) -> Option<JobKey> {
        self.message.as_ref().and_then(|m| m.job)
    }

    /// Set a message with its tone, payload and clock moment.
    pub fn set_with_at(&mut self, text: impl Into<String>, tone: Tone, payload: P, now: Moment) {
        self.observe(now);
        self.set_with(text, tone, payload)
    }

    /// Clear the message. `true` when there was one.
    pub fn clear(&mut self) -> bool {
        self.message.take().is_some()
    }

    /// The current message text and tone.
    pub fn get(&self) -> Option<(&str, Tone)> {
        self.message.as_ref().map(|m| (m.text.as_str(), m.tone))
    }

    /// The current message text, tone and payload.
    pub fn entry(&self) -> Option<(&str, Tone, &P)> {
        self.message
            .as_ref()
            .map(|m| (m.text.as_str(), m.tone, &m.payload))
    }

    /// The payload that came with the message.
    pub fn payload(&self) -> Option<&P> {
        self.message.as_ref().map(|m| &m.payload)
    }

    /// The current message text.
    pub fn text(&self) -> Option<&str> {
        self.get().map(|(text, _)| text)
    }

    /// Is a message showing?
    pub fn is_some(&self) -> bool {
        self.message.is_some()
    }

    /// Is the line empty?
    pub fn is_none(&self) -> bool {
        self.message.is_none()
    }

    /// Is the current message an error?
    pub fn is_error(&self) -> bool {
        self.message.as_ref().is_some_and(|m| m.tone == TONE_ERROR)
    }

    /// Expire the message if its time to live passed on the clock. Errors
    /// never expire here; the next acting input clears them.
    pub fn expire(&mut self, now: Moment) -> bool {
        self.observe(now);
        let due = self.message.as_ref().is_some_and(|m| {
            m.tone != TONE_ERROR
                && u64::try_from(now.saturating_duration_since(m.at).as_millis())
                    .unwrap_or(u64::MAX)
                    >= self.ttl_ms
        });
        if due {
            self.message = None;
        }
        due
    }

    /// A key, a paste or an acting press clears an error. Returns whether the
    /// line changed, so the caller can fold that into its response.
    pub fn on_input(&mut self, input: &Input) -> bool {
        let acts = match input {
            Input::Key(_) | Input::Paste(_) => true,
            Input::Mouse(m) => matches!(m.kind, MouseKind::Down | MouseKind::Secondary),
            Input::Tick | Input::Resize(..) => false,
        };
        if acts && self.is_error() {
            self.message = None;
            return true;
        }
        false
    }
}

impl<P: Default> StatusLine<P> {
    /// Set a message with its tone and a default payload.
    pub fn set(&mut self, text: impl Into<String>, tone: Tone) {
        self.set_with(text, tone, P::default())
    }

    /// Set a message with its tone at clock moment `now`.
    pub fn set_at(&mut self, text: impl Into<String>, tone: Tone, now: Moment) {
        self.observe(now);
        self.set(text, tone)
    }

    /// Set an error message. It persists until the next acting input.
    pub fn error(&mut self, text: impl Into<String>) {
        self.set(text, TONE_ERROR)
    }

    /// Set a progress message owned by `job`.
    pub fn progress(&mut self, job: JobKey, text: impl Into<String>) {
        self.set(text, TONE_SECONDARY);
        if let Some(m) = self.message.as_mut() {
            m.job = Some(job);
        }
    }

    /// Set a progress message owned by `job` at clock moment `now`.
    pub fn progress_at(&mut self, job: JobKey, text: impl Into<String>, now: Moment) {
        self.observe(now);
        self.progress(job, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termrock_core::event::{Key, Mouse};
    use termrock_core::keys::{KeyCode, KeyModifiers};

    fn ms(n: u64) -> Moment {
        Moment::from_millis(n)
    }

    #[test]
    fn information_expires_after_five_seconds_on_the_clock() {
        let mut s = StatusLine::<()>::new();
        s.observe(ms(1_000));
        s.set("Saved", TONE_SECONDARY);
        assert!(!s.expire(ms(5_999)));
        assert_eq!(s.get(), Some(("Saved", TONE_SECONDARY)));
        assert!(s.expire(ms(6_000)));
        assert!(s.is_none());
    }

    #[test]
    fn a_clock_that_does_not_move_never_ages_a_message() {
        let mut s = StatusLine::<()>::new();
        s.set_at("Paused capture", TONE_NORMAL, ms(80));
        for _ in 0..100 {
            assert!(!s.expire(ms(80)));
        }
        assert_eq!(s.text(), Some("Paused capture"));
    }

    #[test]
    fn errors_persist_until_the_next_action() {
        let mut s = StatusLine::<()>::new();
        s.error("Connection refused");
        assert!(!s.expire(ms(60_000)));
        assert!(s.is_error());

        assert!(!s.on_input(&Input::Tick));
        assert!(!s.on_input(&Input::Resize(80, 24)));
        assert!(!s.on_input(&Input::Mouse(Mouse {
            kind: MouseKind::Move,
            pos: ratatui_core::layout::Position::new(1, 1),
            mods: KeyModifiers::NONE,
        })));
        assert!(s.is_error());

        let key = Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        };
        assert!(s.on_input(&Input::Key(key)));
        assert!(s.is_none());
        assert!(!s.on_input(&Input::Key(key)), "nothing left to clear");
    }

    #[test]
    fn a_progress_status_ends_with_its_job() {
        const SAVE: JobKey = JobKey::new("save");
        const LOAD: JobKey = JobKey::new("load");
        let mut s = StatusLine::<()>::new();
        s.progress_at(SAVE, "Saving settings…", ms(10));
        assert_eq!(s.job(), Some(SAVE));
        assert!(!s.end_job(LOAD), "another job's result leaves it");
        assert!(s.end_job(SAVE));
        assert!(s.is_none());

        s.progress(SAVE, "Saving settings…");
        s.set("Copied", TONE_SECONDARY);
        assert!(!s.end_job(SAVE));
        assert_eq!(s.text(), Some("Copied"));
    }

    #[test]
    fn a_newer_message_replaces_an_error() {
        let mut s = StatusLine::<()>::new();
        s.error("Refresh failed");
        s.set_at("Purged", TONE_SECONDARY, ms(100));
        assert_eq!(s.get(), Some(("Purged", TONE_SECONDARY)));
        assert!(s.expire(ms(5_100)));
    }

    #[test]
    fn a_new_error_replaces_an_error_and_the_ttl_is_configurable() {
        let mut s = StatusLine::<()>::new().ttl_ms(1_000);
        s.error("first");
        s.error("second");
        assert_eq!(s.text(), Some("second"));
        assert!(s.clear());
        s.set_at("info", TONE_NORMAL, ms(10));
        assert!(s.expire(ms(1_010)));
    }

    #[test]
    fn the_payload_comes_and_goes_with_its_message() {
        let mut s = StatusLine::<Vec<&str>>::new();
        s.set_with(
            "2 passed, 1 failed",
            TONE_ERROR,
            vec!["2 passed", "1 failed"],
        );
        assert_eq!(s.payload().map(Vec::len), Some(2));
        assert_eq!(s.entry().map(|(_, tone, _)| tone), Some(TONE_ERROR));
        let mouse = Mouse {
            kind: MouseKind::Down,
            pos: ratatui_core::layout::Position::new(0, 0),
            mods: KeyModifiers::NONE,
        };
        assert!(s.on_input(&Input::Mouse(mouse)));
        assert_eq!(s.payload(), None);
        s.set("plain", TONE_SECONDARY);
        assert_eq!(s.payload(), Some(&vec![]));
    }
}
