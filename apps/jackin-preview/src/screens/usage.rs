//! Read-only usage route state.

use junie_tui::Id;

/// Usage root.
pub const ROOT: Id = Id::root("jackin.usage");
/// Usage tab strip.
pub const TABS: Id = ROOT.sub("tabs");
/// Usage list.
pub const LIST: Id = ROOT.sub("list");

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

/// Read-only usage state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UsageState {
    pub tab: Tab,
}
