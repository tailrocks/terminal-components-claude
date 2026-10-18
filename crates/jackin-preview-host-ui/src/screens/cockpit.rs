use termrock::Id;

use jackin_preview_presentation::rain::{HANDOFF_LEN, HandoffStage, handoff_stage};

pub const ROOT: Id = Id::root("jackin.cockpit");
pub const STAGES: Id = ROOT.sub("stages");
pub const ACCOUNT_LINE: Id = ROOT.sub("account-line");
pub const LOG: Id = ROOT.sub("log");
pub const CANCEL: Id = ROOT.sub("cancel");
pub const RETRY: Id = ROOT.sub("retry");
pub const HANDOFF: Id = ROOT.sub("handoff");

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountLine {
    primary: Option<String>,
    additional: Vec<String>,
}

impl AccountLine {
    pub fn new<I, S>(primary: Option<S>, additional: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut line = Self {
            primary: None,
            additional: Vec::new(),
        };
        if let Some(primary) = primary {
            line.push_primary(primary);
        }
        for label in additional {
            line.push_additional(label);
        }
        line
    }

    pub fn from_labels<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut labels = labels.into_iter();
        let primary = labels.next();
        Self::new(primary, labels)
    }

    pub fn len(&self) -> usize {
        usize::from(self.primary.is_some()) + self.additional.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.primary.is_none() && self.additional.is_empty()
    }

    pub fn primary(&self) -> Option<&str> {
        self.primary.as_deref()
    }

    pub fn additional(&self) -> impl Iterator<Item = &str> {
        self.additional.iter().map(String::as_str)
    }

    pub fn text(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let count = self.len();
        let noun = if count == 1 { "account" } else { "accounts" };
        let labels = self
            .primary
            .iter()
            .chain(self.additional.iter())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" · ");
        Some(format!("{count} {noun} · {labels}"))
    }

    fn push_primary<S: Into<String>>(&mut self, label: S) {
        let label = label.into();
        if !label.is_empty() {
            self.primary = Some(label);
        }
    }

    fn push_additional<S: Into<String>>(&mut self, label: S) {
        let label = label.into();
        if !label.is_empty()
            && self.primary.as_deref() != Some(label.as_str())
            && !self.additional.iter().any(|existing| existing == &label)
        {
            self.additional.push(label);
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HandoffState {
    frame: u64,
    active: bool,
}

impl HandoffState {
    pub const fn start(&mut self) {
        self.frame = 0;
        self.active = true;
    }

    pub const fn frame(self) -> u64 {
        self.frame
    }

    pub const fn is_active(self) -> bool {
        self.active
    }

    pub const fn is_complete(self) -> bool {
        !self.active && self.frame >= HANDOFF_LEN
    }

    pub const fn stage(self) -> HandoffStage {
        if self.active || self.frame > 0 {
            handoff_stage(self.frame)
        } else {
            HandoffStage::Capsule
        }
    }

    pub const fn advance(&mut self) -> bool {
        if !self.active {
            return false;
        }
        self.frame = self.frame.saturating_add(1);
        if self.frame >= HANDOFF_LEN {
            self.active = false;
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CockpitState {
    pub log_open: bool,
    pub log_scroll: u16,
    pub account_line: AccountLine,
    pub handoff: HandoffState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_line_deduplicates_and_never_invents_material() {
        let line = AccountLine::new(
            Some("Claude · Work (1Password)"),
            ["Claude · Personal", "Claude · Personal", ""],
        );
        assert_eq!(line.len(), 2);
        assert_eq!(
            line.text().as_deref(),
            Some("2 accounts · Claude · Work (1Password) · Claude · Personal")
        );
        assert!(!line.text().is_some_and(|text| text.contains("valid-ant")));
    }

    #[test]
    fn handoff_starts_at_frame_zero_and_advances_only_when_called() {
        let mut handoff = HandoffState::default();
        assert!(!handoff.is_active());
        assert_eq!(handoff.stage(), HandoffStage::Capsule);
        handoff.start();
        assert_eq!(handoff.frame(), 0);
        assert_eq!(handoff.stage(), HandoffStage::CockpitDim(1));
        assert!(!handoff.advance());
        assert_eq!(handoff.frame(), 1);
        for _ in 1..HANDOFF_LEN - 1 {
            assert!(!handoff.advance());
        }
        assert_eq!(handoff.frame(), HANDOFF_LEN - 1);
        assert!(handoff.advance());
        assert!(handoff.is_complete());
        assert_eq!(handoff.frame(), HANDOFF_LEN);
    }
}
