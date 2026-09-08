//! Captured Manager commands and deterministic simulation effects.
//!
//! Rendering never calls this reducer. Reviews retain their original target;
//! confirmation and completion validate it again, independently of selection.

use crate::domain::{
    account::AccountId,
    agent::Agent,
    fixtures,
    instance::{DaemonSnapshot, Instance, InstanceStatus, RunId},
    workspace::{Workspace, WorkspaceId},
};
use crate::sim::{
    pty::Daemon,
    world::{DaemonHealth, GithubRepo, Msg, World},
};

/// Stable object selected by a Manager command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Durable instance key.
    Instance(String),
    /// Saved workspace key.
    Workspace(WorkspaceId),
}

/// Domain actions; launch/editor/prelude navigation stays with the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Attach to a live instance or restore a preserved one.
    Reconnect,
    /// Open read-only public instance facts.
    Inspect,
    /// Review stopping a live container.
    Stop,
    /// Review irreversible simulated removal of recovery state.
    Purge,
    /// Schedule a derived-image prewarm.
    Prewarm,
    /// Review removal of saved workspace configuration.
    Delete,
    /// Choose an agent/account for a live instance.
    NewSession,
    /// Start a shell without a provider account.
    Shell,
}

/// Semantic emphasis for app-owned shared property rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactTone {
    /// Ordinary public data.
    Normal,
    /// Supporting or inactive data.
    Secondary,
    /// Recovery risk or unavailable daemon.
    Warning,
    /// Destructive action scope.
    Error,
}

/// A public, non-secret inspection or confirmation fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    /// Human-readable fact label.
    pub label: &'static str,
    /// Non-secret display or copy value.
    pub value: String,
    /// Whether the app may expose a copy action.
    pub copyable: bool,
    /// Source-authored semantic emphasis, before theme overrides.
    pub tone: FactTone,
}

/// A session choice has no magic account sentinel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionChoice {
    /// Start a shell without a provider account.
    Shell,
    /// An explicit agent and selected provider account.
    Agent {
        /// Selected agent runtime.
        agent: Agent,
        /// Selected canonical account identity.
        account: Option<AccountId>,
    },
}

/// Captured durable incarnation, not a current row index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceTarget {
    /// Durable instance identifier.
    pub id: String,
    /// Captured launch incarnation.
    pub run_id: RunId,
}

#[derive(Debug, Clone)]
struct CapturedInstance(Box<Instance>, Option<Box<Workspace>>);
impl CapturedInstance {
    fn valid(&self, world: &World) -> bool {
        self.0.workspace.and_then(|id| world.workspace(id)) == self.1.as_deref()
            && world.instance(&self.0.id).is_some_and(|now| {
                // Daemon snapshots and observation timestamps advance naturally.
                // All reviewed durable identity, lifecycle and destructive facts stay fixed.
                let mut expected = self.0.as_ref().clone();
                expected.daemon = now.daemon.clone();
                expected.last_seen_secs = now.last_seen_secs;
                expected == *now
            })
    }
    fn target(&self) -> InstanceTarget {
        InstanceTarget {
            id: self.0.id.clone(),
            run_id: self.0.run_id,
        }
    }
}

/// Immutable destructive review; only this module can construct its capture.
#[derive(Debug, Clone)]
pub struct Review {
    token: u64,
    action: Action,
    captured: Capture,
    title: String,
    body: String,
    facts: Vec<Fact>,
    acknowledgement: Option<String>,
}
impl Review {
    /// Exact required acknowledgement, if any.
    pub fn acknowledgement(&self) -> Option<&str> {
        self.acknowledgement.as_deref()
    }
    /// Action described by this immutable review.
    pub const fn action(&self) -> Action {
        self.action
    }
    /// Captured confirmation heading.
    pub fn title(&self) -> &str {
        &self.title
    }
    /// Captured confirmation explanation.
    pub fn body(&self) -> &str {
        &self.body
    }
    /// Captured non-secret review facts.
    pub fn facts(&self) -> &[Fact] {
        &self.facts
    }
}
#[derive(Debug, Clone)]
enum Capture {
    Instance(CapturedInstance),
    Workspace(Box<Workspace>),
}
impl Capture {
    fn valid(&self, world: &World) -> bool {
        match self {
            Self::Instance(i) => i.valid(world),
            Self::Workspace(w) => world.workspace(w.id) == Some(w.as_ref()),
        }
    }
    fn target(&self) -> Target {
        match self {
            Self::Instance(i) => Target::Instance(i.0.id.clone()),
            Self::Workspace(w) => Target::Workspace(w.id),
        }
    }
}

