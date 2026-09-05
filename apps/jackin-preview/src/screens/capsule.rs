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
    /// Selected capsule tab index.
    pub tab: u8,
    /// Whether the selected pane is zoomed.
    pub zoomed: bool,
    /// Stable identifier of the selected pane.
    pub selected_pane: u64,
    /// Whether the pane context menu is open.
    pub context_open: bool,
}
