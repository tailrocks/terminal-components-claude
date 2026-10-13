//! Settings route state.

use junie_tui::Id;

/// Settings root.
pub const ROOT: Id = Id::root("jackin.settings");
/// Settings form namespace.
pub const FORM: Id = ROOT.sub("form");
/// Save action.
pub const SAVE: Id = FORM.sub("save");
/// Host-trust toggle.
pub const TRUST: Id = FORM.sub("trust");

/// Settings tabs. Trust is the only editable one on this host.
pub const TAB_NAMES: [&str; 5] = ["General", "Mounts", "Environments", "Agents", "Trust"];

/// Settings draft and save lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsState {
    /// Whether the settings draft has unsaved changes.
    pub dirty: bool,
    /// Number of save attempts made for the current draft.
    pub save_attempts: u8,
    /// Latest save error, if one occurred.
    pub save_error: Option<String>,
    /// Active settings tab. General is 0.
    pub tab: u8,
}

impl SettingsState {
    /// Record a user edit and retain it across a failed save.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.save_error = None;
    }

    /// Begin a new draft lifecycle, clearing attempts from the prior draft.
    pub fn begin_draft(&mut self) {
        self.dirty = true;
        self.save_attempts = 0;
        self.save_error = None;
    }

    /// Whether the current draft can be discarded without confirmation.
    pub const fn is_clean(&self) -> bool {
        !self.dirty
    }

    /// Clear a displayed save error while retaining the draft.
    pub fn clear_error(&mut self) {
        self.save_error = None;
    }

    /// Move to the next or previous tab, wrapping inside the strip.
    pub const fn cycle_tab(&mut self, next: bool) {
        self.tab = if next {
            self.tab.saturating_add(1) % 5
        } else if self.tab == 0 {
            4
        } else {
            self.tab.saturating_sub(1)
        };
    }

    /// One painted strip. The active tab carries the cursor mark.
    pub fn tab_line(&self) -> String {
        let active = usize::from(self.tab.min(4));
        TAB_NAMES
            .iter()
            .enumerate()
            .map(|(index, name)| {
                if index == active {
                    format!("▸ {name}")
                } else {
                    (*name).to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
    }

    /// Record one save attempt and return whether the draft should remain.
    pub fn attempt_save(&mut self, fails: bool) -> bool {
        self.save_attempts = self.save_attempts.saturating_add(1);
        if fails && self.save_attempts == 1 {
            self.dirty = true;
            self.save_error = Some("Settings error · host rejected the update".into());
            true
        } else {
            self.dirty = false;
            self.save_error = None;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_save_keeps_dirty_draft_and_retry_commits() {
        let mut state = SettingsState::default();
        state.begin_draft();
        assert!(state.attempt_save(true));
        assert!(!state.is_clean());
        assert!(state.save_error.is_some());

        assert!(!state.attempt_save(true));
        assert!(state.is_clean());
        assert!(state.save_error.is_none());
    }

    #[test]
    fn a_new_edit_clears_old_error_and_restarts_attempts() {
        let mut state = SettingsState::default();
        state.begin_draft();
        assert!(state.attempt_save(true));
        state.begin_draft();
        assert_eq!(state.save_attempts, 0);
        assert!(state.save_error.is_none());
        assert!(!state.is_clean());
    }

    #[test]
    fn tabs_wrap_and_mark_only_the_active_name() {
        let mut state = SettingsState::default();
        assert!(state.tab_line().starts_with("▸ General"));
        state.cycle_tab(false);
        assert_eq!(state.tab, 4);
        assert!(state.tab_line().contains("▸ Trust"));
        assert!(!state.tab_line().contains("▸ General"));
    }
}
