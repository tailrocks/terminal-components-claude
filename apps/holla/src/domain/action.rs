//! Action: one ranked, scoped, previewable intent. The catalogue derives
//! actions from discovered fixture truth; every suggestion carries its
//! reason, scope and risk — capability ≠ action ≠ recommendation.

use crate::domain::fixtures;

/// Context ring an action belongs to (CONCEPT.md §5: Here → Project →
/// Workspace → Host → Personal, cwd primary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    /// Under the launch directory.
    Here,
    /// The enclosing project / repository.
    Project,
    /// The wider workspace (monorepo root, project collection).
    Workspace,
    /// This machine or remote host as a whole.
    Host,
    /// The user's personal config and caches.
    Personal,
}

impl Scope {
    pub const ORDER: [Scope; 5] = [
        Scope::Here,
        Scope::Project,
        Scope::Workspace,
        Scope::Host,
        Scope::Personal,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Scope::Here => "here",
            Scope::Project => "project",
            Scope::Workspace => "workspace",
            Scope::Host => "host",
            Scope::Personal => "personal",
        }
    }

    /// Ring weight: closer scopes rank higher.
    pub fn weight(self) -> i32 {
        match self {
            Scope::Here => 100,
            Scope::Project => 80,
            Scope::Workspace => 60,
            Scope::Host => 40,
            Scope::Personal => 20,
        }
    }
}

/// How much damage a run could do; risk changes treatment, never
/// availability (CONCEPT.md §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Risk {
    /// Inspects only; runs directly.
    ReadOnly,
    /// Mutates a bounded, named target; asks once.
    Bounded,
    /// Broad or irreversible; two gates with a typed target-bound phrase.
    Broad,
}

impl Risk {
    pub fn confirmation(self) -> &'static str {
        match self {
            Risk::ReadOnly => "runs directly",
            Risk::Bounded => "asks once before changing anything",
            Risk::Broad => "two gates · typed target-bound phrase",
        }
    }
}

/// Whether the action can run right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    Ready,
    /// A mise task file that must be trusted first (exact path).
    NeedsTrust(String),
    /// Known but not runnable; the reason is shown, the row stays.
    Blocked(String),
}

/// What family the action belongs to (ordering tiebreak + preview wording).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionKind {
    Task,
    Git,
    Plan,
    Connect,
    Flow,
    Clone,
    System,
}

/// Effect identity is independent of the row that presents it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionIntent {
    Canonical(String),
    FixtureCommand(FixtureCommand),
    Unavailable,
}

/// Explicit simulation-only commands present in accepted ranking fixtures.
/// These are data labels, never executable process arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureCommand {
    MakeTest,
    CargoBuild,
    CargoTest,
    PnpmDev,
    DockerUsage,
}

impl FixtureCommand {
    pub fn command(self) -> &'static str {
        match self {
            Self::MakeTest => "make test",
            Self::CargoBuild => "cargo build",
            Self::CargoTest => "cargo test",
            Self::PnpmDev => "pnpm dev",
            Self::DockerUsage => "docker system df",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    intent: ActionIntent,
    /// Stable identity for focus, pins and modal tags (`task:test`,
    /// `git.pull`, `docker.cleanup`).
    pub id: String,
    pub title: String,
    pub kind: ActionKind,
    pub scope: Scope,
    /// Where the scope shows up as a place (`apps/frontend`, `prod-eu-1`).
    pub scope_label: String,
    /// Why this is recommended — the fact, not marketing.
    pub reason: String,
    pub risk: Risk,
    pub availability: Availability,
    /// The path/host/service the run acts on.
    pub target: String,
    /// The operation, previewable before any run (CONCEPT.md §6.7).
    pub command: String,
    /// Effective working directory of the run.
    pub workdir: String,
    /// Long-running work starts a named activity instead of a one-shot.
    pub long_running: bool,
    /// Extra search haystack, never displayed (a task's real command line).
    pub keywords: String,
}

impl Action {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        title: &str,
        kind: ActionKind,
        scope: Scope,
        reason: &str,
        risk: Risk,
        target: &str,
        command: &str,
    ) -> Self {
        Self {
            intent: ActionIntent::Canonical(id.into()),
            id: id.into(),
            title: title.into(),
            kind,
            scope,
            scope_label: String::new(),
            reason: reason.into(),
            risk,
            availability: Availability::Ready,
            target: target.into(),
            command: command.into(),
            workdir: String::new(),
            long_running: false,
            keywords: String::new(),
        }
    }

    pub fn intent(&self) -> &ActionIntent {
        &self.intent
    }

    pub fn intent_id(&self) -> Option<&str> {
        match &self.intent {
            ActionIntent::Canonical(id) => Some(id),
            _ => None,
        }
    }

    /// Preserve effect policy while presenting a separate stable memory row.
    pub(crate) fn remembered(mut self, row_id: String, reason: String, pinned: bool) -> Self {
        self.id = row_id;
        self.title = self.command.clone();
        self.kind = ActionKind::Task;
        self.reason = reason;
        self.scope = Scope::Here;
        self.scope_label = if pinned { "pin".into() } else { String::new() };
        self
    }

    pub(crate) fn fixture_memory(command: FixtureCommand, cwd: &str) -> Self {
        let mut action = Self::new(
            command.command(),
            command.command(),
            ActionKind::Task,
            Scope::Here,
            "fixture command",
            Risk::ReadOnly,
            cwd,
            command.command(),
        );
        action.intent = ActionIntent::FixtureCommand(command);
        action
    }

    pub(crate) fn unavailable_memory(command: &str, cwd: &str) -> Self {
        let mut action = Self::new(
            command,
            command,
            ActionKind::Task,
            Scope::Here,
            "stored command",
            Risk::Broad,
            cwd,
            command,
        );
        action.intent = ActionIntent::Unavailable;
        action.availability =
            Availability::Blocked("stored command has no current action policy".into());
        action
    }

    /// What changes when this runs — the preview's "What will change" row.
    pub fn changes(&self) -> String {
        match self.risk {
            Risk::ReadOnly => "nothing · inspection only".to_owned(),
            Risk::Bounded => format!("changes {} on this host", self.target),
            Risk::Broad => format!("irreversible across {}", self.target),
        }
    }

    /// Absolute display form of the target for confirmations (P3 phrases
    /// expand `~` through the fixture home).
    pub fn resolved_target(&self) -> String {
        fixtures::expand_home(&self.target)
    }
}
