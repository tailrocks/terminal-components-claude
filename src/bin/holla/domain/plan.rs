//! Plan: a compound intent as a reviewable DAG. Steps declare dependencies
//! and branches; the reviewer may exclude optional steps and the exclusion
//! recalculates dependents; a failed step propagates to everything that
//! needed it. Nothing executes — outcomes are fixture-deterministic.

use junie_tui::widgets::props::Prop;

/// Where a step stands. `Pending` becomes a terminal state on `run()`;
/// `Excluded` is the reviewer's choice and recalculates dependents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepState {
    Pending,
    /// Excluded in review; dependents become blocked.
    Excluded,
    /// Seeded before the run by policy (never executed, never excludable).
    PolicySkipped(String),
    Succeeded,
    Failed(String),
    /// A dependency failed, was skipped or excluded.
    Skipped(String),
}

impl StepState {
    pub fn glyph(&self) -> &'static str {
        match self {
            StepState::Pending => "·",
            StepState::Excluded => "−",
            StepState::PolicySkipped(_) => "−",
            StepState::Succeeded => "✓",
            StepState::Failed(_) => "✗",
            StepState::Skipped(_) => "−",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStep {
    pub id: String,
    pub title: String,
    pub command: String,
    /// Parallel branches share a label (`apt`, `mise`, `builder`).
    pub branch: String,
    /// Indexes of steps that must succeed first (always earlier indexes).
    pub deps: Vec<usize>,
    /// The reviewer may exclude optional steps; exclusion cascades.
    pub optional: bool,
    /// Deterministic failure reason when the step runs, if any.
    pub fails: Option<String>,
    pub ok_lines: Vec<String>,
    pub fail_lines: Vec<String>,
    pub state: StepState,
    /// Output lines after the run (empty while pending).
    pub lines: Vec<String>,
}

impl PlanStep {
    pub fn new(id: &str, title: &str, command: &str, branch: &str, deps: &[usize]) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            command: command.into(),
            branch: branch.into(),
            deps: deps.to_vec(),
            optional: true,
            fails: None,
            ok_lines: vec![],
            fail_lines: vec![],
            state: StepState::Pending,
            lines: vec![],
        }
    }

