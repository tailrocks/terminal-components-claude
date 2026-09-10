//! The fixture world and its tick: discovery sources completing, activities
//! emitting scripted output, plans scheduling ready steps within their
//! parallelism and exclusive-resource rules, and the disk scan streaming
//! candidates. Every change is a pure function of the tick count.

use std::collections::BTreeMap;

use crate::clock::Clock;
use crate::domain::action::Item;
use crate::domain::activity::{Activity, ActivityKind, ActivityState, Script};
use crate::domain::cleanup::{OpsLog, SizeCache};
use crate::domain::context::{Host, Location, ScopeTag};
use crate::domain::custom::{CustomConfig, TrustStore};
use crate::domain::effect::Effect;
use crate::domain::exec::Command;
use crate::domain::plan::{Plan, PlanPhase, StepState};
use crate::domain::stack::{
    AptState, BrewState, CargoState, CleanupRecord, Container, DiskState, DockerState, GitState,
    GithubState, GradleState, MiseState, PgState, Platform, RankingMemory, SshState,
    SystemSnapshot, ToolOutputs, UpgradeState,
};
use crate::scenario::Scenario;
use crate::sim::fs::Fs;
use std::collections::BTreeSet;

/// Virtual milliseconds per tick.
pub const TICK_MS: i64 = 80;

/// A discovery source: mise, git, docker, … each completes at a fixture
/// tick or fails with a reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub name: String,
    pub done_at: u64,
    pub failure: Option<String>,
}

/// How the members of a batch are scheduled and judged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchMode {
    /// One after another; a failure does not short-circuit later members.
    Sequential,
    /// All at once; aggregate success needs every member to succeed.
    Parallel,
}

/// Several activities started by one action (a batch pull, a mirror push).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub id: String,
    pub label: String,
    pub mode: BatchMode,
    pub members: Vec<String>,
    pub summarised: bool,
}

