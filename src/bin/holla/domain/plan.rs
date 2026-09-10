//! Plans: a dependency graph of steps with optional branches, exclusion
//! consequences, parallel eligibility and failure propagation (CONCEPT
//! §8.15). Pure logic; execution timing lives in `sim`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepState {
    /// Every prerequisite has succeeded; may start.
    Ready,
    /// Waiting for a prerequisite that has not finished.
    Waiting,
    Running,
    Succeeded,
    Failed,
    /// Optional step the user skipped after it failed.
    Skipped,
    /// A prerequisite failed or was excluded.
    Blocked,
    Cancelled,
    /// The user excluded it before execution.
    Excluded,
}

impl StepState {
    pub fn label(self) -> &'static str {
        match self {
            StepState::Ready => "ready",
            StepState::Waiting => "waiting",
            StepState::Running => "running",
            StepState::Succeeded => "succeeded",
            StepState::Failed => "failed",
            StepState::Skipped => "skipped",
            StepState::Blocked => "blocked",
            StepState::Cancelled => "cancelled",
            StepState::Excluded => "excluded",
        }
    }

    pub fn finished(self) -> bool {
        matches!(
            self,
            StepState::Succeeded
                | StepState::Failed
                | StepState::Skipped
                | StepState::Blocked
                | StepState::Cancelled
                | StepState::Excluded
        )
    }
}

/// One inspectable unit of a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// Short stable id (`preflight`, `apt-apply`).
    pub id: String,
    pub label: String,
    pub commands: Vec<String>,
    pub cwd: String,
    /// `sudo`, `docker socket`, … or empty.
    pub privilege: String,
    pub optional: bool,
    /// Included in the plan (optional steps may be excluded).
    pub included: bool,
    /// Indices of prerequisite steps.
    pub deps: Vec<usize>,
    /// Steps sharing an exclusive resource are ordered even when
    /// independent (`apt lock`, `docker daemon`).
    pub exclusive: Option<String>,
    pub effects: Vec<String>,
    /// Known uncertainty, shown in review.
    pub note: String,
    pub state: StepState,
    /// Why the step is blocked or excluded, when it is.
    pub because: String,
    /// Scripted duration in ticks and whether the fixture makes it fail.
    pub duration_ticks: u64,
    pub fails: bool,
    pub started_tick: Option<u64>,
    pub ended_tick: Option<u64>,
    pub exit: Option<i32>,
    /// Retained output lines.
    pub output: Vec<String>,
}

