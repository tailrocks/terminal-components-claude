//! mise: tool versions and the project task runner. Tasks may be namespaced
//! (`//projects/frontend:build`); trust is per exact config file.

/// Whether a tool is usable here, absent, or behind the pinned latest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolState {
    Active,
    Missing,
    Outdated { latest: String },
}

impl ToolState {
    pub fn label(&self) -> String {
        match self {
            ToolState::Active => "active".to_owned(),
            ToolState::Missing => "missing".to_owned(),
            ToolState::Outdated { latest } => format!("outdated · latest {latest}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiseTool {
    pub name: String,
    pub version: String,
    pub state: ToolState,
}

/// Trust state of the file that defines a task. Untrusted task files must
/// say so before their tasks run (CONCEPT.md §8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trust {
    Trusted,
    Untrusted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiseTask {
    /// `test` or a namespaced id like `//projects/frontend:build`.
    pub id: String,
    pub command: String,
    /// Exact file the task is defined by; trust attaches to this path.
    pub defined_in: String,
    pub trust: Trust,
}

impl MiseTask {
    pub fn namespaced(&self) -> bool {
        self.id.starts_with("//")
    }

    pub fn name(&self) -> &str {
        self.id.rsplit(':').next().unwrap_or(&self.id)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MiseState {
    pub tools: Vec<MiseTool>,
    pub tasks: Vec<MiseTask>,
}
