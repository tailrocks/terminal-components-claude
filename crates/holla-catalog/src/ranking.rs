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
    aliases: Vec<Alias>,
    pub usage: Vec<Usage>,
    pub hides: Vec<Pin>,
}

impl Memory {
    pub fn seeded(pins: Vec<Pin>, aliases: Vec<Alias>, usage: Vec<Usage>, hides: Vec<Pin>) -> Self {
        let mut memory = Self {
            pins,
            aliases: Vec::new(),
            usage,
            hides,
        };
        for alias in aliases {
            memory.aliases.retain(|old| {
                alias_key(&old.alias) != alias_key(&alias.alias) && old.expansion != alias.expansion
            });
            memory.aliases.push(alias);
        }
        memory
    }

    pub fn aliases(&self) -> &[Alias] {
        &self.aliases
    }

    pub fn set_alias(&mut self, name: &str, command: &str) -> Result<(), AliasError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AliasError::Empty);
        }
        if self
            .aliases
            .iter()
            .any(|alias| alias_key(&alias.alias) == alias_key(name) && alias.expansion != command)
        {
            return Err(AliasError::NameOwned);
        }
        self.aliases.retain(|alias| alias.expansion != command);
        self.aliases.push(Alias {
            alias: name.into(),
            expansion: command.into(),
        });
        Ok(())
    }

    pub fn usage_at(&self, path: &str, command: &str) -> u32 {
        self.usage
            .iter()
            .filter(|u| u.path == path && u.command == command)
            .map(|u| u.count)
            .fold(0, u32::saturating_add)
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
            .find(|a| alias_key(&a.alias) == alias_key(name))
            .map(|a| a.expansion.as_str())
    }

    pub fn alias_for(&self, command: &str) -> Option<&str> {
        self.aliases
            .iter()
            .find(|a| a.expansion == command)
            .map(|a| a.alias.as_str())
    }

    pub fn reset_at(&mut self, path: &str, command: &str) {
        self.pins
            .retain(|p| !(p.path == path && p.command == command));
        self.hides
            .retain(|p| !(p.path == path && p.command == command));
        self.aliases.retain(|a| a.expansion != command);
    }
}

pub fn alias_key(name: &str) -> String {
    name.trim().to_lowercase()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasError {
    Empty,
    NameOwned,
}

impl std::fmt::Display for AliasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Empty => "alias must not be empty",
            Self::NameOwned => "alias belongs to another command",
        })
    }
}
impl std::error::Error for AliasError {}
