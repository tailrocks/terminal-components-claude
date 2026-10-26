use std::borrow::Cow;

use termrock_theme::Role;

use crate::keyboard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub key: &'static str,

    pub action: Cow<'static, str>,

    pub priority: u8,
}

pub const HINT_PINNED: u8 = 2;

pub const HINT_NORMAL: u8 = 1;

pub const HINT_AUTO: u8 = 0;

impl Hint {
    pub fn new(key: &'static str, action: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key,
            action: action.into(),
            priority: HINT_AUTO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterStatus {
    pub text: String,

    pub tone: Role,

    pub parts: Vec<(String, Role)>,
}

impl FooterStatus {
    pub fn is_error(&self) -> bool {
        self.tone == Role::Danger
    }
}

#[derive(Debug, Clone, Default)]
pub struct Footer {
    pub hints: Option<Vec<Hint>>,

    pub fallback: Vec<Hint>,

    pub editing: bool,

    pub tab_next: bool,

    pub status: Option<FooterStatus>,

    pub pin_dirty: Option<&'static str>,

    pub dirty: bool,
}

impl Footer {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn layer(mut self, hints: Option<Vec<Hint>>) -> Self {
        if self.hints.is_none() {
            self.hints = hints.filter(|h| !h.is_empty());
        }
        self
    }

    #[must_use]
    pub fn fallback(mut self, hints: Vec<Hint>) -> Self {
        self.fallback = hints;
        self
    }

    #[must_use]
    pub fn editing(mut self, on: bool) -> Self {
        self.editing = on;
        self
    }

    #[must_use]
    pub fn tab_next(mut self, on: bool) -> Self {
        self.tab_next = on;
        self
    }

    #[must_use]
    pub fn pin_dirty(mut self, key: Option<&'static str>) -> Self {
        self.pin_dirty = key;
        self
    }

    #[must_use]
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    #[must_use]
    pub fn status(mut self, status: Option<(&str, Role)>) -> Self {
        self.status = status.map(|(text, tone)| FooterStatus {
            text: text.to_owned(),
            tone,
            parts: Vec::new(),
        });
        self
    }

    #[must_use]
    pub fn status_parts(
        mut self,
        status: Option<(&str, Role)>,
        parts: Vec<(String, Role)>,
    ) -> Self {
        self.status = status.map(|(text, tone)| FooterStatus {
            text: text.to_owned(),
            tone,
            parts,
        });
        self
    }

    pub fn compose(&self) -> Vec<Hint> {
        let mut hints = self.hints.clone().unwrap_or_else(|| self.fallback.clone());
        if self.tab_next && !self.editing && !hints.iter().any(|h| h.key == "Tab") {
            hints.push(Hint::new("Tab", "Next"));
        }
        hints.retain(|h| keyboard::label_deliverable(h.key));
        if self.dirty
            && let Some(key) = self.pin_dirty
        {
            for h in hints.iter_mut().filter(|h| h.key == key) {
                h.priority = HINT_PINNED;
            }
        }
        if self.status.as_ref().is_some_and(FooterStatus::is_error) {
            for h in &mut hints {
                h.priority = HINT_NORMAL;
            }
        }
        hints
    }
}
