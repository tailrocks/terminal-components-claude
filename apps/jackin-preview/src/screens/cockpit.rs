//! Launch cockpit state and controls.

use junie_tui::Id;

/// Cockpit root.
pub const ROOT: Id = Id::root("jackin.cockpit");
/// Stage list.
pub const STAGES: Id = ROOT.sub("stages");
/// Build log viewport.
pub const LOG: Id = ROOT.sub("log");
/// Cancel action.
pub const CANCEL: Id = ROOT.sub("cancel");
/// Retry action.
pub const RETRY: Id = ROOT.sub("retry");

/// Cockpit interaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CockpitState {
    /// Whether the launch log is visible.
    pub log_open: bool,
    /// Current launch-log scroll offset.
    pub log_scroll: u16,
}
