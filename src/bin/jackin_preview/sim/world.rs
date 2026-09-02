//! The fixture world: every piece of durable and live state the preview
//! simulates, plus a deterministic job queue that delivers typed messages
//! on virtual time.

use std::collections::BTreeMap;

use crate::arbiter::Arbiter;
use crate::clock::Clock;
use crate::domain::account::{AccountId, AccountRegistry};
use crate::domain::agent::{Agent, AuthMode, Provider};
use crate::domain::instance::{Instance, InstanceId, InstanceStatus};
use crate::domain::workspace::{
    EnvVar, Mount, RoleEntry, RoleName, Usability, Workspace, WorkspaceId,
};
use crate::scenario::Scenario;
use crate::sim::onepassword::SimOnePassword;
use crate::sim::pty::Daemon;

/// Global (host) configuration edited in Settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalConfig {
    pub coauthor_trailer: bool,
    pub dco_signoff: bool,
    pub mounts: Vec<Mount>,
    pub env: Vec<EnvVar>,
    pub role_env: BTreeMap<RoleName, Vec<EnvVar>>,
    /// How each agent runtime is given credentials inside the container.
    /// Which account backs it is the registry's business, not this table's.
    pub agent_modes: BTreeMap<Agent, AuthMode>,
    pub trust: Vec<TrustRow>,
}

impl GlobalConfig {
    pub fn change_count(&self, other: &GlobalConfig) -> usize {
        let mut n = 0;
        n += usize::from(self.coauthor_trailer != other.coauthor_trailer);
        n += usize::from(self.dco_signoff != other.dco_signoff);
        n += keyed_diff(&self.mounts, &other.mounts, |m| m.destination.clone());
        n += keyed_diff(&self.env, &other.env, |e| e.key.clone());
        n += usize::from(self.role_env != other.role_env);
        n += self
            .agent_modes
            .iter()
            .filter(|(a, m)| other.agent_modes.get(a) != Some(m))
            .count();
        n += keyed_diff(&self.trust, &other.trust, |t| t.source.clone());
        n
    }
}