/// Instance context retained while the app's account picker is open.
#[derive(Debug, Clone)]
pub struct SessionReview {
    token: u64,
    captured: CapturedInstance,
}
impl SessionReview {
    /// Captured durable instance incarnation.
    pub fn target(&self) -> InstanceTarget {
        self.captured.target()
    }
    /// Canonical role used for account policy.
    pub fn role(&self) -> &str {
        &self.captured.0.role
    }
    /// Captured workspace association for account policy.
    pub const fn workspace(&self) -> Option<WorkspaceId> {
        self.captured.0.workspace
    }
}

/// Shell-facing effects. The app supplies real shared overlays/navigation.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No state or navigation change.
    None,
    /// Operator feedback without navigation.
    Status(String),
    /// Source error feedback requiring the app error presentation.
    Error(String),
    /// Attach to the captured live/restored instance.
    Attach {
        /// Validated destination instance.
        target: InstanceTarget,
        /// Optional restore feedback.
        status: Option<String>,
    },
    /// Read-only inspection overlay data.
    Inspect {
        /// Overlay heading.
        title: String,
        /// Public fact rows.
        facts: Vec<Fact>,
    },
    /// Destructive review requiring a later explicit confirmation.
    Confirm(Review),
    /// Open an instance-scoped session picker.
    ChooseSession(SessionReview),
    /// Create a session in the captured live instance.
    NewSession {
        /// Validated destination instance.
        target: InstanceTarget,
        /// Validated explicit session choice.
        choice: SessionChoice,
    },
    /// A validated simulated repository open; no host browser is launched.
    RepositoryOpened {
        /// Captured source URL.
        url: String,
        /// Source operator feedback.
        status: String,
    },
    /// Configuration removed; app reconciles selection.
    WorkspaceDeleted {
        /// Deleted saved workspace identifier.
        id: WorkspaceId,
        /// Source-derived completion feedback.
        status: String,
    },
}

#[derive(Debug, Clone)]
struct Pending {
    id: u64,
    due_ms: i64,
    action: Action,
    capture: Capture,
}

/// Sole owner of busy rows and pending completion tokens.
#[derive(Debug, Clone, Default)]
pub struct ManagerActions {
    last_refresh_error: bool,
    pending: Vec<Pending>,
}
impl ManagerActions {
    /// Apply the source five-second refresh on a Manager product tick.
    /// First failure latches stale health; later refreshes retain that observation.
    pub fn refresh(&mut self, world: &mut World) -> bool {
        if world.now_secs().saturating_sub(world.last_refresh_secs) < 5 {
            return false;
        }
        world.last_refresh_secs = world.now_secs();
        if world.refresh_fails && !self.last_refresh_error {
            self.last_refresh_error = true;
            world.daemon_health = DaemonHealth::Stale;
            world.schedule(0, Msg::Refreshed { ok: false });
        } else {
            fixtures::pinned::refresh_snapshots(world);
            let now = world.now_secs();
            for instance in &mut world.instances {
                if instance.status.is_live() {
                    instance.last_seen_secs = now;
                }
            }
        }
        true
    }

