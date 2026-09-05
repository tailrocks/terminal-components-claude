//! Settings route state.

use junie_tui::Id;

/// Settings root.
pub const ROOT: Id = Id::root("jackin.settings");
/// Settings form namespace.
pub const FORM: Id = ROOT.sub("form");
/// Save action.
pub const SAVE: Id = FORM.sub("save");

/// Settings draft and save lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsState {
    /// Whether the settings draft has unsaved changes.
    pub dirty: bool,
    /// Number of save attempts made for the current draft.
    pub save_attempts: u8,
    /// Latest save error, if one occurred.
    pub save_error: Option<String>,
}

impl SettingsState {
    /// Record one save attempt and return whether the draft should remain.
    pub fn attempt_save(&mut self, fails: bool) -> bool {
        self.save_attempts = self.save_attempts.saturating_add(1);
        if fails && self.save_attempts == 1 {
            self.save_error = Some("Settings error · host rejected the update".into());
            true
        } else {
            self.dirty = false;
            self.save_error = None;
            false
        }
    }
}