/// Added + modified + removed rows, matching rows by identity so an edited
/// row counts once rather than as a removal plus an addition.
pub fn keyed_diff<T: PartialEq>(a: &[T], b: &[T], key: impl Fn(&T) -> String) -> usize {
    let mut n = 0;
    for x in a {
        match b.iter().find(|y| key(y) == key(x)) {
            None => n += 1,
            Some(y) if y != x => n += 1,
            _ => {}
        }
    }
    n += b
        .iter()
        .filter(|y| !a.iter().any(|x| key(x) == key(y)))
        .count();
    n
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustRow {
    pub source: String,
    pub kind: &'static str,
    pub trusted: bool,
    pub roles: usize,
}

/// A fixture filesystem entry for the file browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsEntry {
    pub path: String,
    pub dir: bool,
    pub git: Option<String>,
    pub meta: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubRepo {
    pub full_name: String,
    pub default_branch: String,
    pub branches: Vec<String>,
    pub updated: String,
    pub url: String,
}

/// Typed results of simulated asynchronous work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    /// Config write finished.
    WorkspaceSaved {
        id: WorkspaceId,
        ok: bool,
    },
    GlobalSaved {
        ok: bool,
    },
    /// Instance refresh cycle finished.
    Refreshed {
        ok: bool,
    },
    Prewarmed {
        workspace: WorkspaceId,
    },
    Stopped {
        instance: InstanceId,
    },
    Purged {
        instance: InstanceId,
    },
    AccountRefreshed {
        account: AccountId,
    },
    AccountValidated {
        account: AccountId,
    },
    RoleLoaded {
        role: String,
        ok: bool,
        error: Option<String>,
    },
    /// Another client attached to this instance and displaced us.
    Takeover {
        instance: InstanceId,
        by: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub due_ms: i64,
    pub msg: Msg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonHealth {
    Healthy,
    Stale,
}

pub struct World {
    pub scenario: Scenario,
    pub clock: Clock,
    pub arbiter: Arbiter,
    pub home: String,
    pub cwd: String,
    pub workspaces: Vec<Workspace>,
    pub next_workspace_id: WorkspaceId,
    pub roles: Vec<RoleEntry>,
    pub instances: Vec<Instance>,
    pub daemons: BTreeMap<InstanceId, Daemon>,
    pub global: GlobalConfig,
    pub accounts: AccountRegistry,
    pub op: SimOnePassword,
    pub fs: Vec<FsEntry>,
    pub github: Vec<GithubRepo>,
    pub clipboard: Option<String>,
    pub jobs: Vec<Job>,
    pub daemon_health: DaemonHealth,
    pub last_refresh_secs: i64,
    /// The next refresh cycle fails (hard cases).
    pub refresh_fails: bool,
    /// Saving fails once (hard cases).
    pub save_fails_once: bool,
    /// A foreign client takes over the attached instance after this many ms.
    pub takeover_at_ms: Option<i64>,
    /// Session-scope account choices made in the Capsule.
    pub session_accounts: BTreeMap<(InstanceId, u64), AccountId>,
    /// Discovered accounts appear only after a refresh in first use.
    pub discovery_pending: bool,
}

impl World {
    pub fn now_secs(&self) -> i64 {
        self.clock.now_secs()
    }

    pub fn now_ms(&self) -> i64 {
        self.clock.now_ms
    }

    pub fn schedule(&mut self, delay_ms: i64, msg: Msg) {
        let due = self.clock.now_ms + delay_ms;
        self.jobs.push(Job { due_ms: due, msg });
        self.jobs.sort_by_key(|j| j.due_ms);
    }

    /// Advance virtual time and pop due jobs. Daemons of running instances
    /// keep emitting output whether or not a client is attached.
    pub fn tick(&mut self, interval_ms: i64) -> Vec<Msg> {
        self.clock.advance(interval_ms);
        if !self.clock.running {
            return vec![];
        }
        let now = self.clock.now_ms;
        let mut due = vec![];
        self.jobs.retain(|j| {
            if j.due_ms <= now {
                due.push(j.msg.clone());
                false
            } else {
                true
            }
        });
        for (id, d) in self.daemons.iter_mut() {
            if self
                .instances
                .iter()
                .any(|i| i.id == *id && i.status == InstanceStatus::Running)
            {
                d.tick(now);
            }
        }
        if let Some(at) = self.takeover_at_ms
            && now >= at
        {
            self.takeover_at_ms = None;
            if let Some(i) = self
                .instances
                .iter()
                .find(|i| i.status == InstanceStatus::Running)
            {
                due.push(Msg::Takeover {
                    instance: i.id.clone(),
                    by: "tty004 · MacBook".into(),
                });
            }
        }
        due
    }

    pub fn workspace(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    pub fn workspace_mut(&mut self, id: WorkspaceId) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|w| w.id == id)
    }

    /// The saved Workspace whose workdir is the current directory.
    pub fn cwd_workspace(&self) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| {
            w.mounts.iter().any(|m| {
                m.source_label() == self.cwd || expand(&self.home, m.source_label()) == self.cwd
            })
        })
    }

    pub fn instance(&self, id: &str) -> Option<&Instance> {
        self.instances.iter().find(|i| i.id == id)
    }

    pub fn instance_mut(&mut self, id: &str) -> Option<&mut Instance> {
        self.instances.iter_mut().find(|i| i.id == id)
    }

    pub fn running_count(&self) -> usize {
        self.instances
            .iter()
            .filter(|i| i.status == InstanceStatus::Running)
            .count()
    }

    pub fn running(&self) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|i| i.status == InstanceStatus::Running)
            .collect()
    }

    /// Visible (non purged / superseded) instances of a Workspace.
    pub fn instances_of(&self, ws: Option<WorkspaceId>) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|i| i.workspace == ws && !i.status.hidden())
            .collect()
    }

    /// Sync the arbiter's discovery with the live instance count.
    pub fn sync_arbiter(&mut self) {
        if self.arbiter.discovery.is_ok() {
            let n = self.running_count();
            self.arbiter.set_running(n);
        }
    }

    /// Mask a private path: home → `~`, keep the last segment.
    pub fn mask_path(&self, path: &str) -> String {
        mask_path(&self.home, path)
    }

    /// Shorten with `~`.
    pub fn tilde(&self, path: &str) -> String {
        if let Some(rest) = path.strip_prefix(&self.home) {
            format!("~{rest}")
        } else {
            path.to_owned()
        }
    }

    pub fn new_instance_id(&self) -> String {
        let n = self.instances.len() as u32 + 0x7f3a + 0x1234 * 3;
        format!("jk-{:04x}", n & 0xffff)
    }

    pub fn account_for(
        &self,
        provider: Provider,
        ws: Option<&Workspace>,
        role: Option<&str>,
        session: Option<&AccountId>,
    ) -> crate::domain::fixtures::ResolvedAccount {
        crate::domain::fixtures::resolve_account(provider, ws, role, session, &self.accounts)
    }

    /// The agent runtime mode configured in Settings (`sync` when unset).
    pub fn agent_mode(&self, agent: Agent) -> AuthMode {
        self.global
            .agent_modes
            .get(&agent)
            .copied()
            .unwrap_or(AuthMode::Sync)
    }

    /// Accounts a session for `agent` may use: the Workspace's effective set
    /// (or, without a Workspace, the registry defaults and discovered
    /// logins) restricted to the agent's provider and to accounts that are
    /// ready right now. Preferred first.
    pub fn eligible_accounts(
        &self,
        agent: Agent,
        ws: Option<&Workspace>,
        role: Option<&str>,
    ) -> Vec<AccountId> {
        let offer = self.offer_for(agent, ws, role);
        offer.accounts
    }

    /// Everything the session picker needs for one agent. `None` when the
    /// agent has no configured account at all (it must not be offered);
    /// `blocked` when every configured account is temporarily unusable.
    pub fn offer_for(
        &self,
        agent: Agent,
        ws: Option<&Workspace>,
        role: Option<&str>,
    ) -> AgentOffer {
        let provider = agent.provider();
        let mut ready: Vec<AccountId> = vec![];
        let mut blocked: Vec<(AccountId, String)> = vec![];
        match ws {
            Some(w) => {
                for e in w.effective_accounts(&self.accounts) {
                    if e.provider != provider {
                        continue;
                    }
                    match &e.usable {
                        Usability::Ready => ready.push(e.id.clone()),
                        u => blocked.push((e.id.clone(), u.label())),
                    }
                }
            }
            None => {
                // no Workspace: the registry default, else the discovered login
                if let Some(a) = self.accounts.default_for(provider) {
                    match crate::domain::workspace::usability_of(a) {
                        Usability::Ready => ready.push(a.id.clone()),
                        u => blocked.push((a.id.clone(), u.label())),
                    }
                }
                if ready.is_empty()
                    && let Some(a) = self.accounts.discovered_current(provider)
                    && !ready.contains(&a.id)
                {
                    ready.push(a.id.clone());
                }
            }
        }
        if self.agent_mode(agent) == AuthMode::Ignore {
            return AgentOffer {
                accounts: vec![],
                preselected: None,
                blocked: Some("agent runtime mode is ignore · set in Settings › Agents".into()),
                configured: !ready.is_empty() || !blocked.is_empty(),
            };
        }
        // preselection follows the precedence resolver, restricted to the ready set
        let preselected = self
            .account_for(provider, ws, role, None)
            .account
            .filter(|id| ready.contains(id))
            .or_else(|| ready.first().cloned());
        if let Some(p) = &preselected
            && let Some(i) = ready.iter().position(|x| x == p)
            && i > 0
        {
            let x = ready.remove(i);
            ready.insert(0, x);
        }
        let block = if ready.is_empty() {
            blocked.first().map(|(id, why)| {
                let name = self
                    .accounts
                    .get(id)
                    .map(|a| a.title())
                    .unwrap_or(id.clone());
                format!("{name} · {why}")
            })
        } else {
            None
        };
        AgentOffer {
            configured: !ready.is_empty() || !blocked.is_empty(),
            accounts: ready,
            preselected,
            blocked: block,
        }
    }

    /// Agents a new session may run, in `Agent::ALL` order. Unconfigured
    /// agents are absent; agents whose only accounts are temporarily unusable
    /// come back with `blocked` set so the picker can show a disabled row.
    /// Shell is never in this list.
    pub fn offered_agents(
        &self,
        ws: Option<&Workspace>,
        role: Option<&str>,
    ) -> Vec<(Agent, AgentOffer)> {
        Agent::ALL
            .iter()
            .map(|a| (*a, self.offer_for(*a, ws, role)))
            .filter(|(_, o)| o.configured)
            .collect()
    }
}