impl Batch {
    /// `(succeeded, failed, cancelled, pending)`.
    pub fn tally(&self, w: &World) -> (usize, usize, usize, usize) {
        let mut t = (0, 0, 0, 0);
        for m in &self.members {
            match w.activity(m).map(|a| a.state) {
                Some(ActivityState::Succeeded) => t.0 += 1,
                Some(ActivityState::Failed) => t.1 += 1,
                Some(ActivityState::Stopped) => t.2 += 1,
                _ => t.3 += 1,
            }
        }
        t
    }
    pub fn done(&self, w: &World) -> bool {
        self.tally(w).3 == 0
    }
    pub fn ok(&self, w: &World) -> bool {
        let (ok, failed, cancelled, pending) = self.tally(w);
        pending == 0 && failed == 0 && cancelled == 0 && ok == self.members.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    ActivityEnded {
        id: String,
        ok: bool,
    },
    /// Every member of a batch has settled.
    BatchDone {
        id: String,
        ok: bool,
    },
    PlanStepEnded {
        plan: String,
        step: usize,
        ok: bool,
    },
    PlanDone {
        plan: String,
    },
    DiscoveryDone {
        source: String,
    },
    ScanDone,
}

pub struct World {
    pub scenario: Scenario,
    pub clock: Clock,
    pub tick: u64,
    pub host: Host,
    pub location: Location,
    pub git: Vec<GitState>,
    pub github: GithubState,
    pub docker: DockerState,
    pub mise: MiseState,
    pub system: SystemSnapshot,
    pub disk: DiskState,
    pub pg: Option<PgState>,
    pub ssh: SshState,
    pub apt: Option<AptState>,
    pub memory: RankingMemory,
    pub activities: Vec<Activity>,
    pub batches: Vec<Batch>,
    pub plans: Vec<Plan>,
    pub sources: Vec<Source>,
    /// Activity scripts by key (`task:frontend-dev`).
    pub scripts: BTreeMap<String, Script>,
    /// Retained per-step output by `plan/step` id.
    pub step_output: BTreeMap<String, Vec<String>>,
    pub clipboard: Option<String>,
    /// Tick the disk scan started, if it did.
    pub scan_started: Option<u64>,
    pub next_activity: u32,
    /// Config paths trusted during this session.
    pub trusted_now: Vec<String>,
    /// systemd units restarted by holla this session: their degraded state
    /// is healed in the catalogue and the snapshot.
    pub healed_units: Vec<String>,
    /// The virtual filesystem every file, disk and cleanup flow reads.
    pub fs: Fs,
    /// Executables on PATH.
    pub tools: BTreeSet<String>,
    pub cargo: CargoState,
    pub brew: Option<BrewState>,
    pub gradle: GradleState,
    pub upgrade: UpgradeState,
    pub platform: Platform,
    pub outputs: ToolOutputs,
    /// Global and project custom-action configurations and the trust store.
    pub custom_global: Option<CustomConfig>,
    pub custom_project: Option<CustomConfig>,
    pub trust: TrustStore,
    /// `sudo -n true` succeeds: no password prompt.
    pub sudo_cached: bool,
    /// systemd units whose restart fails.
    pub service_failures: Vec<String>,
    /// Native task names that exit nonzero.
    pub task_failures: Vec<String>,
    pub ops_log: OpsLog,
    pub size_cache: SizeCache,
    /// What a restart would find on disk: the persisted texts.
    pub persisted: Persisted,
    /// Cleanup reports produced this session, newest last.
    pub reports: Vec<crate::domain::cleanup::Report>,
    /// The live disk scan, when one is running or finished.
    pub scan: Option<ScanState>,
    /// Directory listings that answer late: (path, ticks).
    pub fs_latency: Vec<(String, u64)>,
}

/// A progressive directory scan over the virtual filesystem (HP18): the
/// tree is computed once and revealed in path order over `scan_ticks`, so
/// partial frames are reproducible; a rescan bumps the generation so stale
/// results can never replace a newer one.
/// One batch member: name, argv, script, effect, scope.
pub type BatchMember = (String, Vec<Command>, Script, Option<Effect>, ScopeTag);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanState {
    pub root: String,
    pub started: u64,
    pub generation: u64,
    pub ticks: u64,
    pub tree: crate::sim::fs::ScanNode,
    /// Every path in reveal order.
    pub order: Vec<String>,
    /// Pre-scan mtimes for the cache freshness snapshot.
    pub snapshot: Vec<(String, i64)>,
    pub cancelled: bool,
    pub include_hidden: bool,
    /// Bounded producer queue: events delivered per tick.
    pub batch: usize,
    pub revealed_at_cancel: usize,
}

impl ScanState {
    /// Paths revealed by `tick`.
    pub fn revealed(&self, tick: u64) -> usize {
        if self.cancelled {
            return self.revealed_at_cancel.min(self.order.len());
        }
        let elapsed = tick.saturating_sub(self.started);
        let per_tick = self
            .order
            .len()
            .div_ceil(self.ticks.max(1) as usize)
            .max(1)
            .min(self.batch);
        (elapsed as usize * per_tick).min(self.order.len())
    }
    pub fn done(&self, tick: u64) -> bool {
        self.cancelled || self.revealed(tick) >= self.order.len()
    }
    pub fn progress(&self, tick: u64) -> f64 {
        if self.order.is_empty() {
            return 1.0;
        }
        self.revealed(tick) as f64 / self.order.len() as f64
    }
    pub fn is_revealed(&self, path: &str, tick: u64) -> bool {
        let n = self.revealed(tick);
        self.order.iter().take(n).any(|p| p == path)
    }
}

/// Persisted stores as the next launch would read them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Persisted {
    pub trust: Option<String>,
    pub frecency: Option<String>,
    pub sizes: Option<String>,
    pub brew_cache: Option<String>,
    pub ops_log: Vec<String>,
}

impl World {
    pub fn now_ms(&self) -> i64 {
        self.clock.now_ms
    }

    pub fn now_secs(&self) -> i64 {
        self.clock.now_secs()
    }

    pub fn source_state(&self, name: &str) -> SourceState {
        match self.sources.iter().find(|s| s.name == name) {
            None => SourceState::Done,
            Some(s) => {
                if let Some(f) = &s.failure {
                    if self.tick >= s.done_at {
                        SourceState::Failed(f.clone())
                    } else {
                        SourceState::Loading
                    }
                } else if self.tick >= s.done_at {
                    SourceState::Done
                } else {
                    SourceState::Loading
                }
            }
        }
    }

    pub fn sources_done(&self) -> usize {
        self.sources
            .iter()
            .filter(|s| self.tick >= s.done_at)
            .count()
    }

    pub fn discovering(&self) -> bool {
        self.sources_done() < self.sources.len()
    }

    /// Names of the sources still loading, in order.
    pub fn pending_sources(&self) -> Vec<&str> {
        self.sources
            .iter()
            .filter(|s| self.tick < s.done_at)
            .map(|s| s.name.as_str())
            .collect()
    }

    pub fn failed_sources(&self) -> Vec<(&str, &str)> {
        self.sources
            .iter()
            .filter(|s| self.tick >= s.done_at)
            .filter_map(|s| s.failure.as_deref().map(|f| (s.name.as_str(), f)))
            .collect()
    }

