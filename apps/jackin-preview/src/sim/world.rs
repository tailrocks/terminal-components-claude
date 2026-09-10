//! Deterministic in-memory services for the preview.
//!
//! `World` owns virtual time, durable fixture data, live instance snapshots,
//! and a typed job queue.  It has no terminal or process access.

use std::collections::BTreeMap;

use crate::arbiter::Arbiter;
use crate::clock::{Clock, EPOCH_SECS};
use crate::domain::account::{AccountId, AccountRegistry};
use crate::domain::agent::{Agent, AuthMode, Provider};
use crate::domain::fixtures::{self, HOME};
use crate::domain::instance::{Instance, InstanceStatus};
use crate::domain::workspace::{RoleEntry, Usability, Workspace, WorkspaceId};
use crate::domain::workspace_save::{PendingWrite, SaveError, SaveResult, SaveTicket};
use crate::scenario::Scenario;
use crate::sim::onepassword::SimOnePassword;
use crate::sim::pty::Daemon;

/// Host trust setting projected by the Settings route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustRow {
    /// Stable source label.
    pub source: String,
    /// Whether the source is trusted.
    pub trusted: bool,
}

/// Host-level configuration shared by workspace drafts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GlobalConfig {
    /// Trust rows edited by Settings.
    pub trust: Vec<TrustRow>,
}

/// Public repository metadata from the deterministic host discovery fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubRepo {
    /// Namespace and repository name.
    pub full_name: String,
    /// Default checkout branch.
    pub default_branch: String,
    /// Available branches, default first.
    pub branches: Vec<String>,
    /// Source-authored freshness display.
    pub updated: String,
    /// Public repository URL; simulation never launches a host process.
    pub url: String,
}

/// Last observed discovery health, separate from injected refresh failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonHealth {
    /// Discovery has not observed a failed refresh.
    Healthy,
    /// A refresh failed; last-good records remain visible.
    Stale,
}

/// Typed results of deterministic asynchronous work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    /// A captured editor write reached its virtual deadline.
    EditorSaveCompleted {
        /// Opaque World-owned write identity.
        operation: u64,
    },
    /// One captured Manager operation reached its virtual deadline.
    ManagerOperation {
        /// Opaque identity owned and consumed by the Manager reducer.
        operation: u64,
    },
    /// A workspace save completed.
    WorkspaceSaved {
        /// Workspace identifier that was saved.
        id: WorkspaceId,
        /// Whether the save succeeded.
        ok: bool,
    },
    /// A refresh operation completed.
    Refreshed {
        /// Whether the refresh succeeded.
        ok: bool,
    },
    /// One account refresh completed.
    AccountRefreshed {
        /// Account identifier that was refreshed.
        account: AccountId,
    },
}

/// One delayed message in the virtual job queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// Absolute virtual due time in milliseconds.
    pub due_ms: i64,
    /// Message delivered when the deadline is reached.
    pub msg: Msg,
}

/// Complete deterministic service state for a preview scenario.
#[derive(Debug, Clone)]
pub struct World {
    /// Scenario that seeded this world.
    pub scenario: Scenario,
    /// Virtual fixture clock.
    pub clock: Clock,
    /// Running-instance arbiter state.
    pub arbiter: Arbiter,
    /// Fixture home directory.
    pub home: String,
    /// Actual current host directory; it can be unsaved.
    pub cwd: String,
    /// Mutable host configuration.
    pub global: GlobalConfig,
    /// Durable workspace rows.
    pub workspaces: Vec<Workspace>,
    /// Discovered public repository metadata.
    pub github: Vec<GithubRepo>,
    /// Available role entries.
    pub roles: Vec<RoleEntry>,
    /// Persisted instance rows.
    pub instances: Vec<Instance>,
    /// Live daemon models keyed by persisted instance id.
    pub daemons: BTreeMap<String, Daemon>,
    /// Account registry.
    pub accounts: AccountRegistry,
    /// Simulated 1Password service.
    pub op: SimOnePassword,
    /// Delayed asynchronous jobs.
    pub jobs: Vec<Job>,
    /// Whether the next refresh should fail.
    pub refresh_fails: bool,
    /// Last observed refresh health.
    pub daemon_health: DaemonHealth,
    manager_operation_sequence: u64,
    manager_review_watermark: u64,
    /// Whether a workspace was saved during this run.
    pub saved: bool,
    /// Source allocator for newly saved configurations.
    pub next_workspace_id: WorkspaceId,
    /// Inject one simulated write failure, consumed only by an admitted save.
    pub save_fails_once: bool,
    editor_save_sequence: u64,
    editor_writes: Vec<PendingWrite>,
    /// Last successful refresh time in fixture seconds.
    pub last_refresh_secs: i64,
    /// Last copied transcript selection, if any.
    pub clipboard: Option<String>,
}

