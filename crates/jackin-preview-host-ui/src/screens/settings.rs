use termrock::Id;

pub const ROOT: Id = Id::root("jackin.settings");
pub const FORM: Id = ROOT.sub("form");
pub const SAVE: Id = FORM.sub("save");
pub const TRUST: Id = FORM.sub("trust");

pub const TAB_NAMES: [&str; 5] = ["General", "Mounts", "Environments", "Agents", "Trust"];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsState {
    pub dirty: bool,
    pub save_attempts: u8,
    pub save_error: Option<String>,
    pub tab: u8,
}

impl SettingsState {
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.save_error = None;
    }

    pub fn begin_draft(&mut self) {
        self.dirty = true;
        self.save_attempts = 0;
        self.save_error = None;
    }

    pub const fn is_clean(&self) -> bool {
        !self.dirty
    }

    pub fn clear_error(&mut self) {
        self.save_error = None;
    }

    pub const fn cycle_tab(&mut self, next: bool) {
        self.tab = if next {
            self.tab.saturating_add(1) % 5
        } else if self.tab == 0 {
            4
        } else {
            self.tab.saturating_sub(1)
        };
    }

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