    pub fn git_here(&self) -> Option<&GitState> {
        let root = self
            .location
            .project
            .as_ref()
            .map(|p| p.root.as_str())
            .unwrap_or(&self.location.cwd);
        self.git.iter().find(|g| g.path == root).or_else(|| {
            self.git
                .iter()
                .find(|g| self.location.cwd.starts_with(&g.path))
        })
    }

    pub fn activity(&self, id: &str) -> Option<&Activity> {
        self.activities.iter().find(|a| a.id == id)
    }

    pub fn activity_mut(&mut self, id: &str) -> Option<&mut Activity> {
        self.activities.iter_mut().find(|a| a.id == id)
    }

    pub fn plan(&self, id: &str) -> Option<&Plan> {
        self.plans.iter().find(|p| p.id == id)
    }

    pub fn plan_mut(&mut self, id: &str) -> Option<&mut Plan> {
        self.plans.iter_mut().find(|p| p.id == id)
    }

    pub fn live_activities(&self) -> usize {
        self.activities.iter().filter(|a| a.state.live()).count()
    }

    /// The simulated script for a named key, else for the exact argv, else
    /// the truthful unmodeled failure. Nothing generically succeeds.
    pub fn script_for(&self, key: Option<&str>, argv: &[Command]) -> Script {
        if let Some(k) = key
            && let Some(s) = self.scripts.get(k)
        {
            return s.clone();
        }
        if let Some(s) = crate::domain::outcomes::for_commands(self, argv) {
            return s;
        }
        let display = argv
            .iter()
            .map(|c| c.display())
            .collect::<Vec<_>>()
            .join(" && ");
        Script::unmodeled(&display)
    }

    /// Start an activity from an explicit specification: the exact argv,
    /// its simulated script and its effect. Returns its id.
    pub fn start_activity(
        &mut self,
        argv: Vec<Command>,
        script: Script,
        name: &str,
        origin: &str,
        kind: ActivityKind,
        scope: ScopeTag,
    ) -> String {
        self.next_activity += 1;
        let id = format!("act-{}", self.next_activity);
        let host = self.host.name.clone();
        let mut a = Activity::new(&id, name, origin, kind, scope, &host, script, self.tick);
        a.argv = argv;
        a.attachable = matches!(
            a.kind,
            ActivityKind::Monitor { .. } | ActivityKind::Ssh { .. }
        );
        a.advance(self.tick);
        self.activities.push(a);
        id
    }

    /// Start an activity by script key (fixtures and handoffs).
    pub fn start_scripted(
        &mut self,
        script_key: &str,
        name: &str,
        origin: &str,
        kind: ActivityKind,
        scope: ScopeTag,
    ) -> String {
        let script = self.script_for(Some(script_key), &[]);
        self.start_activity(vec![], script, name, origin, kind, scope)
    }

    /// Start several activities as one batch. Sequential members queue
    /// behind their predecessor; parallel members all start now.
    #[allow(clippy::too_many_arguments)]
    pub fn start_batch(
        &mut self,
        id: &str,
        label: &str,
        mode: BatchMode,
        members: Vec<BatchMember>,
        origin: &str,
    ) -> Vec<String> {
        let mut ids = vec![];
        let mut prev: Option<String> = None;
        for (name, argv, script, effect, scope) in members {
            self.next_activity += 1;
            let aid = format!("act-{}", self.next_activity);
            let host = self.host.name.clone();
            let mut a = Activity::new(
                &aid,
                &name,
                origin,
                ActivityKind::Task,
                scope,
                &host,
                script,
                self.tick,
            );
            a.argv = argv;
            a.effect = effect;
            a.batch = Some(id.to_owned());
            if mode == BatchMode::Sequential && prev.is_some() {
                a.state = ActivityState::Queued;
                a.after = prev.clone();
            } else {
                a.advance(self.tick);
            }
            prev = Some(aid.clone());
            ids.push(aid);
            self.activities.push(a);
        }
        self.batches.retain(|b| b.id != id);
        self.batches.push(Batch {
            id: id.into(),
            label: label.into(),
            mode,
            members: ids.clone(),
            summarised: false,
        });
        ids
    }

    /// Stop every live activity (the quit path). Returns how many were
    /// asked to stop.
    pub fn stop_all(&mut self) -> usize {
        let tick = self.tick;
        let mut n = 0;
        for a in &mut self.activities {
            if a.state.live() {
                a.stop(tick);
                n += 1;
            }
        }
        n
    }

    /// Activities still being torn down.
    pub fn cancelling(&self) -> usize {
        self.activities
            .iter()
            .filter(|a| a.state == ActivityState::Cancelling)
            .count()
    }