impl World {
    /// Current fixture time in seconds.
    pub fn now_secs(&self) -> i64 {
        self.clock.now_secs()
    }

    /// Current fixture time in milliseconds.
    pub fn now_ms(&self) -> i64 {
        self.clock.now_ms
    }

    /// Allocate a world-lifetime operation token, independent of screen replacement.
    pub(crate) fn next_manager_operation(&mut self) -> Option<u64> {
        let next = self.manager_operation_sequence.checked_add(1)?;
        self.manager_operation_sequence = next;
        Some(next)
    }

    /// Consume a review once, invalidating older UI contexts without shared clone state.
    pub(crate) fn consume_manager_review(&mut self, review: u64) -> bool {
        if review <= self.manager_review_watermark || review > self.manager_operation_sequence {
            return false;
        }
        self.manager_review_watermark = review;
        true
    }

    /// Admit a captured write; UI callbacks never supply its completion payload.
    pub(crate) fn begin_editor_write(
        &mut self,
        expected: Option<&Workspace>,
        mut proposed: Workspace,
    ) -> Result<SaveTicket, SaveError> {
        if let Some(original) = expected {
            if self.workspace(original.id) != Some(original) {
                return Err(SaveError::TargetChanged);
            }
            if self
                .editor_writes
                .iter()
                .any(|write| write.ticket.workspace == original.id)
            {
                return Err(SaveError::Busy);
            }
        }
        let operation = self
            .editor_save_sequence
            .checked_add(1)
            .ok_or(SaveError::IdentityExhausted)?;
        let id = match expected {
            Some(original) => original.id,
            None => {
                let mut id = self.next_workspace_id;
                while self.workspace(id).is_some()
                    || self
                        .editor_writes
                        .iter()
                        .any(|write| write.ticket.workspace == id)
                {
                    id = id.checked_add(1).ok_or(SaveError::IdentityExhausted)?;
                }
                // Keep a representable next source identifier after success.
                id.checked_add(1).ok_or(SaveError::IdentityExhausted)?;
                id
            }
        };
        proposed.id = id;
        let ticket = SaveTicket {
            operation,
            workspace: id,
        };
        let ok = !self.save_fails_once;
        self.save_fails_once = false;
        self.editor_save_sequence = operation;
        self.editor_writes.push(PendingWrite {
            ticket,
            due_ms: self.now_ms().saturating_add(900),
            expected: expected.cloned().map(Box::new),
            proposed: Box::new(proposed),
            ok,
        });
        self.schedule(900, Msg::EditorSaveCompleted { operation });
        Ok(ticket)
    }

