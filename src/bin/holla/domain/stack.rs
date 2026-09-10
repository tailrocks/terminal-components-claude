//! Fixture state for the priority stack: mise, Git and GitHub, Docker, the
//! system snapshot, disk and cleanup candidates, PostgreSQL, SSH, Debian
//! packages, and the user's ranking memory. Plain data; the catalogue turns
//! it into rows and the simulation advances it.

// ------------------------------------------------------------------ git

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitState {
    pub path: String,
    /// `None` while detached.
    pub branch: Option<String>,
    pub head_short: String,
    pub primary: String,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
    pub conflicts: u32,
    pub stash: u32,
    pub remotes: Vec<String>,
    pub submodule: bool,
    /// Worktree of another checkout: deduplicated in bulk plans.
    pub worktree_of: Option<String>,
    /// `rebase`, `merge`, `cherry-pick` in progress.
    pub in_progress: Option<String>,
    pub modified: Vec<String>,
    pub diverged: bool,
}

impl GitState {
    pub fn clean(path: &str, branch: &str, primary: &str) -> Self {
        Self {
            path: path.into(),
            branch: Some(branch.into()),
            head_short: "a1b2c3d".into(),
            primary: primary.into(),
            upstream: Some(format!("origin/{branch}")),
            ahead: 0,
            behind: 0,
            staged: 0,
            unstaged: 0,
            untracked: 0,
            conflicts: 0,
            stash: 0,
            remotes: vec!["origin".into()],
            submodule: false,
            worktree_of: None,
            in_progress: None,
            modified: vec![],
            diverged: false,
        }
    }

    pub fn dirty(&self) -> bool {
        self.staged + self.unstaged + self.untracked + self.conflicts > 0
    }

    pub fn changed(&self) -> u32 {
        self.staged + self.unstaged
    }

    /// Why an automatic pull or switch is blocked, if it is.
    pub fn block_reason(&self) -> Option<String> {
        if self.conflicts > 0 {
            return Some(format!("{} conflicting files", self.conflicts));
        }
        if let Some(op) = &self.in_progress {
            return Some(format!("{op} in progress"));
        }
        if self.branch.is_none() {
            return Some("detached HEAD".into());
        }
        if self.diverged {
            return Some(format!(
                "diverged · {} ahead, {} behind",
                self.ahead, self.behind
            ));
        }
        if self.upstream.is_none() {
            return Some("no upstream".into());
        }
        if self.dirty() {
            return Some(format!("{} modified", self.changed() + self.untracked));
        }
        None
    }

    /// `main ↓3 • 4` for the status bar.
    pub fn summary(&self) -> String {
        let head = self.head_label();
        let state = self.state_summary();
        if state.is_empty() {
            head
        } else {
            format!("{head} {state}")
        }
    }

    /// The branch name or the detached head.
    pub fn head_label(&self) -> String {
        match &self.branch {
            Some(b) => b.clone(),
            None => format!("detached @ {}", self.head_short),
        }
    }

    /// The state after the branch name: `↓3 ↑1 • 5 · rebase`, empty when
    /// clean and in sync. This is the part that earns the warning tone.
    pub fn state_summary(&self) -> String {
        let mut s = String::new();
        if self.behind > 0 {
            s.push_str(&format!(" ↓{}", self.behind));
        }
        if self.ahead > 0 {
            s.push_str(&format!(" ↑{}", self.ahead));
        }
        if self.dirty() {
            s.push_str(&format!(" • {}", self.changed() + self.untracked));
        }
        if let Some(op) = &self.in_progress {
            s.push_str(&format!(" · {op}"));
        }
        s.trim_start().to_owned()
    }

    /// The porcelain status line Holla would run.
    pub fn status_cmd(&self) -> String {
        format!(
            "git -C {} status --porcelain=v2 --branch --show-stash",
            self.path
        )
    }
}

