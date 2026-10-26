use termrock_core::event::{Input, MouseKind};
use termrock_runtime::Moment;
use termrock_theme::{FgStep, Role};

pub type Tone = Role;

pub const TONE_SECONDARY: Tone = Role::Fg(FgStep::Secondary);

pub const TONE_NORMAL: Tone = Role::Fg(FgStep::Primary);

pub const TONE_SUCCESS: Tone = Role::Success;

pub const TONE_ERROR: Tone = Role::Danger;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobKey(pub &'static str);

impl JobKey {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

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
        self.message.as_ref().is_some_and(|m| m.tone == TONE_ERROR)
    }

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
    pub fn set(&mut self, text: impl Into<String>, tone: Tone) {
        self.set_with(text, tone, P::default())
    }

    pub fn set_at(&mut self, text: impl Into<String>, tone: Tone, now: Moment) {
        self.observe(now);
        self.set(text, tone)
    }

    pub fn error(&mut self, text: impl Into<String>) {
        self.set(text, TONE_ERROR)
    }

    pub fn progress(&mut self, job: JobKey, text: impl Into<String>) {
        self.set(text, TONE_SECONDARY);
        if let Some(m) = self.message.as_mut() {
            m.job = Some(job);
        }
    }

    pub fn progress_at(&mut self, job: JobKey, text: impl Into<String>, now: Moment) {
        self.observe(now);
        self.progress(job, text)
    }
}