    /// Validate and consume exactly one due write, independently of the active screen.
    pub fn complete_editor_save(&mut self, operation: u64) -> SaveResult {
        let Some(index) = self
            .editor_writes
            .iter()
            .position(|write| write.ticket.operation == operation)
        else {
            return SaveResult::Ignored;
        };
        if self
            .editor_writes
            .get(index)
            .is_some_and(|write| self.now_ms() < write.due_ms)
        {
            return SaveResult::Ignored;
        }
        let write = self.editor_writes.remove(index);
        let current = self.workspace(write.ticket.workspace);
        if current != write.expected.as_deref() {
            return SaveResult::Stale(write.ticket);
        }
        if !write.ok {
            return SaveResult::Failed(write.ticket);
        }
        match write.expected {
            Some(_) => {
                let Some(workspace) = self.workspace_mut(write.ticket.workspace) else {
                    return SaveResult::Stale(write.ticket);
                };
                *workspace = *write.proposed.clone();
            }
            None => {
                self.next_workspace_id = self
                    .next_workspace_id
                    .max(write.ticket.workspace.saturating_add(1));
                self.workspaces.push(*write.proposed.clone());
            }
        }
        self.saved = true;
        SaveResult::Saved {
            ticket: write.ticket,
            workspace: write.proposed,
        }
    }

    /// Queue a message after a non-negative virtual delay.
    pub fn schedule(&mut self, delay_ms: i64, msg: Msg) {
        let due_ms = self.clock.now_ms.saturating_add(delay_ms.max(0));
        self.jobs.push(Job { due_ms, msg });
        self.jobs.sort_by_key(|job| job.due_ms);
    }

    /// Advance virtual time and return jobs whose deadline has passed.
    pub fn tick(&mut self, interval_ms: i64) -> Vec<Msg> {
        self.clock.advance(interval_ms.max(0));
        if !self.clock.running {
            return Vec::new();
        }
        let now = self.clock.now_ms;
        let mut ready = Vec::new();
        self.jobs.retain(|job| {
            if job.due_ms <= now {
                ready.push(job.msg.clone());
                false
            } else {
                true
            }
        });
        for daemon in self.daemons.values_mut() {
            daemon.tick(now);
        }
        for instance in &mut self.instances {
            if let Some(daemon) = self.daemons.get(&instance.id) {
                instance.daemon = daemon.snapshot();
            }
        }
        ready
    }

    /// Find a workspace by stable identifier.
    pub fn workspace(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.iter().find(|workspace| workspace.id == id)
    }

    /// Find a mutable workspace by stable identifier.
    pub fn workspace_mut(&mut self, id: WorkspaceId) -> Option<&mut Workspace> {
        self.workspaces
            .iter_mut()
            .find(|workspace| workspace.id == id)
    }

    /// Saved workspace whose mount source exactly matches the current directory.
    ///
    /// Uses the pinned source's literal or home-expanded mount spelling. No
    /// prefix/descendant matching or fallback to the first workspace occurs.
    /// Returns `None` for an unsaved directory. Reads current domain records,
    /// so rename, removal and mount edits cannot leave a cached association.
    pub fn cwd_workspace(&self) -> Option<&Workspace> {
        self.workspaces.iter().find(|workspace| {
            workspace.mounts.iter().any(|mount| {
                let source = mount.source_label();
                source == self.cwd
                    || source
                        .strip_prefix('~')
                        .is_some_and(|rest| self.cwd.strip_prefix(self.home.as_str()) == Some(rest))
            })
        })
    }

    /// Find an instance by stable identifier.
    pub fn instance(&self, id: &str) -> Option<&Instance> {
        self.instances.iter().find(|instance| instance.id == id)
    }

    /// Find a mutable instance by stable identifier.
    pub fn instance_mut(&mut self, id: &str) -> Option<&mut Instance> {
        self.instances.iter_mut().find(|instance| instance.id == id)
    }

    /// Count instances currently in the running state.
    pub fn running_count(&self) -> usize {
        self.instances
            .iter()
            .filter(|instance| instance.status == InstanceStatus::Running)
            .count()
    }