    /// Record a use in memory (path-scoped and host-scoped).
    pub fn record_use(&mut self, item: &str) {
        let cwd = self.location.cwd.clone();
        let host = self.host.name.clone();
        let now = self.now_secs();
        self.memory.used(item, Some(&cwd), &host, now);
        self.memory.used(item, None, &host, now);
    }

    /// Advance one tick. Returns the messages produced.
    pub fn tick(&mut self) -> Vec<Msg> {
        self.clock.advance(TICK_MS);
        if !self.clock.running {
            return vec![];
        }
        self.tick += 1;
        let mut msgs = vec![];
        for s in &self.sources {
            if s.done_at == self.tick {
                msgs.push(Msg::DiscoveryDone {
                    source: s.name.clone(),
                });
            }
        }
        if let Some(start) = self.scan_started
            && self.tick == start + self.disk.scan_ticks
        {
            msgs.push(Msg::ScanDone);
            self.finish_scan_cache();
        }
        let tick = self.tick;
        let mut effects = vec![];
        // queued members start once their predecessor has settled, whatever
        // the result: a failure never short-circuits later batch members
        let ready: Vec<String> = self
            .activities
            .iter()
            .filter(|a| a.state == ActivityState::Queued)
            .filter(|a| {
                a.after
                    .as_ref()
                    .is_none_or(|p| self.activity(p).is_none_or(|x| x.state.finished()))
            })
            .map(|a| a.id.clone())
            .collect();
        for id in ready {
            if let Some(a) = self.activity_mut(&id) {
                a.start(tick);
            }
        }
        for a in &mut self.activities {
            let was = a.state;
            a.advance(tick);
            if was.live() && !a.state.live() {
                let ok = a.state == ActivityState::Succeeded;
                if ok && let Some(e) = &a.effect {
                    effects.push(e.clone());
                }
                msgs.push(Msg::ActivityEnded {
                    id: a.id.clone(),
                    ok,
                });
            }
        }
        for e in &effects {
            self.apply_effect(e);
        }
        for bi in 0..self.batches.len() {
            if !self.batches[bi].summarised && self.batches[bi].done(self) {
                let ok = self.batches[bi].ok(self);
                self.batches[bi].summarised = true;
                msgs.push(Msg::BatchDone {
                    id: self.batches[bi].id.clone(),
                    ok,
                });
            }
        }
        for pi in 0..self.plans.len() {
            msgs.extend(self.tick_plan(pi));
        }
        msgs
    }

    fn tick_plan(&mut self, pi: usize) -> Vec<Msg> {
        let tick = self.tick;
        let mut msgs = vec![];
        let plan_id = self.plans[pi].id.clone();
        if self.plans[pi].phase != PlanPhase::Running {
            return msgs;
        }
        // finish running steps whose duration elapsed; stream their output
        for si in 0..self.plans[pi].steps.len() {
            let (running, started, duration, fails) = {
                let s = &self.plans[pi].steps[si];
                (
                    s.state == StepState::Running,
                    s.started_tick.unwrap_or(tick),
                    s.duration_ticks.max(1),
                    s.fails,
                )
            };
            if !running {
                continue;
            }
            let key = format!("{plan_id}/{}", self.plans[pi].steps[si].id);
            let lines = self.step_output.get(&key).cloned().unwrap_or_default();
            let elapsed = tick.saturating_sub(started);
            let want = ((elapsed as f64 / duration as f64) * lines.len() as f64).ceil() as usize;
            let want = want.min(lines.len());
            let have = self.plans[pi].steps[si].output.len();
            if want > have {
                self.plans[pi].steps[si]
                    .output
                    .extend(lines[have..want].iter().cloned());
            }
            if elapsed >= duration {
                let s = &mut self.plans[pi].steps[si];
                s.ended_tick = Some(tick);
                s.exit = Some(if fails { 1 } else { 0 });
                s.state = if fails {
                    StepState::Failed
                } else {
                    StepState::Succeeded
                };
                if fails {
                    s.output.push("exit status 1".into());
                }
                msgs.push(Msg::PlanStepEnded {
                    plan: plan_id.clone(),
                    step: si,
                    ok: !fails,
                });
            }
        }
        self.plans[pi].recompute();
        // start ready steps within the parallel budget and exclusive locks
        loop {
            let plan = &self.plans[pi];
            let running = plan.running();
            if running.len() >= plan.max_parallel {
                break;
            }
            let locks: Vec<String> = running
                .iter()
                .filter_map(|&i| plan.steps[i].exclusive.clone())
                .collect();
            let next = plan.ready().into_iter().find(|&i| {
                plan.steps[i]
                    .exclusive
                    .as_ref()
                    .is_none_or(|e| !locks.contains(e))
            });
            match next {
                Some(i) => {
                    let s = &mut self.plans[pi].steps[i];
                    s.state = StepState::Running;
                    s.started_tick = Some(tick);
                }
                None => break,
            }
        }
        if self.plans[pi].is_done() && self.plans[pi].phase == PlanPhase::Running {
            self.plans[pi].phase = PlanPhase::Done;
            self.plans[pi].ended_tick = Some(tick);
            self.apply_plan_effects(pi);
            msgs.push(Msg::PlanDone { plan: plan_id });
        }
        msgs
    }