    /// Current operation label from the sole pending-operation owner.
    pub fn busy(&self, target: &Target) -> Option<&'static str> {
        self.pending
            .iter()
            .find(|p| match (&p.capture, target) {
                (Capture::Instance(i), Target::Instance(id)) => &i.0.id == id,
                (Capture::Workspace(w), Target::Workspace(id)) => w.id == *id,
                _ => false,
            })
            .map(|p| match p.action {
                Action::Stop => "stopping…",
                Action::Purge => "purging…",
                _ => "prewarming…",
            })
    }

    /// Compute action availability from the same lifecycle used by execution.
    pub fn available(action: Action, target: &Target, world: &World) -> bool {
        match target {
            Target::Workspace(id) => {
                world.workspace(*id).is_some() && matches!(action, Action::Prewarm | Action::Delete)
            }
            Target::Instance(id) => world.instance(id).is_some_and(|i| match action {
                Action::Reconnect => {
                    i.status.reconnectable() && i.status != InstanceStatus::Crashed
                }
                Action::Inspect | Action::Purge => true,
                Action::Stop => i.status.stoppable(),
                Action::NewSession | Action::Shell => i.status.is_live(),
                _ => false,
            }),
        }
    }

    /// Resolve one explicit command against current durable state.
    pub fn request(&mut self, action: Action, target: Target, world: &mut World) -> Effect {
        if self.busy(&target).is_some() {
            return Effect::Status("An operation is already running for this target".into());
        }
        match target {
            Target::Instance(id) => {
                let Some(instance) = world.instance(&id).cloned() else {
                    return Effect::None;
                };
                let workspace = instance
                    .workspace
                    .and_then(|id| world.workspace(id))
                    .cloned();
                let captured = CapturedInstance(Box::new(instance), workspace.map(Box::new));
                match action {
                    Action::Reconnect => reconnect(captured, world),
                    Action::Inspect => inspect(&captured.0, world),
                    Action::NewSession if captured.0.status.is_live() => {
                        let Some(token) = world.next_manager_operation() else {
                            return Effect::None;
                        };
                        Effect::ChooseSession(SessionReview { token, captured })
                    }
                    Action::Shell if captured.0.status.is_live() => Effect::NewSession {
                        target: captured.target(),
                        choice: SessionChoice::Shell,
                    },
                    Action::Stop if !captured.0.status.stoppable() => {
                        Effect::Status(format!("{} is not running · nothing to stop", short(&id)))
                    }
                    Action::Stop | Action::Purge => {
                        let Some(token) = world.next_manager_operation() else {
                            return Effect::None;
                        };
                        Effect::Confirm(review_instance(token, action, captured, world))
                    }
                    _ => Effect::None,
                }
            }
            Target::Workspace(id) => {
                let Some(workspace) = world.workspace(id).cloned() else {
                    return Effect::None;
                };
                match action {
                    Action::Delete => {
                        let count = world.instances_of(Some(id)).len();
                        let Some(token) = world.next_manager_operation() else {
                            return Effect::None;
                        };
                        Effect::Confirm(Review {
                            token,
                            action,
                            title: format!("Delete workspace {}?", workspace.name),
                            body: format!(
                                "The saved configuration is removed. {} and files on disk are kept.",
                                if count == 0 {
                                    "Instances".to_owned()
                                } else {
                                    plural(count, "instance record", "instance records")
                                }
                            ),
                            facts: Vec::new(),
                            acknowledgement: None,
                            captured: Capture::Workspace(Box::new(workspace)),
                        })
                    }
                    Action::Prewarm => {
                        self.schedule(action, Capture::Workspace(Box::new(workspace)), world)
                    }
                    _ => Effect::None,
                }
            }
        }
    }

    /// Cancel by dropping a Review. Only an exact acknowledgement can confirm purge.
    pub fn confirm(&mut self, review: Review, acknowledgement: &str, world: &mut World) -> Effect {
        if !world.consume_manager_review(review.token) || !review.captured.valid(world) {
            return stale();
        }
        if review
            .acknowledgement
            .as_deref()
            .is_some_and(|expected| expected != acknowledgement)
        {
            return Effect::Status("Acknowledgement does not match · nothing changed".into());
        }
        if self.busy(&review.captured.target()).is_some() {
            return Effect::Status("An operation is already running for this target".into());
        }
        match (review.action, review.captured) {
            (Action::Delete, Capture::Workspace(workspace)) => {
                world.workspaces.retain(|w| w.id != workspace.id);
                for instance in &mut world.instances {
                    if instance.workspace == Some(workspace.id) {
                        instance.workspace = None;
                    }
                }
                Effect::WorkspaceDeleted {
                    id: workspace.id,
                    status: format!(
                        "Deleted workspace {} · instances and files kept",
                        workspace.name
                    ),
                }
            }
            (action @ (Action::Stop | Action::Purge), capture @ Capture::Instance(_)) => {
                self.schedule(action, capture, world)
            }
            _ => Effect::None,
        }
    }

    fn schedule(&mut self, action: Action, capture: Capture, world: &mut World) -> Effect {
        let Some(id) = world.next_manager_operation() else {
            return Effect::Status("Operation identity exhausted".into());
        };
        let (delay, status) = match action {
            Action::Stop => (1_800, "Stopping…"),
            Action::Purge => (2_200, "Purging…"),
            _ => (2_400, "Prewarming the derived image…"),
        };
        self.pending.push(Pending {
            id,
            due_ms: world.now_ms().saturating_add(delay),
            action,
            capture,
        });
        world.schedule(delay, Msg::ManagerOperation { operation: id });
        Effect::Status(status.into())
    }

    /// Consume a scheduled token once; stale or duplicate results cannot mutate a target.
    pub fn complete(&mut self, world: &mut World, operation: u64) -> Effect {
        let Some(index) = self.pending.iter().position(|p| p.id == operation) else {
            return Effect::None;
        };
        if self
            .pending
            .get(index)
            .is_some_and(|pending| world.now_ms() < pending.due_ms)
        {
            return Effect::None;
        }
        let pending = self.pending.remove(index);
        if !pending.capture.valid(world) {
            return stale();
        }
        match (pending.action, pending.capture) {
            (Action::Prewarm, Capture::Workspace(workspace)) => Effect::Status(format!(
                "Prewarmed {} · derived image cached",
                workspace.name
            )),
            (action @ (Action::Stop | Action::Purge), Capture::Instance(captured)) => {
                let now = world.now_secs();
                let id = captured.0.id;
                let Some(instance) = world.instance_mut(&id) else {
                    return Effect::None;
                };
                if action == Action::Stop {
                    instance.status = if instance.is_dirty() {
                        InstanceStatus::PreservedDirty
                    } else {
                        InstanceStatus::RestoreAvailable
                    };
                    instance.last_seen_secs = now;
                } else {
                    instance.status = InstanceStatus::Purged;
                }
                world.daemons.remove(&id);
                if action == Action::Stop {
                    fixtures::pinned::refresh_snapshots(world);
                }
                world.sync_arbiter();
                Effect::Status(if action == Action::Stop {
                    format!("Instance {} stopped", short(&id))
                } else {
                    format!(
                        "Purged {} · container, sidecar, volume and recovery state removed",
                        short(&id)
                    )
                })
            }
            _ => Effect::None,
        }
    }

    /// Validate a captured session picker result before navigation.
    pub fn session(
        &self,
        review: SessionReview,
        choice: SessionChoice,
        world: &mut World,
    ) -> Effect {
        if !world.consume_manager_review(review.token)
            || !review.captured.valid(world)
            || !review.captured.0.status.is_live()
        {
            return stale();
        }
        if self
            .busy(&Target::Instance(review.captured.0.id.clone()))
            .is_some()
        {
            return stale();
        }
        if let SessionChoice::Agent { agent, account } = &choice {
            let workspace = review.workspace().and_then(|id| world.workspace(id));
            let offer = world.offer_for(*agent, workspace, Some(review.role()));
            if !account
                .as_ref()
                .is_some_and(|id| offer.accounts.contains(id))
            {
                return Effect::Status("Selected account is no longer available".into());
            }
        }
        Effect::NewSession {
            target: review.target(),
            choice,
        }
    }
}

