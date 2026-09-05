//! Capsule route state and public pane controls.

use junie_tui::Id;

/// Capsule root.
pub const ROOT: Id = Id::root("jackin.capsule");
/// Capsule tabs.
pub const TABS: Id = ROOT.sub("tabs");
/// Pane list.
pub const PANES: Id = ROOT.sub("panes");
/// New-tab action.
pub const NEW_TAB: Id = ROOT.sub("new-tab");
/// Split action.
pub const SPLIT: Id = ROOT.sub("split");
/// Detach action.
pub const DETACH: Id = ROOT.sub("detach");

/// Capsule interaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CapsuleState {
    pub tab: u8,
    pub zoomed: bool,
    pub selected_pane: u64,
    pub context_open: bool,
}