    /// The PID blocking other sessions, when one is.
    pub fn pg_blocker(&self) -> Option<u32> {
        self.pg
            .as_ref()
            .and_then(|pg| pg.sessions.iter().find_map(|s| s.blocked_by))
    }

    /// Land a finished activity's effect in the world.
    pub fn apply_effect(&mut self, e: &Effect) {
        match e {
            Effect::DockerRestart(name) => {
                if let Some(c) = self.docker.containers.iter_mut().find(|c| &c.name == name) {
                    c.running = true;
                    if c.health.is_some() {
                        c.health = Some("healthy".into());
                    }
                }
            }
            Effect::DockerStop(name) => {
                if let Some(c) = self.docker.containers.iter_mut().find(|c| &c.name == name) {
                    c.running = false;
                    c.cpu_pct = 0.0;
                }
            }
            Effect::ServiceRestart(unit) => {
                if !self.healed_units.contains(unit) {
                    self.healed_units.push(unit.clone());
                }
            }
            Effect::PgCancel(pid) | Effect::PgTerminate(pid) => {
                if let Some(pg) = &mut self.pg {
                    let terminate = matches!(e, Effect::PgTerminate(_));
                    if terminate {
                        pg.sessions.retain(|s| s.pid != *pid);
                        pg.connections = pg.connections.saturating_sub(1);
                    } else if let Some(s) = pg.sessions.iter_mut().find(|s| s.pid == *pid) {
                        s.state = "idle".into();
                        s.txn_secs = 0;
                        s.query_secs = 0;
                        s.query = "ROLLBACK".into();
                    }
                    for s in pg
                        .sessions
                        .iter_mut()
                        .filter(|s| s.blocked_by == Some(*pid))
                    {
                        s.blocked_by = None;
                        s.wait = None;
                        s.state = "active".into();
                    }
                }
            }
            Effect::GitPull(path) => {
                if let Some(g) = self.git.iter_mut().find(|g| &g.path == path) {
                    g.behind = 0;
                }
            }
            Effect::CargoClean(path) => {
                let target = format!("{path}/target");
                let _ = self.fs.remove_permanent(&target);
                if self.cargo.target.as_deref() == Some(target.as_str()) {
                    self.cargo.target_files = 0;
                    self.cargo.target_bytes = 0;
                }
                self.remove_candidate(&target, "cargo clean");
            }
            Effect::DockerStopAll => {
                for c in &mut self.docker.containers {
                    c.running = false;
                    c.cpu_pct = 0.0;
                }
            }
            Effect::DockerRemoveAll => {
                self.docker.containers.clear();
                for v in &mut self.docker.volumes {
                    v.used_by = None;
                }
            }
            Effect::DockerImagesPrune => {
                self.docker.images = 0;
                self.docker.images_gb = 0.0;
                self.docker.dangling_images = 0;
                self.docker.reclaimable_gb =
                    self.docker.volumes_gb() + self.docker.builder_cache_gb;
            }
            Effect::DockerNetworkPrune => {
                self.docker.networks = 0;
            }
            Effect::DockerVolumePrune => {
                // only volumes no container references are removed
                self.docker.volumes.retain(|v| v.used_by.is_some());
                self.docker.reclaimable_gb =
                    self.docker.images_gb + self.docker.volumes_gb() + self.docker.builder_cache_gb;
            }
            Effect::DockerBuilderPrune => {
                self.docker.builder_cache_gb = 0.0;
                self.docker.reclaimable_gb = self.docker.images_gb + self.docker.volumes_gb();
            }
            Effect::ComposeUp => {
                if let Some(c) = self.docker.compose.clone() {
                    for svc in &c.services {
                        let name = format!("{}-{svc}-1", c.name);
                        match self.docker.containers.iter_mut().find(|x| {
                            x.name == name
                                || (x.project.as_deref() == Some(c.name.as_str())
                                    && x.image.starts_with(svc.as_str()))
                        }) {
                            Some(x) => {
                                x.running = true;
                                if x.health.is_some() {
                                    x.health = Some("healthy".into());
                                }
                            }
                            None => self.docker.containers.push(Container {
                                name,
                                image: format!("{svc}:latest"),
                                running: true,
                                health: None,
                                ports: String::new(),
                                project: Some(c.name.clone()),
                                cpu_pct: 0.1,
                                mem_mb: 32,
                                size_mb: 0,
                            }),
                        }
                    }
                }
            }
            Effect::ComposeDown => {
                if let Some(c) = self.docker.compose.clone() {
                    self.docker
                        .containers
                        .retain(|x| x.project.as_deref() != Some(c.name.as_str()));
                    self.docker.networks = self.docker.networks.saturating_sub(1);
                }
            }
            Effect::GitPullMerge(path) => {
                if let Some(g) = self.git.iter_mut().find(|g| &g.path == path) {
                    if g.behind > 0 && g.ahead > 0 && g.pull_config != "rebase" {
                        // a merge commit lands on top of the local commits
                        g.ahead += 1;
                    }
                    g.behind = 0;
                    g.diverged = false;
                }
            }
            Effect::GitPush(path) => {
                if let Some(g) = self.git.iter_mut().find(|g| &g.path == path) {
                    g.ahead = 0;
                    g.push_rejected = None;
                    if g.upstream.is_none()
                        && let Some(b) = &g.branch
                    {
                        g.upstream = Some(format!("origin/{b}"));
                    }
                }
            }
            Effect::GitPushRemote(path, remote) => {
                if let Some(g) = self.git.iter_mut().find(|g| &g.path == path) {
                    let tracks = g
                        .upstream
                        .as_deref()
                        .is_some_and(|u| u.starts_with(&format!("{remote}/")));
                    if tracks || g.upstream.is_none() {
                        g.ahead = 0;
                        g.push_rejected = None;
                    }
                }
            }
            Effect::GitFetch(_) => {
                // remote-tracking refs are already what the fixture reports
            }
            Effect::GitSwitch(path, branch) => {
                if let Some(g) = self.git.iter_mut().find(|g| &g.path == path) {
                    g.branch = Some(branch.clone());
                    if branch == &g.primary {
                        g.ahead = 0;
                        g.behind = 0;
                        g.diverged = false;
                        g.upstream = Some(format!("origin/{branch}"));
                    }
                }
            }
            Effect::GitGc(path) => {
                if let Some(g) = self.git.iter_mut().find(|g| &g.path == path) {
                    g.gc_needed = false;
                }
            }
            Effect::GitDeleteBranches(path, names) => {
                if let Some(g) = self.git.iter_mut().find(|g| &g.path == path) {
                    g.merged.retain(|b| !names.contains(b));
                }
            }
            Effect::MiseUpgrade => {
                for t in self.mise.global_tools.iter_mut().chain(
                    self.mise
                        .configs
                        .iter_mut()
                        .flat_map(|c| c.tools.iter_mut()),
                ) {
                    if t.installed {
                        t.active = Some(t.latest.clone());
                    }
                }
            }
            Effect::BrewService(name, verb) => {
                if let Some(b) = &mut self.brew
                    && let Some(s) = b.services.iter_mut().find(|s| &s.name == name)
                {
                    s.status = match verb.as_str() {
                        "start" | "restart" | "run" => "started".into(),
                        "stop" => "none".into(),
                        _ => s.status.clone(),
                    };
                }
            }
            Effect::BrewUpgrade(casks) => {
                self.upgrade.brew_outdated.clear();
                if *casks {
                    self.upgrade.casks_outdated.clear();
                }
            }
            Effect::AmpUpdate | Effect::OmzUpgrade => {
                // the tools report themselves current; nothing else in the
                // world depends on their version
            }
            Effect::GradleStop => {
                self.gradle.daemon_running = false;
            }
            Effect::GradleClean(path) => {
                let build = format!("{path}/build");
                let _ = self.fs.remove_permanent(&build);
                self.remove_candidate(&build, "gradle clean");
            }
            Effect::KillProcess(pid) => {
                self.system.top.retain(|p| p.pid != *pid);
            }
            Effect::DeleteAll(path) => {
                let children: Vec<String> = self
                    .fs
                    .list(path)
                    .map(|v| v.iter().map(|n| n.path.clone()).collect())
                    .unwrap_or_default();
                for c in children {
                    let _ = self.fs.remove_permanent(&c);
                    self.remove_candidate(&c, "rm -rf");
                }
                let prefix = format!("{path}/");
                self.disk
                    .candidates
                    .retain(|c| !c.path.starts_with(&prefix));
                self.disk.large.retain(|l| !l.path.starts_with(&prefix));
            }
            Effect::MiseInstall => {
                for cfg in &mut self.mise.configs {
                    for t in &mut cfg.tools {
                        if !t.installed {
                            t.installed = true;
                            t.active = Some(t.requested.clone());
                        }
                    }
                }
                for t in &mut self.mise.global_tools {
                    if !t.installed {
                        t.installed = true;
                        t.active = Some(t.requested.clone());
                    }
                }
            }
        }
    }

