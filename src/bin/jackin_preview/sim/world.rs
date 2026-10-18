



use std::collections::BTreeMap;

use crate::arbiter::Arbiter;
use crate::domain::account::{AccountId, AccountRegistry};
use crate::domain::agent::{Agent, AuthMode, Provider};
use crate::domain::instance::{Instance, InstanceId, InstanceStatus};
use crate::domain::workspace::{
    EnvVar, Mount, RoleEntry, RoleName, Usability, Workspace, WorkspaceId,
};
use crate::scenario::Scenario;
use crate::sim::onepassword::SimOnePassword;
use crate::sim::pty::Daemon;
use junie_tui::clock::SimClock;
use junie_tui::shell::JobKey;




pub mod job {
    use junie_tui::shell::JobKey;
    pub const WORKSPACE_SAVE: JobKey = JobKey::new("workspace.save");
    pub const SETTINGS_SAVE: JobKey = JobKey::new("settings.save");
    pub const ROLE_LOAD: JobKey = JobKey::new("role.load");
    pub const PREWARM: JobKey = JobKey::new("prewarm");
    pub const STOP: JobKey = JobKey::new("instance.stop");
    pub const PURGE: JobKey = JobKey::new("instance.purge");
    pub const ACCOUNT_VALIDATE: JobKey = JobKey::new("account.validate");
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalConfig {
    pub coauthor_trailer: bool,
    pub dco_signoff: bool,
    pub mounts: Vec<Mount>,
    pub env: Vec<EnvVar>,
    pub role_env: BTreeMap<RoleName, Vec<EnvVar>>,


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


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {

    WorkspaceSaved {
        id: WorkspaceId,
        ok: bool,
    },
    GlobalSaved {
        ok: bool,
    },

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

    Takeover {
        instance: InstanceId,
        by: String,
    },
}

impl Msg {

    pub fn job(&self) -> Option<JobKey> {
        match self {
            Msg::WorkspaceSaved { .. } => Some(job::WORKSPACE_SAVE),
            Msg::GlobalSaved { .. } => Some(job::SETTINGS_SAVE),
            Msg::RoleLoaded { .. } => Some(job::ROLE_LOAD),
            Msg::Prewarmed { .. } => Some(job::PREWARM),
            Msg::Stopped { .. } => Some(job::STOP),
            Msg::Purged { .. } => Some(job::PURGE),
            Msg::AccountValidated { .. } => Some(job::ACCOUNT_VALIDATE),
            Msg::Refreshed { .. } | Msg::AccountRefreshed { .. } | Msg::Takeover { .. } => None,
        }
    }
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
    pub clock: SimClock,
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

    pub refresh_fails: bool,

    pub save_fails_once: bool,

    pub takeover_at_ms: Option<i64>,

    pub session_accounts: BTreeMap<(InstanceId, u64), AccountId>,

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


    pub fn instances_of(&self, ws: Option<WorkspaceId>) -> Vec<&Instance> {
        self.instances
            .iter()
            .filter(|i| i.workspace == ws && !i.status.hidden())
            .collect()
    }


    pub fn sync_arbiter(&mut self) {
        if self.arbiter.discovery.is_ok() {
            let n = self.running_count();
            self.arbiter.set_running(n);
        }
    }


    pub fn mask_path(&self, path: &str) -> String {
        mask_path(&self.home, path)
    }


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


    pub fn agent_mode(&self, agent: Agent) -> AuthMode {
        self.global
            .agent_modes
            .get(&agent)
            .copied()
            .unwrap_or(AuthMode::Sync)
    }





    pub fn eligible_accounts(
        &self,
        agent: Agent,
        ws: Option<&Workspace>,
        role: Option<&str>,
    ) -> Vec<AccountId> {
        let offer = self.offer_for(agent, ws, role);
        offer.accounts
    }




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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentOffer {

    pub accounts: Vec<AccountId>,
    pub preselected: Option<AccountId>,

    pub blocked: Option<String>,

    pub configured: bool,
}

pub fn expand(home: &str, path: &str) -> String {
    if let Some(rest) = path.strip_prefix('~') {
        format!("{home}{rest}")
    } else {
        path.to_owned()
    }
}



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