fn short(id: &str) -> &str {
    id.trim_start_matches("jk-")
}
fn stale() -> Effect {
    Effect::Status("Target changed · nothing changed".into())
}
fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}
fn fact(label: &'static str, value: impl Into<String>, copyable: bool) -> Fact {
    Fact {
        label,
        value: value.into(),
        copyable,
        tone: FactTone::Normal,
    }
}

impl Fact {
    fn tone(mut self, tone: FactTone) -> Self {
        self.tone = tone;
        self
    }
}

fn reconnect(captured: CapturedInstance, world: &mut World) -> Effect {
    let id = captured.0.id.clone();
    let target = captured.target();
    match captured.0.status {
        InstanceStatus::Running => Effect::Attach {
            target,
            status: None,
        },
        InstanceStatus::RestoreAvailable
        | InstanceStatus::PreservedDirty
        | InstanceStatus::PreservedUnpushed => {
            let now = world.now_secs();
            if let Some(instance) = world.instance_mut(&id) {
                instance.status = InstanceStatus::Running;
                instance.last_seen_secs = now;
            }
            let mut daemon = Daemon::new(captured.0.workdir.trim_start_matches("/workspace/"));
            daemon.new_tab(Some(captured.0.agent), None, world.now_ms(), false);
            world.daemons.insert(id.clone(), daemon);
            fixtures::pinned::refresh_snapshots(world);
            world.sync_arbiter();
            Effect::Attach {
                target,
                status: Some(format!("Restored {} · container restarted", short(&id))),
            }
        }
        InstanceStatus::Crashed => Effect::Error(format!(
            "Cannot reconnect: {} crashed (exit 137) · inspect or purge it",
            short(&id)
        )),
        InstanceStatus::FailedSetup => Effect::Error(format!(
            "Cannot reconnect: {} never reached the Capsule · launch again",
            short(&id)
        )),
        InstanceStatus::CleanExited => Effect::Status(format!(
            "{} exited cleanly · launch the Workspace to start a new instance",
            short(&id)
        )),
        _ => Effect::None,
    }
}