impl Step {
    pub fn new(id: &str, label: &str, commands: &[&str], cwd: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            commands: commands.iter().map(|s| (*s).to_owned()).collect(),
            cwd: cwd.into(),
            privilege: String::new(),
            optional: false,
            included: true,
            deps: vec![],
            exclusive: None,
            effects: vec![],
            note: String::new(),
            state: StepState::Ready,
            because: String::new(),
            duration_ticks: 12,
            fails: false,
            started_tick: None,
            ended_tick: None,
            exit: None,
            output: vec![],
        }
    }
    pub fn after(mut self, deps: &[usize]) -> Self {
        self.deps = deps.to_vec();
        self
    }
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    pub fn excluded(mut self) -> Self {
        self.optional = true;
        self.included = false;
        self
    }
    pub fn privilege(mut self, p: &str) -> Self {
        self.privilege = p.into();
        self
    }
    pub fn exclusive(mut self, e: &str) -> Self {
        self.exclusive = Some(e.into());
        self
    }
    pub fn effects(mut self, e: &[&str]) -> Self {
        self.effects = e.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn note(mut self, n: &str) -> Self {
        self.note = n.into();
        self
    }
    pub fn ticks(mut self, t: u64) -> Self {
        self.duration_ticks = t;
        self
    }
    pub fn fails(mut self) -> Self {
        self.fails = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanPhase {
    Review,
    Running,
    Done,
}

/// A reviewable execution graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub id: String,
    pub title: String,
    /// The intent in plain language.
    pub intent: String,
    pub host: String,
    pub steps: Vec<Step>,
    /// Second-gate phrase when the plan is broadly destructive.
    pub phrase: Option<String>,
    pub broad: bool,
    pub phase: PlanPhase,
    /// Bounded parallelism.
    pub max_parallel: usize,
    /// Independent branches keep running when one fails.
    pub continue_on_failure: bool,
    /// Tick the run started, for durations.
    pub started_tick: Option<u64>,
    pub ended_tick: Option<u64>,
    /// Facts shown at Gate 1 besides the steps.
    pub facts: Vec<(String, String)>,
    /// Fixture-side revalidation: when set, the first Execute finds a
    /// changed target and invalidates confirmation (CONCEPT §10).
    pub drift: Option<String>,
    pub drift_consumed: bool,
    /// What to offer once the plan has finished: (label, item id).
    pub follow_ups: Vec<(String, String)>,
}

impl Plan {
    pub fn new(id: &str, title: &str, intent: &str, host: &str, steps: Vec<Step>) -> Self {
        let mut p = Self {
            id: id.into(),
            title: title.into(),
            intent: intent.into(),
            host: host.into(),
            steps,
            phrase: None,
            broad: false,
            phase: PlanPhase::Review,
            max_parallel: 2,
            continue_on_failure: true,
            started_tick: None,
            ended_tick: None,
            facts: vec![],
            drift: None,
            drift_consumed: false,
            follow_ups: vec![],
        };
        p.recompute();
        p
    }
    pub fn phrase(mut self, p: &str, broad: bool) -> Self {
        self.phrase = Some(p.into());
        self.broad = broad;
        self
    }
    pub fn follow_up(mut self, label: &str, item: &str) -> Self {
        self.follow_ups.push((label.into(), item.into()));
        self
    }
    pub fn fact(mut self, k: &str, v: &str) -> Self {
        self.facts.push((k.into(), v.into()));
        self
    }
    pub fn drift(mut self, d: &str) -> Self {
        self.drift = Some(d.into());
        self
    }

    /// The full second-gate phrase, with the broad prefix.
    pub fn gate_phrase(&self) -> Option<String> {
        self.phrase.as_ref().map(|p| {
            if self.broad {
                format!("I UNDERSTAND: {p}")
            } else {
                p.clone()
            }
        })
    }

    /// Toggle an optional step; recomputes dependents. Returns false when the
    /// step is required.
    pub fn toggle(&mut self, i: usize) -> bool {
        if self.phase != PlanPhase::Review {
            return false;
        }
        let Some(s) = self.steps.get_mut(i) else {
            return false;
        };
        if !s.optional {
            return false;
        }
        s.included = !s.included;
        self.recompute();
        true
    }

    /// Steps that depend directly on `i`.
    pub fn dependents(&self, i: usize) -> Vec<usize> {
        (0..self.steps.len())
            .filter(|&j| self.steps[j].deps.contains(&i))
            .collect()
    }

    /// Steps `i` may run beside: included, not in each other's ancestry,
    /// and not sharing an exclusive resource.
    pub fn parallel_with(&self, i: usize) -> Vec<usize> {
        (0..self.steps.len())
            .filter(|&j| j != i && self.steps[j].included && self.steps[i].included)
            .filter(|&j| !self.ancestor(i, j) && !self.ancestor(j, i))
            .filter(|&j| {
                !(self.steps[i].exclusive.is_some()
                    && self.steps[i].exclusive == self.steps[j].exclusive)
            })
            .collect()
    }

    /// True when `a` is an ancestor of `b`.
    pub fn ancestor(&self, a: usize, b: usize) -> bool {
        let mut stack = vec![b];
        let mut seen = vec![false; self.steps.len()];
        while let Some(x) = stack.pop() {
            if seen[x] {
                continue;
            }
            seen[x] = true;
            for &d in &self.steps[x].deps {
                if d == a {
                    return true;
                }
                stack.push(d);
            }
        }
        false
    }

    /// Recompute states from inclusion and results. In review, excluded
    /// prerequisites block their dependents with a reason; while running,
    /// finished prerequisites unlock dependents and failures block them.
    pub fn recompute(&mut self) {
        let n = self.steps.len();
        for i in 0..n {
            let s = &self.steps[i];
            if s.state == StepState::Running
                || s.state == StepState::Succeeded
                || s.state == StepState::Failed
                || s.state == StepState::Skipped
                || s.state == StepState::Cancelled
            {
                continue;
            }
            if !s.included {
                self.steps[i].state = StepState::Excluded;
                self.steps[i].because = "excluded".into();
                continue;
            }
            let deps = self.steps[i].deps.clone();
            let mut blocked_by: Option<String> = None;
            let mut waiting = false;
            for d in deps {
                match self.steps[d].state {
                    StepState::Excluded | StepState::Blocked => {
                        blocked_by = Some(format!(
                            "blocked · needs {:02} {}",
                            d + 1,
                            self.steps[d].label.to_lowercase()
                        ));
                    }
                    StepState::Failed | StepState::Cancelled => {
                        blocked_by = Some(format!("blocked · {:02} failed", d + 1));
                    }
                    StepState::Skipped => {
                        // an optional failure that was skipped does not unblock
                        // dependents that need its result
                        blocked_by = Some(format!("blocked · {:02} skipped", d + 1));
                    }
                    StepState::Succeeded => {}
                    _ => waiting = true,
                }
            }
            self.steps[i].state = if let Some(why) = blocked_by {
                self.steps[i].because = why;
                StepState::Blocked
            } else if waiting {
                self.steps[i].because.clear();
                StepState::Waiting
            } else {
                self.steps[i].because.clear();
                StepState::Ready
            };
        }
    }

    /// Included steps that are ready to start, in order.
    pub fn ready(&self) -> Vec<usize> {
        (0..self.steps.len())
            .filter(|&i| self.steps[i].included && self.steps[i].state == StepState::Ready)
            .collect()
    }

    pub fn running(&self) -> Vec<usize> {
        (0..self.steps.len())
            .filter(|&i| self.steps[i].state == StepState::Running)
            .collect()
    }

    /// The plan is finished when nothing can still start or run.
    pub fn is_done(&self) -> bool {
        self.steps
            .iter()
            .all(|s| s.state.finished() || (!s.included))
            && self.running().is_empty()
            && self.ready().is_empty()
    }

    /// Counts by outcome for the summary: (succeeded, failed, excluded,
    /// skipped, never started).
    pub fn outcome(&self) -> (usize, usize, usize, usize, usize) {
        let c = |st: StepState| self.steps.iter().filter(|s| s.state == st).count();
        (
            c(StepState::Succeeded),
            c(StepState::Failed),
            c(StepState::Excluded),
            c(StepState::Skipped),
            c(StepState::Blocked)
                + c(StepState::Cancelled)
                + c(StepState::Waiting)
                + c(StepState::Ready),
        )
    }

    pub fn included_count(&self) -> usize {
        self.steps.iter().filter(|s| s.included).count()
    }

    pub fn optional_count(&self) -> usize {
        self.steps.iter().filter(|s| s.optional).count()
    }

    /// Estimated duration in ticks along the critical path of included
    /// steps.
    pub fn estimate_ticks(&self) -> u64 {
        fn finish(p: &Plan, i: usize, memo: &mut Vec<Option<u64>>) -> u64 {
            if let Some(v) = memo[i] {
                return v;
            }
            let start = p.steps[i]
                .deps
                .iter()
                .map(|&d| finish(p, d, memo))
                .max()
                .unwrap_or(0);
            let v = if p.steps[i].included {
                start + p.steps[i].duration_ticks
            } else {
                start
            };
            memo[i] = Some(v);
            v
        }
        let mut memo = vec![None; self.steps.len()];
        (0..self.steps.len())
            .map(|i| finish(self, i, &mut memo))
            .max()
            .unwrap_or(0)
    }

    /// Included steps with no prerequisites.
    pub fn roots(&self) -> Vec<usize> {
        (0..self.steps.len())
            .filter(|&i| self.steps[i].deps.is_empty() && self.steps[i].included)
            .collect()
    }

    /// Branch count: independent included roots, or the widest fan-out
    /// after a shared prerequisite.
    pub fn parallel_branches(&self) -> usize {
        let mut best = self.roots().len();
        for i in 0..self.steps.len() {
            let n = self
                .dependents(i)
                .into_iter()
                .filter(|&j| self.steps[j].included)
                .count();
            best = best.max(n);
        }
        best
    }

    /// Retry a failed step: it becomes ready again and its blocked
    /// dependents are recomputed; succeeded unrelated work is untouched.
    pub fn retry(&mut self, i: usize) -> bool {
        let Some(s) = self.steps.get_mut(i) else {
            return false;
        };
        if s.state != StepState::Failed {
            return false;
        }
        s.state = StepState::Ready;
        s.fails = false;
        s.started_tick = None;
        s.ended_tick = None;
        s.exit = None;
        s.output.push("retrying…".into());
        let deps = self.dependents(i);
        for d in deps {
            if self.steps[d].state == StepState::Blocked {
                self.steps[d].state = StepState::Waiting;
            }
        }
        self.recompute();
        self.phase = PlanPhase::Running;
        self.ended_tick = None;
        true
    }

    /// Skip an optional failed step; dependents stay blocked.
    pub fn skip(&mut self, i: usize) -> bool {
        let Some(s) = self.steps.get_mut(i) else {
            return false;
        };
        if s.state != StepState::Failed || !s.optional {
            return false;
        }
        s.state = StepState::Skipped;
        self.recompute();
        true
    }

    /// Cancel everything that has not started.
    pub fn cancel_remaining(&mut self) {
        for s in &mut self.steps {
            if matches!(s.state, StepState::Ready | StepState::Waiting) {
                s.state = StepState::Cancelled;
                s.because = "cancelled".into();
            }
        }
        self.recompute();
    }

    /// Reset every step for a fresh run after confirmation.
    pub fn arm(&mut self, tick: u64) {
        for s in &mut self.steps {
            s.state = StepState::Ready;
            s.started_tick = None;
            s.ended_tick = None;
            s.exit = None;
            s.output.clear();
        }
        self.phase = PlanPhase::Running;
        self.started_tick = Some(tick);
        self.ended_tick = None;
        self.recompute();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upgrade() -> Plan {
        Plan::new(
            "upgrade",
            "Upgrade everything on devbox",
            "upgrade",
            "devbox",
            vec![
                Step::new("preflight", "Preflight", &["true"], "/"),
                Step::new("apt-update", "Debian metadata", &["apt-get update"], "/")
                    .after(&[0])
                    .exclusive("apt lock"),
                Step::new("apt-review", "Review Debian upgrades", &["apt list"], "/").after(&[1]),
                Step::new(
                    "apt-apply",
                    "Apply Debian upgrades",
                    &["apt-get upgrade"],
                    "/",
                )
                .after(&[2])
                .exclusive("apt lock"),
                Step::new(
                    "mise-inspect",
                    "Inspect global mise tools",
                    &["mise outdated"],
                    "/",
                )
                .after(&[0]),
                Step::new(
                    "mise-upgrade",
                    "Upgrade selected mise tools",
                    &["mise upgrade"],
                    "/",
                )
                .after(&[4])
                .optional(),
                Step::new("cleanup", "Package cleanup", &["apt-get autoremove"], "/")
                    .after(&[3])
                    .optional()
                    .exclusive("apt lock"),
                Step::new("verify", "Final verification", &["true"], "/").after(&[3, 5]),
            ],
        )
    }

    #[test]
    fn exclusion_cascades_with_a_reason_and_recomputes() {
        let mut p = upgrade();
        assert_eq!(p.steps[7].state, StepState::Waiting);
        assert!(p.toggle(5), "optional step toggles");
        assert!(!p.steps[5].included);
        assert_eq!(p.steps[5].state, StepState::Excluded);
        assert_eq!(p.steps[7].state, StepState::Blocked);
        assert!(p.steps[7].because.contains("06"), "{}", p.steps[7].because);
        assert!(!p.toggle(0), "required steps cannot be excluded");
        p.toggle(5);
        assert_eq!(p.steps[7].state, StepState::Waiting);
    }

    #[test]
    fn parallel_eligibility_respects_ancestry_and_locks() {
        let p = upgrade();
        // Debian metadata and mise inspection are independent branches
        assert!(p.parallel_with(1).contains(&4));
        // apt-update and apt-apply share the apt lock
        assert!(!p.parallel_with(1).contains(&3));
        // verify depends on both branches: never parallel with apply
        assert!(!p.parallel_with(7).contains(&3));
        assert!(p.ancestor(0, 7));
        assert!(!p.ancestor(7, 0));
        assert_eq!(p.parallel_branches(), 2);
    }

    #[test]
    fn failure_blocks_dependents_and_retry_restores_them() {
        let mut p = upgrade();
        p.arm(0);
        assert_eq!(p.ready(), vec![0]);
        p.steps[0].state = StepState::Succeeded;
        p.recompute();
        assert_eq!(p.ready(), vec![1, 4]);
        p.steps[1].state = StepState::Failed;
        p.recompute();
        assert_eq!(p.steps[2].state, StepState::Blocked);
        assert_eq!(p.steps[3].state, StepState::Blocked);
        assert_eq!(p.steps[7].state, StepState::Blocked);
        assert_eq!(
            p.steps[4].state,
            StepState::Ready,
            "the mise branch keeps going"
        );
        assert!(p.retry(1));
        assert_eq!(p.steps[1].state, StepState::Ready);
        assert_eq!(p.steps[2].state, StepState::Waiting);
        assert_eq!(
            p.steps[0].state,
            StepState::Succeeded,
            "succeeded work is not repeated"
        );
        let (ok, failed, excluded, skipped, never) = p.outcome();
        assert_eq!((ok, failed, excluded, skipped), (1, 0, 0, 0));
        assert!(never > 0);
    }

    #[test]
    fn estimate_follows_the_critical_path() {
        let mut p = upgrade();
        for s in &mut p.steps {
            s.duration_ticks = 10;
        }
        // preflight → apt-update → review → apply → cleanup = 5 steps
        assert_eq!(p.estimate_ticks(), 50);
        p.toggle(6);
        assert_eq!(
            p.estimate_ticks(),
            50,
            "verify path is as long as cleanup path"
        );
        p.toggle(5);
        assert_eq!(
            p.estimate_ticks(),
            50,
            "the Debian branch is the critical path"
        );
        p.steps[3].duration_ticks = 2;
        assert_eq!(p.estimate_ticks(), 42);
    }
}
