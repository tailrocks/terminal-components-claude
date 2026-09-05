//! Deterministic in-memory services for the preview.
//!
//! `World` owns virtual time, durable fixture data, live instance snapshots,
//! and a typed job queue.  It has no terminal or process access.

use std::collections::BTreeMap;

use crate::arbiter::Arbiter;
use crate::clock::{Clock, EPOCH_SECS};
use crate::domain::account::{AccountId, AccountRegistry};
use crate::domain::agent::{Agent, AuthMode, Provider};
use crate::domain::fixtures::{
    self, HOME, fixture_accounts, fixture_hard_accounts, fixture_instance, fixture_roles_for,
    fixture_workspaces_for,
};
use crate::domain::instance::{Instance, InstanceStatus};
use crate::domain::workspace::{RoleEntry, Usability, Workspace, WorkspaceId};
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

/// Typed results of deterministic asynchronous work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
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
    /// Mutable host configuration.
    pub global: GlobalConfig,
    /// Durable workspace rows.
    pub workspaces: Vec<Workspace>,
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
    /// Whether a workspace was saved during this run.
    pub saved: bool,
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
            configured: !ready.is_empty() || !blocked.is_none(),
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
    let populated = scenario != Scenario::FirstUse;
    let workspaces = if populated {
        fixture_workspaces_for(scenario)
    } else {
        Vec::new()
    };
    let accounts = if populated {
        if scenario == Scenario::HardCases {
            fixture_hard_accounts(&op, now)
        } else {
            fixture_accounts(&op, now)
        }
    } else {
        AccountRegistry::default()
    };
    let roles = fixture_roles_for(scenario);
    let mut instances = Vec::new();
    if populated && !matches!(scenario, Scenario::LaunchRunning | Scenario::LaunchFailure) {
        instances.push(fixture_instance(
            InstanceStatus::Running,
            crate::domain::instance::RunId::new(0x9c41_e2f0),
            now,
            fixtures::live_capsule(),
        ));
    }
    if matches!(scenario, Scenario::AccountsMixed | Scenario::HardCases) {
        instances.push(fixture_instance(
            InstanceStatus::Crashed,
            crate::domain::instance::RunId::new(0x0011_2233),
            now,
            crate::domain::instance::DaemonSnapshot::Unavailable,
        ));
    }
    if scenario == Scenario::CapsuleMulti {
        let mut secondary = fixture_instance(
            InstanceStatus::Running,
            crate::domain::instance::RunId::new(0x0a0b_0c0d),
            now,
            fixtures::live_capsule(),
        );
        secondary.id = "jk-ops".into();
        secondary.container = "jackin-ops-platform".into();
        secondary.role = "chainargos/reviewer".into();
        instances.push(secondary);
    }
    if scenario == Scenario::LaunchFailure {
        // A failed launch leaves an already-running session attachable.  The
        // manager must remain useful instead of presenting an empty shell.
        instances.push(fixture_instance(
            InstanceStatus::Running,
            crate::domain::instance::RunId::new(0x0e0f_1011),
            now,
            fixtures::live_capsule(),
        ));
    }
    if scenario == Scenario::HardCases {
        instances.push(fixture_instance(
            InstanceStatus::PreservedDirty,
            crate::domain::instance::RunId::new(0x0044_5566),
            now,
            crate::domain::instance::DaemonSnapshot::NoTabs,
        ));
        instances.push(fixture_instance(
            InstanceStatus::PreservedUnpushed,
            crate::domain::instance::RunId::new(0x0077_8899),
            now,
            crate::domain::instance::DaemonSnapshot::Unavailable,
        ));
    }
    let running = instances
        .iter()
        .filter(|instance| instance.status == InstanceStatus::Running)
        .count();
    let mut daemons = BTreeMap::new();
    for instance in &instances {
        if let crate::domain::instance::DaemonSnapshot::Tabs(_) = &instance.daemon {
            daemons.insert(
                instance.id.clone(),
                Daemon::from_snapshot(&instance.daemon, &instance.container, now),
            );
        }
    }
    World {
        scenario,
        clock,
        arbiter: Arbiter::new(running),
        home: HOME.into(),
        global: GlobalConfig {
            trust: vec![TrustRow {
                source: "chainargos/the-architect".into(),
                trusted: false,
            }],
        },
        workspaces,
        roles,
        instances,
        daemons,
        accounts,
        op,
        jobs: Vec::new(),
        refresh_fails: scenario == Scenario::HardCases,
        saved: false,
        last_refresh_secs: now,
        clipboard: None,
    }
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
                "customer-portal",
                "data-pipeline",
            ]
        );
        assert_eq!(returning.running_count(), 1);
        assert_eq!(returning.instances[0].run_id.value(), 0x9c41_e2f0);
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
