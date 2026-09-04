//! Deterministic in-memory services for the preview.
//!
//! `World` owns virtual time, durable fixture data, live instance snapshots,
//! and a typed job queue.  It has no terminal or process access.

use crate::arbiter::Arbiter;
use crate::clock::{Clock, EPOCH_SECS};
use crate::domain::account::{AccountId, AccountRegistry};
use crate::domain::agent::{Agent, AuthMode, Provider};
use crate::domain::fixtures::{
    self, HOME, fixture_accounts, fixture_instance, fixture_roles, fixture_workspace,
};
use crate::domain::instance::{Instance, InstanceStatus};
use crate::domain::workspace::{RoleEntry, Usability, Workspace, WorkspaceId};
use crate::scenario::Scenario;
use crate::sim::onepassword::SimOnePassword;

/// Typed results of deterministic asynchronous work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    WorkspaceSaved { id: WorkspaceId, ok: bool },
    Refreshed { ok: bool },
    AccountRefreshed { account: AccountId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub due_ms: i64,
    pub msg: Msg,
}

#[derive(Debug, Clone)]
pub struct World {
    pub scenario: Scenario,
    pub clock: Clock,
    pub arbiter: Arbiter,
    pub home: String,
    pub workspaces: Vec<Workspace>,
    pub roles: Vec<RoleEntry>,
    pub instances: Vec<Instance>,
    pub accounts: AccountRegistry,
    pub op: SimOnePassword,
    pub jobs: Vec<Job>,
    pub refresh_fails: bool,
    pub saved: bool,
    pub last_refresh_secs: i64,
}

impl World {
    pub fn now_secs(&self) -> i64 {
        self.clock.now_secs()
    }

    pub fn now_ms(&self) -> i64 {
        self.clock.now_ms
    }

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
        ready
    }

    pub fn workspace(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.iter().find(|workspace| workspace.id == id)
    }

    pub fn workspace_mut(&mut self, id: WorkspaceId) -> Option<&mut Workspace> {
        self.workspaces
            .iter_mut()
            .find(|workspace| workspace.id == id)
    }

    pub fn instance(&self, id: &str) -> Option<&Instance> {
        self.instances.iter().find(|instance| instance.id == id)
    }

    pub fn instance_mut(&mut self, id: &str) -> Option<&mut Instance> {
        self.instances.iter_mut().find(|instance| instance.id == id)
    }

    pub fn running_count(&self) -> usize {
        self.instances
            .iter()
            .filter(|instance| instance.status == InstanceStatus::Running)
            .count()
    }

    pub fn running(&self) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|instance| instance.status == InstanceStatus::Running)
            .collect()
    }

    pub fn instances_of(&self, workspace: Option<WorkspaceId>) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|instance| {
                instance.workspace == workspace && !instance.status.hidden()
            })
            .collect()
    }

    pub fn sync_arbiter(&mut self) {
        self.arbiter.set_running(self.running_count());
    }

    pub fn new_instance_id(&self) -> String {
        let next = self.instances.len().saturating_add(1);
        format!("jk-{next:04x}")
    }

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

    pub fn eligible_accounts(
        &self,
        agent: Agent,
        workspace: Option<&Workspace>,
        role: Option<&str>,
    ) -> Vec<AccountId> {
        self.offer_for(agent, workspace, role).accounts
    }

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
    let workspaces = populated.then(|| fixture_workspace()).into_iter().collect();
    let accounts = if populated {
        fixture_accounts(&op, now)
    } else {
        AccountRegistry::default()
    };
    let roles = fixture_roles();
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
    World {
        scenario,
        clock,
        arbiter: Arbiter::new(running),
        home: HOME.into(),
        workspaces,
        roles,
        instances,
        accounts,
        op,
        jobs: Vec::new(),
        refresh_fails: scenario == Scenario::HardCases,
        saved: false,
        last_refresh_secs: now,
    }
}

/// What a new session knows about one agent's account choices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentOffer {
    pub configured: bool,
    pub accounts: Vec<AccountId>,
    pub preselected: Option<AccountId>,
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
        assert_eq!(returning.workspaces.len(), 1);
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
