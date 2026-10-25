#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanFact {
    pub label: String,
    pub value: String,
}

impl PlanFact {
    fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepState {
    Pending,
    Excluded,
    PolicySkipped(String),
    Succeeded,
    Failed(String),
    Skipped(String),
}

impl StepState {
    pub fn glyph(&self) -> &'static str {
        match self {
            StepState::Pending => "·",
            StepState::Excluded | StepState::PolicySkipped(_) | StepState::Skipped(_) => "−",
            StepState::Succeeded => "✓",
            StepState::Failed(_) => "✗",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStep {
    pub success_effects: Vec<holla_domain::effect::Mutation>,
    pub failure_effects: Vec<holla_domain::effect::Mutation>,
    pub id: String,
    pub title: String,
    pub command: String,
    pub branch: String,
    pub deps: Vec<usize>,
    pub optional: bool,
    pub fails: Option<String>,
    pub ok_lines: Vec<String>,
    pub fail_lines: Vec<String>,
    pub state: StepState,
    pub lines: Vec<String>,
}

impl PlanStep {
    pub fn new(id: &str, title: &str, command: &str, branch: &str, deps: &[usize]) -> Self {
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

    pub fn succeeds_with(mut self, effects: Vec<holla_domain::effect::Mutation>) -> Self {
        self.ok_lines = effects
            .iter()
            .flat_map(|effect| effect.lines(false))
            .collect();
        if effects
            .iter()
            .any(|effect| matches!(effect, holla_domain::effect::Mutation::UpgradeTool { .. }))
        {
            self.ok_lines.push("shims reshimmed".into());
        }
        self.success_effects = effects;
        self
    }
    pub fn fails_after(
        mut self,
        reason: &str,
        effects: Vec<holla_domain::effect::Mutation>,
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

    pub fn required(mut self) -> Self {
        self.optional = false;
        self
    }
    pub fn fails_with(mut self, reason: &str, lines: &[&str]) -> Self {
        self.fails = Some(reason.into());
        self.fail_lines = lines.iter().map(ToString::to_string).collect();
        self
    }
    pub fn lines(mut self, lines: &[&str]) -> Self {
        self.ok_lines = lines.iter().map(ToString::to_string).collect();
        self
    }
    pub fn policy_skipped(mut self, reason: &str) -> Self {
        self.optional = false;
        self.state = StepState::PolicySkipped(reason.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanEffect {
    RestartContainer(String),
    DockerCleanup,
    DiskReclaim,
    DebianUpgraded,
    GitSync,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanSpec {
    pub action_id: String,
    pub title: String,
    pub host: String,
    pub phrase: String,
    pub will_change: String,
    pub steps: Vec<PlanStep>,
    pub effect: Option<PlanEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    action_id: String,
    title: String,
    host: String,
    phrase: String,
    will_change: String,
    steps: Vec<PlanStep>,
    effect: Option<PlanEffect>,
    ran: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
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
    pub fn try_new(spec: PlanSpec) -> Result<Self, PlanError> {
        let mut ids = std::collections::BTreeSet::new();
        for (index, step) in spec.steps.iter().enumerate() {
            for effects in [&step.success_effects, &step.failure_effects] {
                let amounts: Result<Vec<_>, _> = effects
                    .iter()
                    .map(holla_domain::effect::Mutation::reclaimed_bytes)
                    .collect();
                let amounts = amounts.map_err(|_| PlanError::InvalidInventory)?;
                holla_domain::accounting::bytes(amounts)
                    .map_err(|_| PlanError::InvalidInventory)?;
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

    pub fn action_id(&self) -> &str {
        &self.action_id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn host(&self) -> &str {
        &self.host
    }
    pub fn phrase(&self) -> &str {
        &self.phrase
    }
    pub fn will_change(&self) -> &str {
        &self.will_change
    }
    pub fn steps(&self) -> &[PlanStep] {
        &self.steps
    }
    pub fn ran(&self) -> bool {
        self.ran
    }
    pub fn effect(&self) -> Option<&PlanEffect> {
        self.effect.as_ref()
    }

    pub fn included_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| !matches!(s.state, StepState::Excluded | StepState::PolicySkipped(_)))
            .count()
    }

    #[expect(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        reason = "Requested row is checked first; private validated dependency indexes are earlier rows and prefix length is at most the allocated vector length"
    )]
    pub fn blocked_by_exclusion(&self, i: usize) -> Option<String> {
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

    pub fn toggle(&mut self, i: usize) -> Result<String, PlanError> {
        let s = self.steps.get_mut(i).ok_or(PlanError::MissingStep)?;
        if self.ran {
            return Err(PlanError::AlreadyRan);
        }
        if !s.optional {
            return Err(PlanError::RequiredStep(s.title.clone()));
        }
        s.state = if matches!(s.state, StepState::Excluded) {
            StepState::Pending
        } else {
            StepState::Excluded
        };
        Ok(match s.state {
            StepState::Excluded => format!("Excluded {}", s.title),
            _ => format!("Included {}", s.title),
        })
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "Private Plan construction validates every dependency as an earlier row; execution changes only states and never indexes or vector length"
    )]
    pub fn run(&mut self) -> Result<(), PlanError> {
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
            if let Some(reason) = s.fails.take() {
                s.lines = std::mem::take(&mut s.fail_lines);
                s.state = StepState::Failed(reason);
            } else {
                s.lines = std::mem::take(&mut s.ok_lines);
                s.state = StepState::Succeeded;
            }
        }
        Ok(())
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

    pub fn facts(&self) -> Vec<PlanFact> {
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