    /// Return instances currently in the running state.
    pub fn running(&self) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|instance| instance.status == InstanceStatus::Running)
            .collect()
    }

    /// Return visible instances belonging to a workspace.
    pub fn instances_of(&self, workspace: Option<WorkspaceId>) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|instance| instance.workspace == workspace && !instance.status.hidden())
            .collect()
    }

    /// Synchronize the arbiter with the current running-instance count.
    pub fn sync_arbiter(&mut self) {
        self.arbiter.set_running(self.running_count());
    }

    /// Allocate the next deterministic instance identifier.
    pub fn new_instance_id(&self) -> String {
        let next = self.instances.len().saturating_add(1);
        format!("jk-{next:04x}")
    }

    /// Resolve the account selected for a launch context.
    pub fn account_for(
        &self,
        provider: Provider,
        workspace: Option<&Workspace>,
        role: Option<&str>,
        session: Option<&AccountId>,
    ) -> fixtures::ResolvedAccount {
        fixtures::resolve_account(provider, workspace, role, session, &self.accounts)
    }

    /// The preview has one deterministic default mode for every agent.
    pub fn agent_mode(&self, _agent: Agent) -> AuthMode {
        AuthMode::Sync
    }

    /// Return usable account identifiers for an agent and context.
    pub fn eligible_accounts(
        &self,
        agent: Agent,
        workspace: Option<&Workspace>,
        role: Option<&str>,
    ) -> Vec<AccountId> {
        self.offer_for(agent, workspace, role).accounts
    }

    /// Build the account offer shown for one agent.
    pub fn offer_for(
        &self,
        agent: Agent,
        workspace: Option<&Workspace>,
        role: Option<&str>,
    ) -> AgentOffer {
        let provider = agent.provider();
        let mut ready = Vec::new();
        let mut blocked = Vec::new();
        if let Some(workspace) = workspace {
            for account in workspace.effective_accounts(&self.accounts) {
                if account.provider != provider {
                    continue;
                }
                match account.usable {
                    Usability::Ready => ready.push(account.id),
                    status => blocked.push((account.id, status.label())),
                }
            }
        } else {
            if let Some(account) = self.accounts.default_for(provider) {
                match crate::domain::workspace::usability_of(account) {
                    Usability::Ready => ready.push(account.id.clone()),
                    status => blocked.push((account.id.clone(), status.label())),
                }
            }
            if ready.is_empty()
                && let Some(account) = self.accounts.discovered_current(provider)
            {
                ready.push(account.id.clone());
            }
        }
        let selected = self
            .account_for(provider, workspace, role, None)
            .account
            .filter(|id| ready.contains(id))
            .or_else(|| ready.first().cloned());
        if let Some(selected) = &selected
            && let Some(index) = ready.iter().position(|id| id == selected)
        {
            let account = ready.remove(index);
            ready.insert(0, account);
        }
        let blocked = if ready.is_empty() {
            blocked.first().map(|(id, reason)| {
                let title = self
                    .accounts
                    .get(id)
                    .map_or_else(|| id.clone(), |account| account.title());
                format!("{title} · {reason}")
            })
        } else {
            None
        };
        AgentOffer {
            configured: !ready.is_empty() || blocked.is_some(),
            accounts: ready,
            preselected: selected,
            blocked,
        }
    }

    /// Return agents with at least one configured or blocked account.
    pub fn offered_agents(
        &self,
        workspace: Option<&Workspace>,
        role: Option<&str>,
    ) -> Vec<(Agent, AgentOffer)> {
        Agent::ALL
            .into_iter()
            .map(|agent| (agent, self.offer_for(agent, workspace, role)))
            .filter(|(_, offer)| offer.configured)
            .collect()
    }
}