    /// Remove a cleanup candidate that was reclaimed and record it.
    fn remove_candidate(&mut self, path: &str, method: &str) {
        let Some(i) = self.disk.candidates.iter().position(|c| c.path == path) else {
            return;
        };
        let c = self.disk.candidates.remove(i);
        if let Some(fs) = self.disk.filesystems.first_mut() {
            fs.used_gb = fs.used_gb.saturating_sub(c.gb.round() as u32);
        }
        self.disk.history.push(CleanupRecord {
            when_secs: self.now_secs(),
            target: c.path.clone(),
            method: method.to_owned(),
            reclaimed_gb: c.gb,
            outcome: "succeeded".into(),
        });
    }

    /// Land a finished plan's effects: only the steps that succeeded change
    /// anything, so a partial run leaves a truthful partial world.
    fn apply_plan_effects(&mut self, pi: usize) {
        let plan = self.plans[pi].clone();
        let ok = |id: &str| {
            plan.steps
                .iter()
                .any(|s| s.id == id && s.state == StepState::Succeeded)
        };
        match plan.id.as_str() {
            "docker-cleanup" => {
                if ok("stop") {
                    for c in &mut self.docker.containers {
                        c.running = false;
                        c.cpu_pct = 0.0;
                    }
                }
                if ok("rm") {
                    self.docker.containers.clear();
                }
                if ok("images") {
                    self.docker.images = 0;
                    self.docker.images_gb = 0.0;
                    self.docker.dangling_images = 0;
                }
                if ok("networks") {
                    self.docker.networks = 0;
                }
                if ok("volumes") {
                    self.docker.volumes.clear();
                }
                if ok("builder") {
                    self.docker.builder_cache_gb = 0.0;
                }
                self.docker.reclaimable_gb =
                    self.docker.images_gb + self.docker.volumes_gb() + self.docker.builder_cache_gb;
                self.docker.cleanup_uses += 1;
            }
            "upgrade" => {
                if ok("apt-apply")
                    && let Some(apt) = &mut self.apt
                {
                    apt.upgradable.clear();
                    apt.pending_reboot = true;
                }
                if ok("mise-upgrade") {
                    for t in &mut self.mise.global_tools {
                        if t.name != "go" && t.installed {
                            t.active = Some(t.latest.clone());
                        }
                    }
                }
            }
            "upgrade-all" => {
                if ok("brew-upgrade") {
                    self.upgrade.brew_outdated.clear();
                }
                if ok("brew-casks") {
                    self.upgrade.casks_outdated.clear();
                }
                if ok("mise-upgrade") {
                    for t in self.mise.global_tools.iter_mut().chain(
                        self.mise
                            .configs
                            .iter_mut()
                            .flat_map(|c| c.tools.iter_mut()),
                    ) {
                        if t.installed {
                            t.active = Some(t.latest.clone());
                        }
                    }
                }
            }
            "cleanup-work" => {
                for s in plan
                    .steps
                    .iter()
                    .filter(|s| s.state == StepState::Succeeded)
                {
                    let Some(target) = s.target.clone() else {
                        continue;
                    };
                    let method = s
                        .commands
                        .first()
                        .and_then(|c| c.split_whitespace().next())
                        .unwrap_or("cleanup")
                        .to_owned();
                    self.remove_candidate(&target, &method);
                }
            }
            "git-pull-all" => {
                for s in plan
                    .steps
                    .iter()
                    .filter(|s| s.state == StepState::Succeeded)
                {
                    if let Some(g) = self.git.iter_mut().find(|g| g.path == s.cwd) {
                        g.behind = 0;
                    }
                }
            }
            "git-switch-primary" => {
                for s in plan
                    .steps
                    .iter()
                    .filter(|s| s.state == StepState::Succeeded)
                {
                    if let Some(g) = self.git.iter_mut().find(|g| g.path == s.cwd) {
                        g.branch = Some(g.primary.clone());
                    }
                }
            }
            _ => {}
        }
    }

