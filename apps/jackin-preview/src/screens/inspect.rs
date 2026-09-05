//! Read-only instance inspection composition.

use junie_tui::Id;

/// Inspector root.
pub const ROOT: Id = Id::root("jackin.inspect");
/// Inspector body.
pub const BODY: Id = ROOT.sub("body");

/// Inspection state.  It intentionally stores only an instance id; all
/// displayed fields are projected from the current `World` on every draw.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InspectState {
    /// Identifier of the instance being inspected, if selected.
    pub instance: Option<String>,
}
