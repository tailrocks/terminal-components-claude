use crate::clock::Moment;
use crate::core::event::{Input, MouseKind};
use crate::theme::Tone;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Message<P> {
    text: String,
    tone: Tone,
    at: Moment,
    payload: P,

    job: Option<JobKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobKey(&'static str);

impl JobKey {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    pub const fn name(self) -> &'static str {
        self.0
    }
}

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
    pub const DEFAULT_TTL_MS: u64 = 5_000;

    pub const fn new() -> Self {
        Self {
            message: None,
            ttl_ms: Self::DEFAULT_TTL_MS,
            now: Moment::ZERO,
        }
    }

    pub const fn ttl_ms(mut self, ms: u64) -> Self {
        self.ttl_ms = ms;
        self
    }

    pub fn observe(&mut self, now: Moment) {
        self.now = self.now.max(now);
    }

    pub fn set_with(&mut self, text: impl Into<String>, tone: Tone, payload: P) {
        self.message = Some(Message {
            text: text.into(),
            tone,
            at: self.now,
            payload,
            job: None,
        });
    }

    pub fn end_job(&mut self, job: JobKey) -> bool {
        let showing = self.message.as_ref().is_some_and(|m| m.job == Some(job));
        if showing {
            self.message = None;
        }
        showing
    }

    pub fn job(&self) -> Option<JobKey> {
        self.message.as_ref().and_then(|m| m.job)
    }

    pub fn set_with_at(&mut self, text: impl Into<String>, tone: Tone, payload: P, now: Moment) {
        self.observe(now);
        self.set_with(text, tone, payload)
    }

    pub fn clear(&mut self) -> bool {
        self.message.take().is_some()
    }

    pub fn get(&self) -> Option<(&str, Tone)> {
        self.message.as_ref().map(|m| (m.text.as_str(), m.tone))
    }

    pub fn entry(&self) -> Option<(&str, Tone, &P)> {
        self.message
            .as_ref()
            .map(|m| (m.text.as_str(), m.tone, &m.payload))
    }

    pub fn payload(&self) -> Option<&P> {
        self.message.as_ref().map(|m| &m.payload)
    }

    pub fn text(&self) -> Option<&str> {
        self.get().map(|(text, _)| text)
    }

    pub fn is_some(&self) -> bool {
        self.message.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.message.is_none()
    }

    pub fn is_error(&self) -> bool {
        self.message.as_ref().is_some_and(|m| m.tone == Tone::Error)
    }

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

impl<P: Default> StatusLine<P> {
    pub fn set(&mut self, text: impl Into<String>, tone: Tone) {
        self.set_with(text, tone, P::default())
    }

    pub fn set_at(&mut self, text: impl Into<String>, tone: Tone, now: Moment) {
        self.observe(now);
        self.set(text, tone)
    }

    pub fn error(&mut self, text: impl Into<String>) {
        self.set(text, Tone::Error)
    }

    pub fn progress(&mut self, job: JobKey, text: impl Into<String>) {
        self.set(text, Tone::Secondary);
        if let Some(m) = self.message.as_mut() {
            m.job = Some(job);
        }
    }

    pub fn progress_at(&mut self, job: JobKey, text: impl Into<String>, now: Moment) {
        self.observe(now);
        self.progress(job, text)
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
        let mut s = StatusLine::<()>::new();
        s.observe(ms(1_000));
        s.set("Saved", Tone::Secondary);
        assert!(!s.expire(ms(5_999)));
        assert_eq!(s.get(), Some(("Saved", Tone::Secondary)));
        assert!(s.expire(ms(6_000)));
        assert!(s.is_none());
    }

    #[test]
    fn a_clock_that_does_not_move_never_ages_a_message() {
        let mut s = StatusLine::<()>::new();
        s.set_at("Paused capture", Tone::Normal, ms(80));
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
        assert!(!s.on_input(&Input::Mouse(Mouse::new(
            MouseKind::Move,
            Position::new(1, 1)
        ))));
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
        s.set("Copied", Tone::Secondary);
        assert!(!s.end_job(SAVE));
        assert_eq!(s.text(), Some("Copied"));
    }

    #[test]
    fn a_newer_message_replaces_an_error() {
        let mut s = StatusLine::<()>::new();
        s.error("Refresh failed");
        s.set_at("Purged", Tone::Secondary, ms(100));
        assert_eq!(s.get(), Some(("Purged", Tone::Secondary)));
        assert!(s.expire(ms(5_100)));
    }

    #[test]
    fn a_new_error_replaces_an_error_and_the_ttl_is_configurable() {
        let mut s = StatusLine::<()>::new().ttl_ms(1_000);
        s.error("first");
        s.error("second");
        assert_eq!(s.text(), Some("second"));
        assert!(s.clear());
        s.set_at("info", Tone::Normal, ms(10));
        assert!(s.expire(ms(1_010)));
    }

    #[test]
    fn the_payload_comes_and_goes_with_its_message() {
        let mut s = StatusLine::<Vec<&str>>::new();
        s.set_with(
            "2 passed, 1 failed",
            Tone::Error,
            vec!["2 passed", "1 failed"],
        );
        assert_eq!(s.payload().map(Vec::len), Some(2));
        assert_eq!(s.entry().map(|(_, tone, _)| tone), Some(Tone::Error));
        let mouse = Mouse::new(MouseKind::Down, Position::new(0, 0));
        assert!(s.on_input(&Input::Mouse(mouse)));
        assert_eq!(s.payload(), None);
        s.set("plain", Tone::Secondary);
        assert_eq!(s.payload(), Some(&vec![]));
    }
}