/// What a session picker knows about one agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentOffer {
    /// Ready accounts, preselected one first.
    pub accounts: Vec<AccountId>,
    pub preselected: Option<AccountId>,
    /// Set when the agent is configured but cannot start right now.
    pub blocked: Option<String>,
    /// False when nothing at all is configured for the agent.
    pub configured: bool,
}

pub fn expand(home: &str, path: &str) -> String {
    if let Some(rest) = path.strip_prefix('~') {
        format!("{home}{rest}")
    } else {
        path.to_owned()
    }
}

/// `/Users/alexey/src/acme/api-gateway` → `~/…/api-gateway`; paths outside
/// home keep only the last segment prefixed by `…/`.
pub fn mask_path(home: &str, path: &str) -> String {
    let last = path.rsplit('/').next().unwrap_or(path);
    if let Some(rest) = path.strip_prefix(home) {
        let segs: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
        match segs.len() {
            0 => "~".into(),
            1 => format!("~/{}", segs[0]),
            _ => format!("~/…/{last}"),
        }
    } else if path.starts_with('/') {
        format!("…/{last}")
    } else {
        path.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_private_paths() {
        assert_eq!(
            mask_path("/Users/x", "/Users/x/src/acme/api-gateway"),
            "~/…/api-gateway"
        );
        assert_eq!(mask_path("/Users/x", "/Users/x/src"), "~/src");
        assert_eq!(mask_path("/Users/x", "/opt/build/cache"), "…/cache");
    }
}