fn review_instance(
    token: u64,
    action: Action,
    captured: CapturedInstance,
    world: &World,
) -> Review {
    let i = &captured.0;
    let short = short(&i.id);
    let (title, body, facts, acknowledgement) = if action == Action::Stop {
        (
            format!("Stop instance {short}?"),
            format!(
                "The container stops and every session in it ends. The instance record stays reconnectable ({}).",
                if i.is_dirty() {
                    "preserved with uncommitted work"
                } else {
                    "restore available"
                }
            ),
            Vec::new(),
            None,
        )
    } else {
        let workspace = i
            .workspace
            .and_then(|id| world.workspace(id))
            .map_or("current directory", |w| w.name.as_str());
        (
            format!("Purge instance {short}?"),
            String::new(),
            vec![
                fact(
                    "Action",
                    "purge the instance record and its container",
                    false,
                ),
                fact(
                    "Target",
                    format!(
                        "{short} · {workspace} › {} · {}",
                        role_label(world, &i.role),
                        i.agent.label()
                    ),
                    false,
                ),
                fact(
                    "Scope",
                    format!(
                        "1 container · {} · {}",
                        plural(
                            i.sessions.as_ref().map_or(0, Vec::len),
                            "preserved session",
                            "preserved sessions"
                        ),
                        if i.is_dirty() {
                            "worktree with changes"
                        } else {
                            "clean worktree"
                        }
                    ),
                    false,
                ),
                fact(
                    "Risk",
                    if i.is_dirty() {
                        "uncommitted changes in the worktree are lost"
                    } else {
                        "the container and its recovery state are removed"
                    },
                    false,
                ),
                fact("Reversible", "no", false),
            ],
            Some(short.to_owned()),
        )
    };
    let mut facts = facts;
    for fact in &mut facts {
        fact.tone = match fact.label {
            "Action" => FactTone::Error,
            "Risk" => FactTone::Warning,
            "Scope" | "Reversible" => FactTone::Secondary,
            _ => FactTone::Normal,
        };
    }
    Review {
        token,
        action,
        captured: Capture::Instance(captured),
        title,
        body,
        facts,
        acknowledgement,
    }
}

fn role_label<'a>(world: &'a World, key: &'a str) -> &'a str {
    world
        .roles
        .iter()
        .find(|entry| {
            key.strip_prefix(&entry.namespace)
                .and_then(|suffix| suffix.strip_prefix('/'))
                == Some(entry.name.as_str())
        })
        .map_or(key, |entry| entry.name.as_str())
}

fn inspect(i: &Instance, world: &World) -> Effect {
    let workspace = i.workspace.and_then(|id| world.workspace(id));
    let resolved = world.account_for(i.agent.provider(), workspace, Some(&i.role), None);
    let account = resolved.label(&world.accounts);
    let mut facts = vec![
        fact("Container", i.container_id(), true),
        fact(
            "Image",
            format!(
                "jackin/derived:{}-{}",
                i.workdir.trim_start_matches("/workspace/"),
                i.run_id.short().get(..4).unwrap_or_default()
            ),
            false,
        ),
        fact(
            "Workspace",
            format!(
                "{} › role {}",
                workspace.map_or("current directory", |w| w.name.as_str()),
                role_label(world, &i.role)
            ),
            false,
        ),
        fact(
            "Agent",
            format!("{} · account {account}", i.agent.label()),
            false,
        ),
        fact(
            "Target",
            format!(
                "{} · {}",
                i.workdir,
                plural(workspace.map_or(1, |w| w.mounts.len()), "mount", "mounts")
            ),
            false,
        ),
        fact("Run id", i.run_id.to_string(), true),
        fact("Lifecycle", i.status.label(), false).tone(if i.status.is_live() {
            FactTone::Normal
        } else {
            FactTone::Secondary
        }),
    ];
    facts.push(fact(
        "Daemon",
        match &i.daemon {
            DaemonSnapshot::Unavailable => "unavailable".into(),
            DaemonSnapshot::NoTabs => "attached · no tabs".into(),
            DaemonSnapshot::Tabs(tabs) => format!(
                "attached · {} · {}",
                plural(tabs.len(), "tab", "tabs"),
                world.clock.ago(i.last_seen_secs)
            ),
        },
        false,
    ));
    if matches!(i.daemon, DaemonSnapshot::Unavailable)
        && let Some(fact) = facts.last_mut()
    {
        fact.tone = FactTone::Warning;
    }
    Effect::Inspect {
        title: format!("Container {}", short(&i.id)),
        facts,
    }
}

