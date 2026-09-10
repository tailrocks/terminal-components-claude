//! The fixture world and its tick: discovery sources completing, activities
//! emitting scripted output, plans scheduling ready steps within their
//! parallelism and exclusive-resource rules, and the disk scan streaming
//! candidates. Every change is a pure function of the tick count.

use std::collections::BTreeMap;

use crate::clock::Clock;
use crate::domain::action::Item;
use crate::domain::activity::{Activity, ActivityKind, ActivityState, Script};
use crate::domain::context::{Host, Location, ScopeTag};
use crate::domain::plan::{Plan, PlanPhase, StepState};
use crate::domain::stack::{
    AptState, DiskState, DockerState, GitState, GithubState, MiseState, PgState, RankingMemory,
    SshState, SystemSnapshot,
};
use crate::scenario::Scenario;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    ActivityEnded { id: String, ok: bool },
    PlanStepEnded { plan: String, step: usize, ok: bool },
    PlanDone { plan: String },
    DiscoveryDone { source: String },
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
    /// Personal workflows contributed by the user (name, command, trusted).
    pub workflows: Vec<(String, String, bool)>,
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

    /// Start an activity from a script key. Returns its id.
    pub fn start_activity(
        &mut self,
        script_key: &str,
        name: &str,
        origin: &str,
        kind: ActivityKind,
        scope: ScopeTag,
    ) -> String {
        let script = self.scripts.get(script_key).cloned().unwrap_or_else(|| {
            match script_key.strip_prefix("generic:") {
                Some(cmd) => crate::domain::scripts::one_shot(cmd, &["done"]),
                None => Script::ending(vec![crate::domain::activity::line(0, "(no output)")], 2, 0),
            }
        });
        self.next_activity += 1;
        let id = format!("act-{}", self.next_activity);
        let host = self.host.name.clone();
        let mut a = Activity::new(&id, name, origin, kind, scope, &host, script, self.tick);
        a.attachable = matches!(
            a.kind,
            ActivityKind::Monitor { .. } | ActivityKind::Ssh { .. }
        );
        a.advance(self.tick);
        self.activities.push(a);
        id
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
        }
        let tick = self.tick;
        for a in &mut self.activities {
            let was = a.state;
            a.advance(tick);
            if was.live() && !a.state.live() {
                msgs.push(Msg::ActivityEnded {
                    id: a.id.clone(),
                    ok: a.state == ActivityState::Succeeded,
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
            msgs.push(Msg::PlanDone { plan: plan_id });
        }
        msgs
    }

    /// Ticks since the scan started, capped at the scan length.
    pub fn scan_progress(&self) -> Option<(u64, u64)> {
        self.scan_started.map(|s| {
            (
                self.tick.saturating_sub(s).min(self.disk.scan_ticks),
                self.disk.scan_ticks,
            )
        })
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
