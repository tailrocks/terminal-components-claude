//! Workspace creation prelude state.

use junie_tui::Id;

/// Prelude composition root.
pub const ROOT: Id = Id::root("jackin.prelude");
/// Source browser control.
pub const SOURCE: Id = ROOT.sub("source");
/// Destination control.
pub const DESTINATION: Id = ROOT.sub("destination");
/// Continue action.
pub const CONTINUE: Id = ROOT.sub("continue");

/// Five-step workspace creation state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreludeState {
    step: u8,
    source: String,
    name: String,
    duplicate: bool,
    selection: u8,
}

impl Default for PreludeState {
    fn default() -> Self {
        Self {
            step: 1,
            source: "~/src/payments-platform".into(),
            name: "payments-platform".into(),
            duplicate: false,
            selection: 0,
        }
    }
}

impl PreludeState {
    /// Current one-based step.
    pub const fn step(&self) -> u8 {
        self.step
    }

    /// Current source path.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Proposed workspace name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Advance one step, bounded by the five-step flow.
    pub fn advance(&mut self) {
        self.step = self.step.saturating_add(1).min(5);
    }

    /// Rewind one step.
    pub fn back(&mut self) {
        self.step = self.step.saturating_sub(1).max(1);
    }

    /// Whether the proposed name collides with an existing workspace.
    pub const fn duplicate(&self) -> bool {
        self.duplicate
    }

    /// Mark duplicate-name validation result.
    pub const fn set_duplicate(&mut self, duplicate: bool) {
        self.duplicate = duplicate;
    }

    /// Move the source browser cursor within the deterministic fixture.
    pub fn move_selection(&mut self, down: bool) {
        if down {
            self.selection = self.selection.saturating_add(1).min(2);
        } else {
            self.selection = self.selection.saturating_sub(1);
        }
    }

    /// Choose the currently highlighted source and advance to destination.
    pub fn choose_source(&mut self) {
        self.source = match self.selection {
            1 => "~/src/customer-portal".into(),
            2 => "/Users/alexey/src/data-pipeline".into(),
            _ => "~/src/payments-platform".into(),
        };
        self.name = match self.selection {
            1 => "customer-portal".into(),
            2 => "data-pipeline".into(),
            _ => "new workspace".into(),
        };
        self.duplicate = self.selection == 1;
        self.step = 2;
    }

    /// Advance one prelude step while preserving skipped edit semantics.
    pub const fn advance_flow(&mut self) {
        self.step = match self.step {
            1 => 2,
            2 => 4,
            4 => 5,
            _ => 5,
        };
    }

    /// Return to the source browser.
    pub fn source_back(&mut self) {
        self.step = 1;
        self.source = "~/src".into();
    }

    /// Source browser cursor index.
    pub const fn selection(&self) -> u8 {
        self.selection
    }
}
