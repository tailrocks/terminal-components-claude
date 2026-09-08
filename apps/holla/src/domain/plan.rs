//! Plan: a compound intent as a reviewable DAG. Steps declare dependencies
//! and branches; the reviewer may exclude optional steps and the exclusion
//! recalculates dependents; a failed step propagates to everything that
//! needed it. Nothing executes — outcomes are fixture-deterministic.

/// A review fact expressed as domain text; the application view chooses its UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlanFact {
    pub(crate) label: String,
    pub(crate) value: String,
}

impl PlanFact {
    fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// Where a step stands. `Pending` becomes a terminal state on `run()`;
/// `Excluded` is the reviewer's choice and recalculates dependents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StepState {
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
    pub(crate) fn glyph(&self) -> &'static str {
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
pub(crate) struct PlanStep {
    pub(crate) success_effects: Vec<super::effect::Mutation>,
    pub(crate) failure_effects: Vec<super::effect::Mutation>,
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) command: String,
    /// Parallel branches share a label (`apt`, `mise`, `builder`).
    pub(crate) branch: String,
    /// Indexes of steps that must succeed first (always earlier indexes).
    pub(crate) deps: Vec<usize>,
    /// The reviewer may exclude optional steps; exclusion cascades.
    pub(crate) optional: bool,
    /// Deterministic failure reason when the step runs, if any.
    pub(crate) fails: Option<String>,
    pub(crate) ok_lines: Vec<String>,
    pub(crate) fail_lines: Vec<String>,
    pub(crate) state: StepState,
    /// Output lines after the run (empty while pending).
    pub(crate) lines: Vec<String>,
}

impl PlanStep {
    pub(crate) fn new(id: &str, title: &str, command: &str, branch: &str, deps: &[usize]) -> Self {
        Self {
            success_effects: Vec::new(),
            failure_effects: Vec::new(),
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

    pub(crate) fn succeeds_with(mut self, effects: Vec<super::effect::Mutation>) -> Self {
        self.ok_lines = effects
            .iter()
            .flat_map(|effect| effect.lines(false))
            .collect();
        if effects
            .iter()
            .any(|effect| matches!(effect, super::effect::Mutation::UpgradeTool { .. }))
        {
            self.ok_lines.push("shims reshimmed".into());
        }
        self.success_effects = effects;
        self
    }
    pub(crate) fn fails_after(
        mut self,
        reason: &str,
        effects: Vec<super::effect::Mutation>,
        error_line: &str,
    ) -> Self {
        self.fails = Some(reason.into());
        self.fail_lines = effects
            .iter()
            .flat_map(|effect| effect.lines(true))
            .collect();
        self.fail_lines.push(error_line.into());
        self.failure_effects = effects;
        self
    }

    pub(crate) fn required(mut self) -> Self {
        self.optional = false;
        self
    }
    pub(crate) fn fails_with(mut self, reason: &str, lines: &[&str]) -> Self {
        self.fails = Some(reason.into());
        self.fail_lines = lines.iter().map(std::string::ToString::to_string).collect();
        self
    }
    pub(crate) fn lines(mut self, lines: &[&str]) -> Self {
        self.ok_lines = lines.iter().map(std::string::ToString::to_string).collect();
        self
    }
    pub(crate) fn policy_skipped(mut self, reason: &str) -> Self {
        self.optional = false;
        self.state = StepState::PolicySkipped(reason.into());
        self
    }
}

/// A world mutation applied after a plan runs — the fixture stays honest:
/// what the plan claimed to change is changed. Payloads stay markers; the
/// apply step reads which steps actually succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PlanEffect {
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
pub(crate) struct PlanSpec {
    /// The catalogue action this plan reviews (`docker.cleanup`).
    pub(crate) action_id: String,
    pub(crate) title: String,
    pub(crate) host: String,
    /// The gate-2 typed phrase, bound to the target host.
    pub(crate) phrase: String,
    pub(crate) will_change: String,
    pub(crate) steps: Vec<PlanStep>,
    pub(crate) effect: Option<PlanEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plan {
    /// The catalogue action this plan reviews (`docker.cleanup`).
    action_id: String,
    title: String,
    host: String,
    /// The gate-2 typed phrase, bound to the target host.
    phrase: String,
    will_change: String,
    steps: Vec<PlanStep>,
    effect: Option<PlanEffect>,
    ran: bool,
}

/// Invalid plan declarations and review transitions fail before execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PlanError {
    DuplicateId,
    InvalidInventory,
    InvalidDependency,
    InvalidInitialState,
    MissingStep,
    AlreadyRan,
    RequiredStep(String),
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInventory => f.write_str("plan mutation inventory is invalid"),
            Self::DuplicateId => f.write_str("plan step IDs must be unique and nonempty"),
            Self::InvalidDependency => f.write_str("plan dependencies must name earlier steps"),
            Self::InvalidInitialState => f.write_str("plan contains an invalid initial step state"),
            Self::MissingStep => f.write_str("plan step no longer exists"),
            Self::AlreadyRan => f.write_str("plan already ran"),
            Self::RequiredStep(title) => write!(f, "{title} is required · cannot exclude"),
        }
    }
}
impl std::error::Error for PlanError {}

