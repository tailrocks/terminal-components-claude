use termrock::Id;

pub const ROOT: Id = Id::root("jackin.prelude");
pub const SOURCE: Id = ROOT.sub("source");
pub const DESTINATION: Id = ROOT.sub("destination");
pub const CONTINUE: Id = ROOT.sub("continue");

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
    pub const fn step(&self) -> u8 {
        self.step
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn advance(&mut self) {
        self.step = self.step.saturating_add(1).min(5);
    }

    pub fn back(&mut self) {
        self.step = self.step.saturating_sub(1).max(1);
    }

    pub const fn duplicate(&self) -> bool {
        self.duplicate
    }

    pub const fn set_duplicate(&mut self, duplicate: bool) {
        self.duplicate = duplicate;
    }

    pub fn move_selection(&mut self, down: bool) {
        if down {
            self.selection = self.selection.saturating_add(1).min(2);
        } else {
            self.selection = self.selection.saturating_sub(1);
        }
    }

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

    pub const fn advance_flow(&mut self) {
        self.step = match self.step {
            1 => 2,
            2 => 4,
            4 => 5,
            _ => 5,
        };
    }

    pub fn source_back(&mut self) {
        self.step = 1;
        self.source = "~/src".into();
    }

    pub const fn selection(&self) -> u8 {
        self.selection
    }

    pub fn marked_source(&self, label: &str, index: u8) -> String {
        if self.selection == index {
            format!("▸ {label}")
        } else {
            label.to_owned()
        }
    }

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