    /// Start (or restart) a scan of `path`: the previous generation is
    /// dropped, cache-valid nodes remain hints until live sizes arrive.
    pub fn start_scan(&mut self, path: &str) {
        let generation = self.scan.as_ref().map(|s| s.generation + 1).unwrap_or(1);
        let include_hidden = true;
        let mut skip = vec![];
        if path == "/" {
            skip.push("/System/Volumes/Data".to_owned());
        }
        skip.push(format!("{}/Library/Mobile Documents", self.location.home));
        let tree = self.fs.scan(
            path,
            &crate::sim::fs::ScanOptions {
                include_hidden,
                skip,
                max_depth: None,
            },
        );
        let mut order = vec![];
        fn walk(n: &crate::sim::fs::ScanNode, out: &mut Vec<String>) {
            out.push(n.path.clone());
            for c in &n.children {
                walk(c, out);
            }
        }
        walk(&tree, &mut order);
        let snapshot: Vec<(String, i64)> = self
            .fs
            .subtree(path)
            .iter()
            .map(|n| (n.path.clone(), n.mtime))
            .chain(self.fs.get(path).map(|n| (n.path.clone(), n.mtime)))
            .collect();
        let ticks = self.disk.scan_ticks.max(1);
        self.scan = Some(ScanState {
            root: path.into(),
            started: self.tick,
            generation,
            ticks,
            tree,
            order,
            snapshot,
            cancelled: false,
            include_hidden,
            batch: 4096,
            revealed_at_cancel: 0,
        });
        self.scan_started = Some(self.tick);
    }

