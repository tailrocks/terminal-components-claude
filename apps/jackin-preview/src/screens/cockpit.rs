//! Launch cockpit state and controls.

use junie_tui::Id;

use crate::rain::{HANDOFF_LEN, HandoffStage, handoff_stage};

/// Cockpit root.
pub const ROOT: Id = Id::root("jackin.cockpit");
/// Stage list.
pub const STAGES: Id = ROOT.sub("stages");
/// Safe account/credential projection below the stage rail.
pub const ACCOUNT_LINE: Id = ROOT.sub("account-line");
/// Build log viewport.
pub const LOG: Id = ROOT.sub("log");
/// Cancel action.
pub const CANCEL: Id = ROOT.sub("cancel");
/// Retry action.
pub const RETRY: Id = ROOT.sub("retry");
/// Cockpit-to-Capsule transition surface.
pub const HANDOFF: Id = ROOT.sub("handoff");

/// Safe account labels projected below a launch rail.
///
/// The cockpit receives display labels only.  It never stores credential
/// material or provider responses; the primary label may include its safe
/// source annotation while the remaining labels are plain account titles.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountLine {
    primary: Option<String>,
    additional: Vec<String>,
}

impl AccountLine {
    /// Build a line from the primary account and the other effective accounts.
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

    /// Build a line from labels in display order.
    pub fn from_labels<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut labels = labels.into_iter();
        let primary = labels.next();
        Self::new(primary, labels)
    }

    /// Number of unique account labels in this line.
    pub fn len(&self) -> usize {
        usize::from(self.primary.is_some()) + self.additional.len()
    }

    /// Whether no account label is available.
    pub const fn is_empty(&self) -> bool {
        self.primary.is_none() && self.additional.is_empty()
    }

    /// Primary account label, including its optional safe source annotation.
    pub fn primary(&self) -> Option<&str> {
        self.primary.as_deref()
    }

    /// Additional effective-account labels in stable display order.
    pub fn additional(&self) -> impl Iterator<Item = &str> {
        self.additional.iter().map(String::as_str)
    }

    /// Render the account line without credential material.
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

/// Tick-driven cockpit-to-Capsule handoff.
///
/// Frame zero is the first visible cockpit-dim frame.  A caller advances this
/// state only for a product tick; bootstrap, input, and repaint passes must not
/// consume a handoff frame.  Completion is reported on the tick after frame
/// `HANDOFF_LEN - 1` was visible.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HandoffState {
    frame: u64,
    active: bool,
}

impl HandoffState {
    /// Start at the first handoff frame without consuming a tick.
    pub const fn start(&mut self) {
        self.frame = 0;
        self.active = true;
    }

    /// Current handoff frame.  Frame zero is the first visible frame.
    pub const fn frame(self) -> u64 {
        self.frame
    }

    /// Whether the transition is currently active.
    pub const fn is_active(self) -> bool {
        self.active
    }

    /// Whether the transition has consumed all product ticks.
    pub const fn is_complete(self) -> bool {
        !self.active && self.frame >= HANDOFF_LEN
    }

    /// Current cross-fade phase for painting.
    pub const fn stage(self) -> HandoffStage {
        if self.active || self.frame > 0 {
            handoff_stage(self.frame)
        } else {
            HandoffStage::Capsule
        }
    }

    /// Consume one product tick and report whether the route may switch.
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

/// Cockpit interaction state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CockpitState {
    /// Whether the launch log is visible.
    pub log_open: bool,
    /// Current launch-log scroll offset.
    pub log_scroll: u16,
    /// Safe account labels shown beneath the stage rail.
    pub account_line: AccountLine,
    /// Tick-owned handoff state.
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
