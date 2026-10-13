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

    /// Paint one fixture row, marking the row the cursor is on.
    pub fn marked_source(&self, label: &str, index: u8) -> String {
        if self.selection == index {
            format!("▸ {label}")
        } else {
            label.to_owned()
        }
    }

    /// Follow a source-list cursor the way `PreludeScreen::update` does.
    ///
    /// Only step 1 listens. A move of exactly one row changes the domain
    /// selection. Any other jump is ignored and the painted mark stays put.
    pub fn follow_source_cursor(&mut self, from: u8, to: u8) -> bool {
        if self.step != 1 {
            return false;
        }
        match i16::from(to) - i16::from(from) {
            1 => {
                self.move_selection(true);
                true
            }
            -1 => {
                self.move_selection(false);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_cursor_follows_only_a_one_row_move_on_step_one() {
        let mut state = PreludeState::default();
        assert!(!state.follow_source_cursor(0, 2));
        assert_eq!(state.selection(), 0);
        assert!(state.follow_source_cursor(0, 1));
        assert_eq!(state.selection(), 1);
        assert!(state.follow_source_cursor(1, 0));
        assert_eq!(state.selection(), 0);
        state.advance_flow();
        assert_eq!(state.step(), 2);
        assert!(!state.follow_source_cursor(0, 1));
        assert_eq!(state.selection(), 0);
    }
}
