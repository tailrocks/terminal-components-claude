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
    aliases: Vec<Alias>,
    pub usage: Vec<Usage>,
    /// Rows the user hid at a path (same shape as a pin; reset restores).
    pub hides: Vec<Pin>,
}

impl Memory {
    /// Seed deterministic ranking facts while normalizing alias ownership.
    pub(crate) fn seeded(
        pins: Vec<Pin>,
        aliases: Vec<Alias>,
        usage: Vec<Usage>,
        hides: Vec<Pin>,
    ) -> Self {
        let mut memory = Self {
            pins,
            aliases: Vec::new(),
            usage,
            hides,
        };
        for alias in aliases {
            // Last seed declaration owns the name and command, just as an
            // explicit replacement does. Fixtures contain no external input.
            memory.aliases.retain(|old| {
                !old.alias.eq_ignore_ascii_case(&alias.alias) && old.expansion != alias.expansion
            });
            memory.aliases.push(alias);
        }
        memory
    }

    pub fn aliases(&self) -> &[Alias] {
        &self.aliases
    }

    /// Set or replace a command's alias without stealing another command's name.
    ///
    /// # Errors
    /// Refuses an empty name or a name already owned by a different command.
    pub fn set_alias(&mut self, name: &str, command: &str) -> Result<(), AliasError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AliasError::Empty);
        }
        if self
            .aliases
            .iter()
            .any(|alias| alias.alias.eq_ignore_ascii_case(name) && alias.expansion != command)
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
            .find(|a| a.alias.eq_ignore_ascii_case(name))
            .map(|a| a.expansion.as_str())
    }

    /// The alias pointing at a command, for reason annotation (`alias gs`).
    pub fn alias_for(&self, command: &str) -> Option<&str> {
        self.aliases
            .iter()
            .find(|a| a.expansion == command)
            .map(|a| a.alias.as_str())
    }

    /// Reset pin/hide at this path and the command’s global alias. Usage history
    /// stays intact, matching the accepted P5 ranking-reset scope.
    pub fn reset_at(&mut self, path: &str, command: &str) {
        self.pins
            .retain(|p| !(p.path == path && p.command == command));
        self.hides
            .retain(|p| !(p.path == path && p.command == command));
        self.aliases.retain(|a| a.expansion != command);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alias_replacement_is_effective_and_name_ownership_is_unique() {
        let mut memory = Memory::default();
        memory.set_alias("gs", "git status").unwrap();
        memory.set_alias("status", "git status").unwrap();
        assert_eq!(memory.alias("gs"), None);
        assert_eq!(memory.alias("STATUS"), Some("git status"));
        assert_eq!(
            memory.set_alias("status", "git diff"),
            Err(AliasError::NameOwned)
        );
        assert_eq!(memory.aliases().len(), 1);
    }
    #[test]
    fn reset_preserves_usage_and_other_paths() {
        let mut memory = Memory::seeded(
            vec![
                Pin {
                    path: "a".into(),
                    command: "git status".into(),
                },
                Pin {
                    path: "b".into(),
                    command: "git status".into(),
                },
            ],
            vec![],
            vec![Usage {
                path: "a".into(),
                command: "git status".into(),
                count: 6,
            }],
            vec![],
        );
        memory.set_alias("gs", "git status").unwrap();
        memory.reset_at("a", "git status");
        assert!(!memory.pin_at("a", "git status"));
        assert!(memory.pin_at("b", "git status"));
        assert_eq!(memory.alias("gs"), None);
        assert_eq!(memory.usage_at("a", "git status"), 6);
    }
}