// ------------------------------------------------------------------ github

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubRepo {
    pub owner: String,
    pub name: String,
    pub private: bool,
    pub primary: String,
    pub fork_of: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubState {
    pub logged_in: bool,
    pub account: String,
    pub host: String,
    pub orgs: Vec<String>,
    pub repos: Vec<GithubRepo>,
    pub protocol: String,
}

// ------------------------------------------------------------------ docker

#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub running: bool,
    pub health: Option<String>,
    pub ports: String,
    pub project: Option<String>,
    pub cpu_pct: f32,
    pub mem_mb: u32,
    pub size_mb: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Volume {
    pub name: String,
    pub gb: f32,
    pub anonymous: bool,
    pub used_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compose {
    pub file: String,
    pub dir: String,
    pub name: String,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DockerState {
    /// `Err(reason)` when the daemon cannot be reached.
    pub daemon: Result<(), String>,
    pub containers: Vec<Container>,
    pub images: u32,
    pub images_gb: f32,
    pub dangling_images: u32,
    pub volumes: Vec<Volume>,
    pub networks: u32,
    pub builder_cache_gb: f32,
    pub reclaimable_gb: f32,
    pub compose: Option<Compose>,
    /// Times the host-wide cleanup ran on this host.
    pub cleanup_uses: u32,
}

impl DockerState {
    pub fn unavailable(reason: &str) -> Self {
        Self {
            daemon: Err(reason.into()),
            containers: vec![],
            images: 0,
            images_gb: 0.0,
            dangling_images: 0,
            volumes: vec![],
            networks: 0,
            builder_cache_gb: 0.0,
            reclaimable_gb: 0.0,
            compose: None,
            cleanup_uses: 0,
        }
    }
    pub fn running(&self) -> usize {
        self.containers.iter().filter(|c| c.running).count()
    }
    pub fn unhealthy(&self) -> Vec<&Container> {
        self.containers
            .iter()
            .filter(|c| c.health.as_deref() == Some("unhealthy"))
            .collect()
    }
    pub fn named_volumes(&self) -> usize {
        self.volumes.iter().filter(|v| !v.anonymous).count()
    }
    pub fn volumes_gb(&self) -> f32 {
        self.volumes.iter().map(|v| v.gb).sum()
    }
}

// ------------------------------------------------------------------ mise

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiseTask {
    pub name: String,
    /// `//apps/frontend:dev` from the monorepo root; the plain name inside
    /// the child.
    pub namespaced: String,
    pub dir: String,
    pub run: String,
    pub depends: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiseTool {
    pub name: String,
    pub requested: String,
    pub active: Option<String>,
    pub latest: String,
    pub installed: bool,
    pub global: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiseConfig {
    pub path: String,
    pub trusted: bool,
    pub tasks: Vec<MiseTask>,
    pub tools: Vec<MiseTool>,
    pub env: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiseState {
    pub installed: bool,
    pub version: String,
    /// Effective configuration chain, nearest first.
    pub configs: Vec<MiseConfig>,
    pub monorepo_root: Option<String>,
    pub global_tools: Vec<MiseTool>,
}

impl MiseState {
    pub fn outdated(&self) -> Vec<&MiseTool> {
        self.configs
            .iter()
            .flat_map(|c| c.tools.iter())
            .chain(self.global_tools.iter())
            .filter(|t| t.active.as_deref() != Some(t.latest.as_str()) && t.installed)
            .collect()
    }
    pub fn missing(&self) -> Vec<&MiseTool> {
        self.configs
            .iter()
            .flat_map(|c| c.tools.iter())
            .filter(|t| !t.installed)
            .collect()
    }
}

// ------------------------------------------------------------------ system

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proc {
    pub pid: u32,
    pub name: String,
    pub cpu_pct: u32,
    pub mem_mb: u32,
    pub state: String,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemSnapshot {
    pub cpu_pct: u32,
    pub cores: Vec<u32>,
    pub load: String,
    pub mem_used_gb: f32,
    pub mem_total_gb: f32,
    pub swap_used_gb: f32,
    /// Linux pressure stall (some/full) percentages, when available.
    pub pressure: Option<(u32, u32)>,
    /// Minutes of sustained pressure, drives the recommendation.
    pub pressure_minutes: u32,
    pub disk_io: Vec<(String, String)>,
    pub net: Vec<(String, String)>,
    pub top: Vec<Proc>,
    pub btm_installed: bool,
    pub temp_c: Option<u32>,
}

// ------------------------------------------------------------------ disk

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    ProjectArtifacts,
    DeveloperCaches,
    ApplicationCaches,
    Containers,
    Logs,
    Temp,
    PackageCaches,
    LargeFiles,
}

impl Family {
    pub fn label(self) -> &'static str {
        match self {
            Family::ProjectArtifacts => "project artifacts",
            Family::DeveloperCaches => "developer-tool caches",
            Family::ApplicationCaches => "application caches",
            Family::Containers => "virtualization and containers",
            Family::Logs => "logs",
            Family::Temp => "temporary data",
            Family::PackageCaches => "package caches",
            Family::LargeFiles => "large files",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Trash,
    Permanent,
    /// Cleaned through the owning tool (`cargo clean`, `pnpm store prune`).
    Tool(String),
}

impl Method {
    pub fn label(&self) -> String {
        match self {
            Method::Trash => "move to Trash · recoverable".into(),
            Method::Permanent => "permanent".into(),
            Method::Tool(cmd) => format!("through the tool · {cmd}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Unverifiable,
}

impl Confidence {
    pub fn label(self) -> &'static str {
        match self {
            Confidence::High => "high",
            Confidence::Medium => "medium",
            Confidence::Unverifiable => "unverifiable",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub path: String,
    pub family: Family,
    pub project: Option<String>,
    pub gb: f32,
    pub items: u32,
    /// `None` when activity cannot be verified.
    pub inactive_days: Option<u32>,
    pub why: String,
    pub regenerate: String,
    pub active_process: Option<String>,
    pub privilege: Option<String>,
    pub method: Method,
    pub confidence: Confidence,
    /// Shares an output directory or cache with this other candidate: the
    /// two are serialised.
    pub shares_with: Option<String>,
    /// Tick at which the scan reveals it.
    pub found_at: u64,
    pub protected: bool,
}

impl Candidate {
    /// Freshness-aware default: only old, confident, unprotected candidates
    /// start selected.
    pub fn default_selected(&self) -> bool {
        !self.protected
            && self.active_process.is_none()
            && self.confidence == Confidence::High
            && self.inactive_days.is_some_and(|d| d >= 14)
    }

    pub fn age_label(&self) -> String {
        match self.inactive_days {
            Some(0) => "active today".into(),
            Some(1) => "inactive 1 day".into(),
            Some(d) => format!("inactive {d} days"),
            None => "activity unknown".into(),
        }
    }

    pub fn skip_reason(&self) -> Option<String> {
        if self.protected {
            return Some("protected path".into());
        }
        if let Some(p) = &self.active_process {
            return Some(format!("in use by {p}"));
        }
        if self.confidence == Confidence::Unverifiable {
            return Some("activity unverifiable".into());
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Filesystem {
    pub mount: String,
    pub total_gb: u32,
    pub used_gb: u32,
}

impl Filesystem {
    pub fn pct(&self) -> u8 {
        ((self.used_gb as f32 / self.total_gb.max(1) as f32) * 100.0).round() as u8
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CleanupRecord {
    pub when_secs: i64,
    pub target: String,
    pub method: String,
    pub reclaimed_gb: f32,
    pub outcome: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LargeEntry {
    pub path: String,
    pub gb: f32,
    pub kind: &'static str,
    pub found_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskState {
    pub filesystems: Vec<Filesystem>,
    pub large: Vec<LargeEntry>,
    pub candidates: Vec<Candidate>,
    pub protected: Vec<String>,
    pub history: Vec<CleanupRecord>,
    /// Total ticks for a full scan of the working directory.
    pub scan_ticks: u64,
    /// Discovery is partial because some paths were unreadable.
    pub partial_reason: Option<String>,
}

impl DiskState {
    pub fn root_fs(&self) -> Option<&Filesystem> {
        self.filesystems.first()
    }
    pub fn reclaimable_gb(&self) -> f32 {
        self.candidates
            .iter()
            .filter(|c| c.skip_reason().is_none())
            .map(|c| c.gb)
            .sum()
    }
}

// ------------------------------------------------------------------ postgres

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgSession {
    pub pid: u32,
    pub user: String,
    pub app: String,
    pub db: String,
    pub client: String,
    pub state: String,
    pub wait: Option<String>,
    pub query_secs: u32,
    pub txn_secs: u32,
    pub query: String,
    pub blocked_by: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgState {
    pub label: String,
    pub source: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub db: String,
    pub max_connections: u32,
    pub connections: u32,
    pub sessions: Vec<PgSession>,
    pub pg_activity_installed: bool,
    pub deadlocks_24h: u32,
    pub cache_hit_pct: u32,
    pub replication_lag: Option<String>,
}

impl PgState {
    pub fn blocked(&self) -> Vec<&PgSession> {
        self.sessions
            .iter()
            .filter(|s| s.blocked_by.is_some())
            .collect()
    }
    pub fn idle_in_txn(&self) -> usize {
        self.sessions
            .iter()
            .filter(|s| s.state == "idle in transaction")
            .count()
    }
    pub fn blockers(&self) -> Vec<&PgSession> {
        let ids: Vec<u32> = self.sessions.iter().filter_map(|s| s.blocked_by).collect();
        self.sessions
            .iter()
            .filter(|s| ids.contains(&s.pid) && s.blocked_by.is_none())
            .collect()
    }
}

// ------------------------------------------------------------------ ssh

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshAlias {
    pub alias: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub jump: Option<String>,
    pub identities: Vec<String>,
    pub forwards: Vec<String>,
    pub hostkey: String,
    pub multiplexed: bool,
    pub from_file: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshState {
    pub config: String,
    pub includes: Vec<String>,
    pub aliases: Vec<SshAlias>,
    pub wildcard_rules: u32,
}

// ------------------------------------------------------------------ debian

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AptPackage {
    pub name: String,
    pub from: String,
    pub to: String,
    pub security: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AptState {
    pub upgradable: Vec<AptPackage>,
    pub pending_reboot: bool,
    pub lock_held_by: Option<String>,
    pub free_gb: f32,
    pub sudo_cached: bool,
}

// ------------------------------------------------------------------ memory

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRecord {
    pub item: String,
    /// `None` counts as global use.
    pub path: Option<String>,
    pub host: String,
    pub count: u32,
    pub last_secs: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RankingMemory {
    /// (path, item id)
    pub pins: Vec<(String, String)>,
    /// (alias, item id)
    pub aliases: Vec<(String, String)>,
    /// (path, item id)
    pub hidden: Vec<(String, String)>,
    pub usage: Vec<UsageRecord>,
    pub personalization: bool,
    /// (item id, tool)
    pub preferred_tools: Vec<(String, String)>,
}

impl RankingMemory {
    pub fn used(&mut self, item: &str, path: Option<&str>, host: &str, now_secs: i64) {
        if let Some(u) = self
            .usage
            .iter_mut()
            .find(|u| u.item == item && u.path.as_deref() == path && u.host == host)
        {
            u.count += 1;
            u.last_secs = now_secs;
        } else {
            self.usage.push(UsageRecord {
                item: item.into(),
                path: path.map(str::to_owned),
                host: host.into(),
                count: 1,
                last_secs: now_secs,
            });
        }
    }

    pub fn alias_for(&self, item: &str) -> Option<&str> {
        self.aliases
            .iter()
            .find(|(_, i)| i == item)
            .map(|(a, _)| a.as_str())
    }

    pub fn set_alias(&mut self, alias: &str, item: &str) {
        self.aliases.retain(|(a, i)| a != alias && i != item);
        self.aliases.push((alias.into(), item.into()));
    }

    pub fn toggle_pin(&mut self, path: &str, item: &str) -> bool {
        if let Some(i) = self.pins.iter().position(|(p, it)| p == path && it == item) {
            self.pins.remove(i);
            false
        } else {
            self.pins.push((path.into(), item.into()));
            true
        }
    }

    pub fn hide(&mut self, path: &str, item: &str) {
        if !self.hidden.iter().any(|(p, it)| p == path && it == item) {
            self.hidden.push((path.into(), item.into()));
        }
    }

    pub fn restore(&mut self, path: &str, item: &str) -> bool {
        let before = self.hidden.len();
        self.hidden.retain(|(p, it)| !(p == path && it == item));
        before != self.hidden.len()
    }

    /// Forget every learned signal for `item`, keeping pins and aliases.
    pub fn reset(&mut self, item: &str) {
        self.usage.retain(|u| u.item != item);
        self.hidden.retain(|(_, it)| it != item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_block_reasons_follow_the_policy_order() {
        let mut g = GitState::clean("/w/api", "main", "main");
        assert_eq!(g.block_reason(), None);
        g.unstaged = 2;
        g.untracked = 1;
        assert_eq!(g.block_reason().as_deref(), Some("3 modified"));
        assert_eq!(g.summary(), "main • 3");
        g.behind = 3;
        assert_eq!(g.summary(), "main ↓3 • 3");
        g.diverged = true;
        g.ahead = 1;
        assert!(g.block_reason().unwrap().starts_with("diverged"));
        g.branch = None;
        assert_eq!(g.block_reason().as_deref(), Some("detached HEAD"));
        g.in_progress = Some("rebase".into());
        assert_eq!(g.block_reason().as_deref(), Some("rebase in progress"));
    }

    #[test]
    fn candidates_default_to_selected_only_when_stale_and_confident() {
        let mut c = Candidate {
            path: "/w/frontend/node_modules".into(),
            family: Family::ProjectArtifacts,
            project: Some("frontend".into()),
            gb: 8.4,
            items: 120_000,
            inactive_days: Some(31),
            why: "".into(),
            regenerate: "pnpm install --frozen-lockfile".into(),
            active_process: None,
            privilege: None,
            method: Method::Trash,
            confidence: Confidence::High,
            shares_with: None,
            found_at: 0,
            protected: false,
        };
        assert!(c.default_selected());
        c.inactive_days = Some(0);
        assert!(!c.default_selected());
        c.inactive_days = None;
        c.confidence = Confidence::Unverifiable;
        assert!(!c.default_selected());
        assert_eq!(c.skip_reason().as_deref(), Some("activity unverifiable"));
        assert_eq!(c.age_label(), "activity unknown");
    }

    #[test]
    fn memory_counts_use_per_path_and_aliases_replace() {
        let mut m = RankingMemory::default();
        m.used("git.pull", Some("/w"), "mbp", 10);
        m.used("git.pull", Some("/w"), "mbp", 20);
        m.used("git.pull", None, "mbp", 30);
        assert_eq!(m.usage.len(), 2);
        assert_eq!(m.usage[0].count, 2);
        m.set_alias("gp", "git.pull");
        m.set_alias("gp", "git.push");
        assert_eq!(m.alias_for("git.push"), Some("gp"));
        assert_eq!(m.alias_for("git.pull"), None);
        assert!(m.toggle_pin("/w", "git.pull"));
        assert!(!m.toggle_pin("/w", "git.pull"));
        m.hide("/w", "x");
        assert!(m.restore("/w", "x"));
        assert!(!m.restore("/w", "x"));
    }
}
