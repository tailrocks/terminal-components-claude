#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Here,
    Project,
    Workspace,
    Host,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Risk {
    ReadOnly,
    Bounded,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    Ready,
    NeedsTrust(String),
    Blocked(String),
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionIntent {
    Canonical(String),
    FixtureCommand(FixtureCommand),
    Unavailable,
}

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
    pub id: String,
    pub title: String,
    pub kind: ActionKind,
    pub scope: Scope,
    pub scope_label: String,
    pub reason: String,
    pub risk: Risk,
    pub availability: Availability,
    pub target: String,
    pub command: String,
    pub workdir: String,
    pub long_running: bool,
    pub keywords: String,
}

impl Action {
    #[expect(
        clippy::too_many_arguments,
        reason = "fixture constructor mirrors the full scan record"
    )]
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

    pub fn remembered(mut self, row_id: String, reason: String, pinned: bool) -> Self {
        self.id = row_id;
        self.title = self.command.clone();
        self.kind = ActionKind::Task;
        self.reason = reason;
        self.scope = Scope::Here;
        self.scope_label = if pinned { "pin".into() } else { String::new() };
        self
    }

    pub fn fixture_memory(command: FixtureCommand, cwd: &str) -> Self {
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

    pub fn unavailable_memory(command: &str, cwd: &str) -> Self {
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

    pub fn changes(&self) -> String {
        match self.risk {
            Risk::ReadOnly => "nothing · inspection only".to_owned(),
            Risk::Bounded => format!("changes {} on this host", self.target),
            Risk::Broad => format!("irreversible across {}", self.target),
        }
    }
}
