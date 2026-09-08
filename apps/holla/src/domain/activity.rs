//! Activities: named, scoped, long-running work that survives navigation.
//! Output is retained so a finished activity stays inspectable (§8.14).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActivityState {
    Running,
    Waiting,
    Succeeded,
    Failed,
    Detached,
}

impl ActivityState {
    pub(crate) fn label(self) -> &'static str {
        match self {
            ActivityState::Running => "running",
            ActivityState::Waiting => "waiting",
            ActivityState::Succeeded => "succeeded",
            ActivityState::Failed => "failed",
            ActivityState::Detached => "detached",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Activity {
    pub(crate) id: u32,
    pub(crate) name: String,
    /// Path scope the activity belongs to (ring: Here or a child project).
    pub(crate) scope: String,
    pub(crate) state: ActivityState,
    /// Virtual ms when it started (relative to the world clock's zero).
    pub(crate) started_ms: i64,
    /// Retained output lines, oldest first.
    pub(crate) lines: Vec<String>,
}

impl Activity {
    pub(crate) fn new(id: u32, name: &str, scope: &str, state: ActivityState, started_ms: i64) -> Self {
        Self {
            id,
            name: name.to_owned(),
            scope: scope.to_owned(),
            state,
            started_ms,
            lines: vec![],
        }
    }
}
