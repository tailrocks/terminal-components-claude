//! Ranking memory: what the user pinned, the aliases they taught, and how
//! often each command ran from each path. Ranking must be inspectable —
//! these facts surface as reasons (`used 6 times in this project`).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub path: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    pub alias: String,
    pub expansion: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usage {
    pub path: String,
    pub command: String,
    pub count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory {
    pub pins: Vec<Pin>,
    pub aliases: Vec<Alias>,
    pub usage: Vec<Usage>,
    /// Rows the user hid at a path (same shape as a pin; reset restores).
    pub hides: Vec<Pin>,
}

impl Memory {
    pub fn usage_at(&self, path: &str, command: &str) -> u32 {
        self.usage
            .iter()
            .filter(|u| u.path == path && u.command == command)
            .map(|u| u.count)
            .sum()
    }

    pub fn pin_at(&self, path: &str, command: &str) -> bool {
        self.pins
            .iter()
            .any(|p| p.path == path && p.command == command)
    }

    pub fn hidden_at(&self, path: &str, command: &str) -> bool {
        self.hides
            .iter()
            .any(|p| p.path == path && p.command == command)
    }

    pub fn alias(&self, name: &str) -> Option<&str> {
        self.aliases
            .iter()
            .find(|a| a.alias == name)
            .map(|a| a.expansion.as_str())
    }

    /// The alias pointing at a command, for reason annotation (`alias gs`).
    pub fn alias_for(&self, command: &str) -> Option<&str> {
        self.aliases
            .iter()
            .find(|a| a.expansion == command)
            .map(|a| a.alias.as_str())
    }

    /// Reset every ranking fact a command carries at a path.
    pub fn reset_at(&mut self, path: &str, command: &str) {
        self.pins
            .retain(|p| !(p.path == path && p.command == command));
        self.hides
            .retain(|p| !(p.path == path && p.command == command));
        self.aliases.retain(|a| a.expansion != command);
    }
}