/// Build the complete deterministic world for one preview scenario.
pub fn world_for(scenario: Scenario) -> World {
    let clock = Clock::new();
    let now = EPOCH_SECS;
    let op = SimOnePassword::fixture(now);
    let mut world = World {
        scenario,
        clock,
        arbiter: Arbiter::new(0),
        home: HOME.into(),
        cwd: fixtures::PAYMENTS_WORKDIR.into(),
        global: GlobalConfig {
            trust: vec![
                TrustRow {
                    source: "github.com/chainargos/roles".into(),
                    trusted: true,
                },
                TrustRow {
                    source: "github.com/acme-labs/roles-experimental".into(),
                    trusted: false,
                },
                TrustRow {
                    source: "~/roles".into(),
                    trusted: true,
                },
                TrustRow {
                    source: "git@corp:infra/roles".into(),
                    trusted: true,
                },
            ],
        },
        workspaces: Vec::new(),
        roles: fixtures::fixture_roles_for(scenario),
        instances: Vec::new(),
        daemons: BTreeMap::new(),
        accounts: AccountRegistry::default(),
        op,
        jobs: Vec::new(),
        refresh_fails: scenario == Scenario::HardCases,
        daemon_health: DaemonHealth::Healthy,
        manager_operation_sequence: 0,
        manager_review_watermark: 0,
        github: fixtures::pinned::github(),
        saved: false,
        next_workspace_id: 100,
        save_fails_once: scenario == Scenario::HardCases,
        editor_save_sequence: 0,
        editor_writes: Vec::new(),
        last_refresh_secs: now - 3,
        clipboard: None,
    };
    if scenario != Scenario::FirstUse {
        fixtures::pinned::populate(&mut world, scenario == Scenario::HardCases);
    }
    if scenario == Scenario::OutroLast {
        if let Some(instance) = world.instance_mut("jk-9b02") {
            instance.status = InstanceStatus::CleanExited;
        }
        world.daemons.remove("jk-9b02");
        fixtures::pinned::refresh_snapshots(&mut world);
        world.sync_arbiter();
        world.arbiter.entered_at_ms = Some(-8_040_000);
    }
    if scenario == Scenario::HardCases {
        world.op.session = crate::sim::onepassword::OpSession::Locked;
        world.arbiter.discovery = Err(crate::arbiter::DiscoveryError::IndexUnreadable);
        world.arbiter.entered_at_ms = None;
    }
    world
}

/// What a new session knows about one agent's account choices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentOffer {
    /// Whether the agent has a configured or blocked account.
    pub configured: bool,
    /// Usable account identifiers, preselected first when applicable.
    pub accounts: Vec<AccountId>,
    /// Account selected by workspace/role precedence, if usable.
    pub preselected: Option<AccountId>,
    /// Human-readable blocking reason when no account is usable.
    pub blocked: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenarios_seed_stable_world_shapes() {
        let first = world_for(Scenario::FirstUse);
        assert!(first.workspaces.is_empty());
        assert!(first.instances.is_empty());
        assert_eq!(first.running_count(), 0);

        let returning = world_for(Scenario::Returning);
        // Returning starts from the populated registry, not the single
        // launch fixture. Keep every durable row in the assertion so a
        // scenario change cannot silently drop manager coverage.
        assert_eq!(returning.workspaces.len(), 4);
        assert_eq!(
            returning
                .workspaces
                .iter()
                .map(|workspace| workspace.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "payments-platform",
                "infra-control-plane",
                "release-automation",
                "customer-portal",
            ]
        );
        assert_eq!(returning.running_count(), 2);
        assert_eq!(
            returning.instances[0].run_id,
            crate::domain::instance::RunId::from_label("run-7f3a")
        );
    }

    #[test]
    fn jobs_follow_virtual_time_and_pause() {
        let mut world = world_for(Scenario::Returning);
        world.schedule(100, Msg::Refreshed { ok: true });
        assert!(world.tick(99).is_empty());
        assert_eq!(world.tick(1), vec![Msg::Refreshed { ok: true }]);
        world.clock.running = false;
        world.schedule(1, Msg::Refreshed { ok: false });
        assert!(world.tick(10).is_empty());
    }
}