impl Plan {
    /// Validate a declaration-ordered DAG before it can enter review.
    ///
    /// # Errors
    /// Rejects duplicate/empty IDs, non-earlier dependencies, and pre-executed
    /// states. Earlier-only edges guarantee acyclic, bounded traversal.
    pub(crate) fn try_new(spec: PlanSpec) -> Result<Self, PlanError> {
        let mut ids = std::collections::BTreeSet::new();
        for (index, step) in spec.steps.iter().enumerate() {
            for effects in [&step.success_effects, &step.failure_effects] {
                let amounts: Result<Vec<_>, _> = effects
                    .iter()
                    .map(super::effect::Mutation::reclaimed_bytes)
                    .collect();
                let amounts = amounts.map_err(|_| PlanError::InvalidInventory)?;
                super::accounting::bytes(amounts).map_err(|_| PlanError::InvalidInventory)?;
            }

            if step.id.is_empty() || !ids.insert(&step.id) {
                return Err(PlanError::DuplicateId);
            }
            if step.deps.iter().any(|&dependency| dependency >= index) {
                return Err(PlanError::InvalidDependency);
            }
            if !matches!(
                step.state,
                StepState::Pending | StepState::Excluded | StepState::PolicySkipped(_)
            ) || (!step.optional && step.state == StepState::Excluded)
                || (step.optional && matches!(step.state, StepState::PolicySkipped(_)))
            {
                return Err(PlanError::InvalidInitialState);
            }
        }
        Ok(Self {
            action_id: spec.action_id,
            title: spec.title,
            host: spec.host,
            phrase: spec.phrase,
            will_change: spec.will_change,
            steps: spec.steps,
            effect: spec.effect,
            ran: false,
        })
    }

    pub(crate) fn action_id(&self) -> &str {
        &self.action_id
    }
    pub(crate) fn title(&self) -> &str {
        &self.title
    }
    pub(crate) fn host(&self) -> &str {
        &self.host
    }
    pub(crate) fn phrase(&self) -> &str {
        &self.phrase
    }
    pub(crate) fn will_change(&self) -> &str {
        &self.will_change
    }
    pub(crate) fn steps(&self) -> &[PlanStep] {
        &self.steps
    }
    pub(crate) fn ran(&self) -> bool {
        self.ran
    }
    pub(crate) fn effect(&self) -> Option<&PlanEffect> {
        self.effect.as_ref()
    }

