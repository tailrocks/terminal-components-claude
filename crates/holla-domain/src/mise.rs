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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trust {
    Trusted,
    Untrusted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiseTask {
    pub id: String,
    pub command: String,
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