    pub fn required(mut self) -> Self {
        self.optional = false;
        self
    }
    pub fn fails_with(mut self, reason: &str, lines: &[&str]) -> Self {
        self.fails = Some(reason.into());
        self.fail_lines = lines.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn lines(mut self, lines: &[&str]) -> Self {
        self.ok_lines = lines.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn policy_skipped(mut self, reason: &str) -> Self {
        self.optional = false;
        self.state = StepState::PolicySkipped(reason.into());
        self
    }
}

/// A world mutation applied after a plan runs — the fixture stays honest:
/// what the plan claimed to change is changed. Payloads stay markers; the
/// apply step reads which steps actually succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanEffect {
    /// A restarted container becomes Running + Healthy.
    RestartContainer(String),
    /// Prune what succeeded: cache bytes, removed exited containers.
    DockerCleanup,
    /// Remove the candidates whose removal step succeeded.
    DiskReclaim,
    /// Upgrades applied: security pending clears, held stays, reboot stays.
    DebianUpgraded,
    /// Each succeeded child pull zeroes that child's behind count.
    GitSync,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The catalogue action this plan reviews (`docker.cleanup`).
    pub action_id: String,
    pub title: String,
    pub host: String,
    /// The gate-2 typed phrase, bound to the target host.
    pub phrase: String,
    pub will_change: String,
    pub steps: Vec<PlanStep>,
    pub effect: Option<PlanEffect>,
    pub ran: bool,
}

impl Plan {
    /// Included = not excluded and not policy-skipped.
    pub fn included_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| !matches!(s.state, StepState::Excluded | StepState::PolicySkipped(_)))
            .count()
    }

    /// The first excluded dependency of a pending step, if any — the
    /// recalculated consequence a reviewer sees before running anything.
    pub fn blocked_by_exclusion(&self, i: usize) -> Option<String> {
        if !matches!(self.steps[i].state, StepState::Pending) {
            return None;
        }
        self.steps[i]
            .deps
            .iter()
            .find(|&&d| {
                matches!(self.steps[d].state, StepState::Excluded)
                    || self.blocked_by_exclusion(d).is_some()
            })
            .map(|&d| self.steps[d].title.clone())
    }

    /// Space on a row: toggle exclusion of an optional, not-yet-run step.
    pub fn toggle(&mut self, i: usize) -> Result<String, String> {
        let s = &self.steps[i];
        if self.ran {
            return Err("plan already ran".into());
        }
        if !s.optional {
            return Err(format!("{} is required · cannot exclude", s.title));
        }
        let s = &mut self.steps[i];
        s.state = if matches!(s.state, StepState::Excluded) {
            StepState::Pending
        } else {
            StepState::Excluded
        };
        Ok(match self.steps[i].state {
            StepState::Excluded => format!("Excluded {}", self.steps[i].title),
            _ => format!("Included {}", self.steps[i].title),
        })
    }

    /// Deterministic execution in declaration order (deps are always
    /// earlier indexes). A failure or skip propagates to dependents.
    pub fn run(&mut self) {
        self.ran = true;
        for i in 0..self.steps.len() {
            if !matches!(self.steps[i].state, StepState::Pending) {
                if matches!(self.steps[i].state, StepState::Excluded) {
                    self.steps[i].state = StepState::Skipped("excluded by reviewer".into());
                }
                continue;
            }
            let bad_dep = self.steps[i].deps.iter().find(|&&d| {
                matches!(
                    self.steps[d].state,
                    StepState::Failed(_) | StepState::Skipped(_)
                )
            });
            if let Some(&d) = bad_dep {
                let dep = self.steps[d].title.clone();
                self.steps[i].state = StepState::Skipped(format!("needs {dep}"));
                continue;
            }
            let s = &mut self.steps[i];
            match s.fails.take() {
                Some(reason) => {
                    s.lines = std::mem::take(&mut s.fail_lines);
                    s.state = StepState::Failed(reason);
                }
                None => {
                    s.lines = std::mem::take(&mut s.ok_lines);
                    s.state = StepState::Succeeded;
                }
            }
        }
    }

    pub fn summary(&self) -> String {
        let count =
            |pred: fn(&StepState) -> bool| self.steps.iter().filter(|s| pred(&s.state)).count();
        let ok = count(|s| matches!(s, StepState::Succeeded));
        let failed = count(|s| matches!(s, StepState::Failed(_)));
        let skipped = count(|s| matches!(s, StepState::Skipped(_) | StepState::PolicySkipped(_)));
        let mut parts = vec![format!("{ok} succeeded")];
        if failed > 0 {
            parts.push(format!("{failed} failed"));
        }
        if skipped > 0 {
            parts.push(format!("{skipped} skipped"));
        }
        parts.join(" · ")
    }

    /// Gate-1 review facts (the full-body surface shows these plus the DAG).
    pub fn facts(&self) -> Vec<Prop> {
        vec![
            Prop::new("Target", format!("host {}", self.host)),
            Prop::new("Will change", &self.will_change),
            Prop::new(
                "Steps",
                format!(
                    "{} steps · {} included after review",
                    self.steps.len(),
                    self.included_count()
                ),
            ),
            Prop::new("Confirmation", "typed phrase, bound to the target"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dag() -> Plan {
        Plan {
            action_id: "test".into(),
            title: "Test plan".into(),
            host: "devbox".into(),
            phrase: "DO IT ON devbox".into(),
            will_change: "things".into(),
            steps: vec![
                PlanStep::new("a", "Inspect", "true", "inspect", &[]).required(),
                PlanStep::new("b", "Mutate", "mut", "mut", &[0]),
                PlanStep::new("c", "Depend", "dep", "mut", &[1]),
                PlanStep::new("d", "Parallel", "par", "other", &[0]),
            ],
            effect: None,
            ran: false,
        }
    }

    #[test]
    fn exclusion_recalculates_dependents() {
        let mut p = dag();
        p.toggle(1).unwrap();
        assert_eq!(p.blocked_by_exclusion(2).as_deref(), Some("Mutate"));
        assert_eq!(p.blocked_by_exclusion(3), None, "parallel branch free");
        assert_eq!(p.included_count(), 3);
        p.toggle(1).unwrap();
        assert_eq!(p.blocked_by_exclusion(2), None);
    }

    #[test]
    fn required_steps_refuse_exclusion() {
        let mut p = dag();
        assert!(p.toggle(0).is_err());
    }

    #[test]
    fn failure_propagates_but_parallel_branch_runs() {
        let mut p = dag();
        p.steps[1].fails = Some("boom".into());
        p.run();
        assert!(matches!(p.steps[1].state, StepState::Failed(_)));
        assert_eq!(p.steps[2].state, StepState::Skipped("needs Mutate".into()));
        assert_eq!(p.steps[3].state, StepState::Succeeded);
        assert_eq!(p.summary(), "2 succeeded · 1 failed · 1 skipped");
    }
}