    /// Included = not excluded and not policy-skipped.
    pub(crate) fn included_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| !matches!(s.state, StepState::Excluded | StepState::PolicySkipped(_)))
            .count()
    }

    /// The first excluded dependency of a pending step, if any — the
    /// recalculated consequence a reviewer sees before running anything.
    pub(crate) fn blocked_by_exclusion(&self, i: usize) -> Option<String> {
        if !matches!(self.steps.get(i)?.state, StepState::Pending) {
            return None;
        }
        let mut blocked = vec![false; i + 1];
        for (index, step) in self.steps.iter().take(i + 1).enumerate() {
            blocked[index] = matches!(step.state, StepState::Excluded)
                || step.deps.iter().any(|&dependency| blocked[dependency]);
        }
        self.steps[i]
            .deps
            .iter()
            .find(|&&dependency| blocked[dependency])
            .map(|&dependency| self.steps[dependency].title.clone())
    }

    /// Space on a row: toggle exclusion of an optional, not-yet-run step.
    pub(crate) fn toggle(&mut self, i: usize) -> Result<String, PlanError> {
        let s = self.steps.get(i).ok_or(PlanError::MissingStep)?;
        if self.ran {
            return Err(PlanError::AlreadyRan);
        }
        if !s.optional {
            return Err(PlanError::RequiredStep(s.title.clone()));
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
    pub(crate) fn run(&mut self) -> Result<(), PlanError> {
        if self.ran {
            return Err(PlanError::AlreadyRan);
        }
        self.ran = true;
        for i in 0..self.steps.len() {
            if !matches!(self.steps[i].state, StepState::Pending) {
                if matches!(self.steps[i].state, StepState::Excluded) {
                    self.steps[i].state = StepState::Skipped("excluded by reviewer".into());
                }
                continue;
            }
            let bad_dep = self.steps[i]
                .deps
                .iter()
                .find(|&&d| !matches!(self.steps[d].state, StepState::Succeeded));
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
        Ok(())
    }

    pub(crate) fn summary(&self) -> String {
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
    pub(crate) fn facts(&self) -> Vec<PlanFact> {
        vec![
            PlanFact::new("Target", format!("host {}", self.host)),
            PlanFact::new("Will change", &self.will_change),
            PlanFact::new(
                "Steps",
                format!(
                    "{} steps · {} included after review",
                    self.steps.len(),
                    self.included_count()
                ),
            ),
            PlanFact::new("Confirmation", "typed phrase, bound to the target"),
        ]
    }
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "Tests assert fixed fixture structure and bounded values; violations must fail the test"
)]
mod tests {
    use super::*;

    fn dag() -> Plan {
        Plan::try_new(PlanSpec {
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
        })
        .unwrap()
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
        p.run().unwrap();
        assert!(matches!(p.steps[1].state, StepState::Failed(_)));
        assert_eq!(p.steps[2].state, StepState::Skipped("needs Mutate".into()));
        assert_eq!(p.steps[3].state, StepState::Succeeded);
        assert_eq!(p.summary(), "2 succeeded · 1 failed · 1 skipped");
    }
    fn specification(steps: Vec<PlanStep>) -> PlanSpec {
        PlanSpec {
            action_id: "test".into(),
            title: "test".into(),
            host: "devbox".into(),
            phrase: "CONFIRM".into(),
            will_change: "fixture".into(),
            steps,
            effect: None,
        }
    }

    #[test]
    fn invalid_dag_declarations_are_rejected_without_execution() {
        for deps in [vec![0], vec![1], vec![999]] {
            assert_eq!(
                Plan::try_new(specification(vec![PlanStep::new(
                    "a", "A", "a", "a", &deps
                )]))
                .unwrap_err(),
                PlanError::InvalidDependency
            );
        }
        let cycle = vec![
            PlanStep::new("a", "A", "a", "a", &[1]),
            PlanStep::new("b", "B", "b", "b", &[0]),
        ];
        assert_eq!(
            Plan::try_new(specification(cycle)).unwrap_err(),
            PlanError::InvalidDependency
        );
        let duplicates = vec![
            PlanStep::new("a", "A", "a", "a", &[]),
            PlanStep::new("a", "B", "b", "b", &[0]),
        ];
        assert_eq!(
            Plan::try_new(specification(duplicates)).unwrap_err(),
            PlanError::DuplicateId
        );
    }

    #[test]
    fn policy_skipped_dependency_never_authorizes_effect() {
        let steps = vec![
            PlanStep::new("a", "A", "a", "a", &[]).policy_skipped("policy"),
            PlanStep::new("b", "B", "b", "b", &[0]),
            PlanStep::new("c", "C", "c", "c", &[]),
        ];
        let mut plan = Plan::try_new(specification(steps)).unwrap();
        plan.run().unwrap();
        assert_eq!(plan.steps()[1].state, StepState::Skipped("needs A".into()));
        assert_eq!(plan.steps()[2].state, StepState::Succeeded);
        let complete = plan.clone();
        assert_eq!(plan.run(), Err(PlanError::AlreadyRan));
        assert_eq!(plan, complete);
    }

    #[test]
    fn absent_review_row_returns_error_and_recalculation_is_bounded() {
        let mut plan = dag();
        assert_eq!(plan.toggle(999), Err(PlanError::MissingStep));
        assert_eq!(plan.blocked_by_exclusion(999), None);
        let mut steps = vec![PlanStep::new("root", "Root", "inspect", "root", &[])];
        for index in 1..10_000 {
            steps.push(PlanStep::new(
                &index.to_string(),
                "Step",
                "fixture",
                "chain",
                &[index - 1],
            ));
        }
        let mut plan = Plan::try_new(specification(steps)).unwrap();
        plan.toggle(0).unwrap();
        assert_eq!(plan.blocked_by_exclusion(9_999).as_deref(), Some("Step"));
    }
}