    /// Cancel the running scan: the partial tree is retained and labelled.
    pub fn cancel_scan(&mut self) -> bool {
        let tick = self.tick;
        match self.scan.as_mut() {
            Some(s) if !s.done(tick) => {
                s.revealed_at_cancel = s.revealed(tick);
                s.cancelled = true;
                true
            }
            _ => false,
        }
    }

    /// A finished scan records the size cache (root plus depth two) and
    /// persists it; changed paths since the pre-scan snapshot are omitted.
    pub fn finish_scan_cache(&mut self) {
        let Some(s) = self.scan.clone() else {
            return;
        };
        if s.cancelled {
            return;
        }
        let now = self.now_secs();
        let on_disk = crate::domain::cleanup::SizeCache::load(self.persisted.sizes.as_deref(), now);
        self.size_cache.merge(&on_disk, now);
        self.size_cache
            .record(&s.root, &s.tree, &s.snapshot, &self.fs, now);
        if self.size_cache.save_error.is_none() {
            self.persisted.sizes = Some(self.size_cache.serialize());
        }
    }

    /// The catalogue for this world.
    pub fn items(&self) -> Vec<Item> {
        crate::domain::catalog::build(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceState {
    Loading,
    Done,
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::fixtures::world_for;
    use crate::scenario::{Motion, Scenario};

    #[test]
    fn ticks_are_deterministic_and_paused_worlds_never_move() {
        let mut a = world_for(Scenario::ActivitiesMulti, Motion::Full);
        let mut b = world_for(Scenario::ActivitiesMulti, Motion::Full);
        for _ in 0..40 {
            a.tick();
            b.tick();
        }
        assert_eq!(a.tick, b.tick);
        assert_eq!(
            a.activities
                .iter()
                .map(|x| x.output.len())
                .collect::<Vec<_>>(),
            b.activities
                .iter()
                .map(|x| x.output.len())
                .collect::<Vec<_>>()
        );
        let mut p = world_for(Scenario::ActivitiesMulti, Motion::Paused);
        let before: Vec<usize> = p.activities.iter().map(|x| x.output.len()).collect();
        for _ in 0..40 {
            assert!(p.tick().is_empty());
        }
        assert_eq!(
            p.activities
                .iter()
                .map(|x| x.output.len())
                .collect::<Vec<_>>(),
            before
        );
    }

    #[test]
    fn plans_run_parallel_branches_and_respect_locks() {
        let mut w = world_for(Scenario::UpgradePlan, Motion::Full);
        let pi = 0;
        w.plans[pi].arm(w.tick);
        let mut seen_parallel = false;
        for _ in 0..400 {
            let msgs = w.tick();
            let running = w.plans[pi].running();
            if running.len() == 2 {
                seen_parallel = true;
                let a = &w.plans[pi].steps[running[0]];
                let b = &w.plans[pi].steps[running[1]];
                assert!(
                    a.exclusive.is_none() || a.exclusive != b.exclusive,
                    "two steps holding {:?} at once",
                    a.exclusive
                );
            }
            if msgs.iter().any(|m| matches!(m, Msg::PlanDone { .. })) {
                break;
            }
        }
        assert!(seen_parallel, "independent branches ran side by side");
        assert_eq!(w.plans[pi].phase, PlanPhase::Done);
    }
}