/// Launch context captured before a picker opens; unrelated selection cannot replace it.
#[derive(Debug, Clone)]
pub struct LaunchTarget {
    workspace: Option<Workspace>,
    cwd: Option<String>,
    role: String,
}
impl LaunchTarget {
    /// Capture an explicitly selected saved workspace or unassociated directory.
    /// The caller resolves source default/last-role policy before this boundary.
    pub fn capture(world: &World, workspace: Option<WorkspaceId>, role: &str) -> Option<Self> {
        if !world.roles.iter().any(|entry| entry.full_name() == role) {
            return None;
        }
        let (workspace, cwd) = match workspace {
            Some(id) => (Some(world.workspace(id)?.clone()), None),
            None => (None, Some(world.cwd.clone())),
        };
        Some(Self {
            workspace,
            cwd,
            role: role.to_owned(),
        })
    }
    /// Stable saved workspace selected when the review began.
    pub fn workspace(&self) -> Option<WorkspaceId> {
        self.workspace.as_ref().map(|w| w.id)
    }
    /// Canonical catalogue identity, never a shortened display label.
    pub fn role(&self) -> &str {
        &self.role
    }
    /// Captured host workdir, including an unassociated current directory.
    pub fn workdir(&self) -> &str {
        self.workspace.as_ref().map_or_else(
            || self.cwd.as_deref().unwrap_or_default(),
            |w| w.workdir.as_str(),
        )
    }
    /// Reject removed/replaced configurations or catalogue roles before launch.
    pub fn valid(&self, world: &World) -> bool {
        world
            .roles
            .iter()
            .any(|entry| entry.full_name() == self.role)
            && match &self.workspace {
                Some(w) => world.workspace(w.id) == Some(w),
                None => self.cwd.as_deref() == Some(world.cwd.as_str()),
            }
    }
}

/// Why the selected context cannot resolve a discovered repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    /// No saved workspace is selected or associated with CurrentDir.
    NoWorkspace,
    /// The selected workspace has no matching repository metadata.
    NoRepository(String),
}
impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoWorkspace => f.write_str("Select a saved workspace to open its repository"),
            Self::NoRepository(name) => write!(f, "No GitHub source for {name}"),
        }
    }
}
impl std::error::Error for RepositoryError {}

/// Repository context captured from a saved workspace or current-directory association.
/// The immutable metadata is revalidated before emitting a simulated open.
#[derive(Debug, Clone)]
pub struct RepositoryTarget {
    workspace: Workspace,
    repository: GithubRepo,
    cwd: Option<String>,
}
impl RepositoryTarget {
    /// Capture the explicitly selected workspace; `None` means CurrentDir.
    /// Source lookup uses the first catalogue name ending in the workspace name.
    pub fn capture(world: &World, workspace: Option<WorkspaceId>) -> Result<Self, RepositoryError> {
        let cwd = workspace.is_none().then(|| world.cwd.clone());
        let workspace = match workspace {
            Some(id) => world.workspace(id),
            None => world.cwd_workspace(),
        }
        .ok_or(RepositoryError::NoWorkspace)?;
        let repository = world
            .github
            .iter()
            .find(|repo| repo.full_name.ends_with(&workspace.name))
            .ok_or_else(|| RepositoryError::NoRepository(workspace.name.clone()))?;
        Ok(Self {
            workspace: workspace.clone(),
            repository: repository.clone(),
            cwd,
        })
    }

    /// Read-only URL for a reviewed simulated destination.
    pub fn url(&self) -> &str {
        &self.repository.url
    }

    /// Revalidate saved configuration, CurrentDir association and exact catalogue entry.
    /// Returns source feedback only after resolving real fixture metadata.
    pub fn execute(self, world: &World) -> Effect {
        let workspace_valid = world.workspace(self.workspace.id) == Some(&self.workspace);
        let cwd_valid = self.cwd.as_ref().is_none_or(|cwd| {
            cwd == &world.cwd && world.cwd_workspace().map(|w| w.id) == Some(self.workspace.id)
        });
        let repository_valid = world
            .github
            .iter()
            .find(|repo| repo.full_name.ends_with(&self.workspace.name))
            == Some(&self.repository);
        if !workspace_valid || !cwd_valid || !repository_valid {
            return stale();
        }
        Effect::RepositoryOpened {
            status: format!("Opened {} on the host", self.repository.url),
            url: self.repository.url,
        }
    }
}
