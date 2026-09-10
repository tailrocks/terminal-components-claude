//! mise: tool versions and the project task runner. Tasks may be namespaced
//! (`//projects/frontend:build`); trust is per exact config file.

/// Whether a tool is usable here, absent, or behind the pinned latest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ToolState {
    Active,
    Missing,
    Outdated { latest: String },
}

impl ToolState {
    pub(crate) fn label(&self) -> String {
        match self {
            ToolState::Active => "active".to_owned(),
            ToolState::Missing => "missing".to_owned(),
            ToolState::Outdated { latest } => format!("outdated · latest {latest}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MiseTool {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) state: ToolState,
}

/// Trust state of the file that defines a task. Untrusted task files must
/// say so before their tasks run (CONCEPT.md §8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Trust {
    Trusted,
    Untrusted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MiseTask {
    /// `test` or a namespaced id like `//projects/frontend:build`.
    pub(crate) id: String,
    pub(crate) command: String,
    /// Exact file the task is defined by; trust attaches to this path.
    pub(crate) defined_in: String,
    pub(crate) trust: Trust,
}

impl MiseTask {
    pub(crate) fn namespaced(&self) -> bool {
        self.id.starts_with("//")
    }

    pub(crate) fn name(&self) -> &str {
        self.id.rsplit(':').next().unwrap_or(&self.id)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MiseState {
    pub(crate) tools: Vec<MiseTool>,
    pub(crate) tasks: Vec<MiseTask>,
}
