//! Read-only usage route state.

use junie_tui::Id;

/// Usage root.
pub const ROOT: Id = Id::root("jackin.usage");
/// Usage tab strip.
pub const TABS: Id = ROOT.sub("tabs");
/// Usage list.
pub const LIST: Id = ROOT.sub("list");
/// Read-only detail overlay.
pub const DETAIL: Id = ROOT.sub("detail");
/// Close action for the detail overlay.
pub const CLOSE: Id = DETAIL.sub("close");
/// Handoff action from usage detail to Accounts.
pub const MANAGE: Id = DETAIL.sub("manage");

/// Usage tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// Account health overview.
    #[default]
    Overview,
    /// Registration details.
    Registration,
    /// Provider quota.
    Quota,
}

impl Tab {
    /// Ordered tabs used by keyboard and click navigation.
    pub const ALL: [Self; 3] = [Self::Overview, Self::Registration, Self::Quota];

    /// Next tab, wrapping at the end of the strip.
    pub const fn next(self) -> Self {
        match self {
            Self::Overview => Self::Registration,
            Self::Registration => Self::Quota,
            Self::Quota => Self::Overview,
        }
    }

    /// Previous tab, wrapping at the beginning of the strip.
    pub const fn previous(self) -> Self {
        match self {
            Self::Overview => Self::Quota,
            Self::Registration => Self::Overview,
            Self::Quota => Self::Registration,
        }
    }
}

/// Read-only usage state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UsageState {
    /// Active usage tab.
    pub tab: Tab,
    /// Stable account id selected for the read-only detail projection.
    selected: Option<String>,
    /// Whether the detail overlay is visible.
    detail_open: bool,
}

impl UsageState {
    /// Select an account without copying or exposing usage material.
    pub fn select(&mut self, id: Option<impl Into<String>>) {
        self.selected = id.map(Into::into).filter(|id| !id.is_empty());
        if self.selected.is_none() {
            self.detail_open = false;
        }
    }

    /// Selected account id, if the list has one.
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Move to the next read-only tab.
    pub const fn next_tab(&mut self) {
        self.tab = self.tab.next();
    }

    /// Move to the previous read-only tab.
    pub const fn previous_tab(&mut self) {
        self.tab = self.tab.previous();
    }

    /// Open the detail overlay only when an account is selected.
    pub const fn open_detail(&mut self) -> bool {
        if self.selected.is_some() {
            self.detail_open = true;
        }
        self.detail_open
    }

    /// Close the detail overlay while retaining list selection.
    pub const fn close_detail(&mut self) {
        self.detail_open = false;
    }

    /// Whether the read-only detail overlay is visible.
    pub const fn detail_open(&self) -> bool {
        self.detail_open
    }

    /// Return the selected account for the Accounts handoff.
    pub fn manage_target(&self) -> Option<&str> {
        self.selected()
    }

    /// Restore the overview list and close any detail projection.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_detail_is_read_only_and_requires_selection() {
        let mut state = UsageState::default();
        assert!(!state.open_detail());
        state.select(Some("acct-claude"));
        assert!(state.open_detail());
        assert_eq!(state.manage_target(), Some("acct-claude"));
        state.close_detail();
        assert_eq!(state.selected(), Some("acct-claude"));
        assert!(!state.detail_open());
    }

    #[test]
    fn usage_tabs_wrap_without_mutating_selection() {
        let mut state = UsageState::default();
        state.select(Some("acct"));
        state.previous_tab();
        assert_eq!(state.tab, Tab::Quota);
        state.next_tab();
        assert_eq!(state.tab, Tab::Overview);
        assert_eq!(state.selected(), Some("acct"));
    }
}
