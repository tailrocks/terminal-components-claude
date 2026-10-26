use std::fmt;

use crate::catalogue::{IntentError, resolve_intent};
use crate::domain::action::{Action, ActionKind, Availability, Risk};
use crate::domain::args::{AcceptedInputs, InputError, InputField, InputSchema, SchemaError};
use crate::domain::host::{Environment, HostKind};
use crate::domain::mise::{MiseTask, Trust};
use crate::world::World;

const NEXTTEST_TASK_ID: &str = "test-nextest";
const NEXTTEST_ROW_ID: &str = "task:test-nextest";
const NEXTTEST_DEFINITION_PATH: &str = "~/work/pave/mise.toml";
const NEXTTEST_TASK_BODY: &str = "cargo nextest run";
const NEXTTEST_ADAPTER_ID: &str = "nextest-filter-profile";
const NEXTTEST_ADAPTER_VERSION: u16 = 1;
const NEXTTEST_SCHEMA_VERSION: u16 = 1;
const FILTER_FIELD_ID: &str = "filter";
const PROFILE_FIELD_ID: &str = "profile";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskArgsProducer {
    task_id: String,
    definition_path: String,
    task_body: String,
    adapter_id: String,
    adapter_version: u16,
    schema: InputSchema,
}

impl TaskArgsProducer {
    pub fn nextest_fixture() -> Result<Self, SchemaError> {
        let schema = nextest_schema(NEXTTEST_SCHEMA_VERSION)?;
        Ok(Self {
            task_id: NEXTTEST_TASK_ID.to_owned(),
            definition_path: NEXTTEST_DEFINITION_PATH.to_owned(),
            task_body: NEXTTEST_TASK_BODY.to_owned(),
            adapter_id: NEXTTEST_ADAPTER_ID.to_owned(),
            adapter_version: NEXTTEST_ADAPTER_VERSION,
            schema,
        })
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn definition_path(&self) -> &str {
        &self.definition_path
    }

    pub fn schema(&self) -> &InputSchema {
        &self.schema
    }

    pub fn accept(&self, values: &[(&str, &str)]) -> Result<AcceptedInputs, InputError> {
        self.schema.accept(values)
    }
}

pub fn nextest_task_producer() -> Result<TaskArgsProducer, SchemaError> {
    TaskArgsProducer::nextest_fixture()
}

fn nextest_schema(version: u16) -> Result<InputSchema, SchemaError> {
    let filter = InputField::text(FILTER_FIELD_ID, "Filterset expression", false)?.with_help(
        "Optional nextest filterset expression; passed through unchanged and not parsed here",
    );
    let profile = InputField::choice(PROFILE_FIELD_ID, "Profile", &["default", "ci"], true)?
        .with_default("default");
    InputSchema::new(version, vec![filter, profile])
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProducerSnapshot {
    task_id: String,
    definition_path: String,
    task_body: String,
    adapter_id: String,
    adapter_version: u16,
    schema: InputSchema,
}

impl From<&TaskArgsProducer> for ProducerSnapshot {
    fn from(producer: &TaskArgsProducer) -> Self {
        Self {
            task_id: producer.task_id.clone(),
            definition_path: producer.definition_path.clone(),
            task_body: producer.task_body.clone(),
            adapter_id: producer.adapter_id.clone(),
            adapter_version: producer.adapter_version,
            schema: producer.schema.clone(),
        }
    }
}

impl ProducerSnapshot {
    fn same_adapter_binding(&self, producer: &TaskArgsProducer) -> bool {
        self.task_id == producer.task_id
            && self.definition_path == producer.definition_path
            && self.task_body == producer.task_body
            && self.adapter_id == producer.adapter_id
            && self.adapter_version == producer.adapter_version
    }
}

pub fn task_args_producer_for(row: &Action) -> Result<Option<TaskArgsProducer>, SchemaError> {
    if row.id == NEXTTEST_ROW_ID || row.intent_id() == Some(NEXTTEST_ROW_ID) {
        nextest_task_producer().map(Some)
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TaskSnapshot {
    id: String,
    definition_path: String,
    body: String,
    trust: Trust,
}

impl From<&MiseTask> for TaskSnapshot {
    fn from(task: &MiseTask) -> Self {
        Self {
            id: task.id.clone(),
            definition_path: task.defined_in.clone(),
            body: task.command.clone(),
            trust: task.trust,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextSnapshot {
    host_name: String,
    host_kind: HostKind,
    environment: Environment,
    cwd: String,
    effect_revision: u64,
}

impl From<&World> for ContextSnapshot {
    fn from(world: &World) -> Self {
        Self {
            host_name: world.host.name.clone(),
            host_kind: world.host.kind,
            environment: world.host.env,
            cwd: world.cwd.clone(),
            effect_revision: world.effect_revision,
        }
    }
}

pub struct TaskArgsSession {
    producer: ProducerSnapshot,
    task: TaskSnapshot,
    action: Action,
    context: ContextSnapshot,
    initially_trusted: bool,
    trust_approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskArgsAcceptError {
    Binding(TaskArgsError),
    Input(InputError),
}

impl fmt::Display for TaskArgsAcceptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binding(error) => fmt::Display::fmt(error, f),
            Self::Input(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl std::error::Error for TaskArgsAcceptError {}

impl TaskArgsSession {
    pub fn begin(
        world: &World,
        row: &Action,
        producer: &TaskArgsProducer,
    ) -> Result<Self, TaskArgsError> {
        let (normalized, task) = canonical_task_binding(world, row, producer)?;
        let initially_trusted = task.trust == Trust::Trusted;
        Ok(Self {
            producer: ProducerSnapshot::from(producer),
            task,
            action: normalized,
            context: ContextSnapshot::from(world),
            initially_trusted,
            trust_approved: initially_trusted,
        })
    }

    pub fn definition_path(&self) -> &str {
        &self.task.definition_path
    }

    pub fn approve_trust(
        &mut self,
        world: &mut World,
        row: &Action,
        producer: &TaskArgsProducer,
        file: &str,
    ) -> Result<Action, TaskArgsError> {
        if self.initially_trusted || self.trust_approved || file != self.task.definition_path {
            return Err(TaskArgsError::TrustTransitionChanged);
        }
        self.validate_snapshot(world, row, producer, Trust::Untrusted)?;
        if !world.trust_mise_file(file) {
            return Err(TaskArgsError::TrustTransitionChanged);
        }
        let current_row = current_row_by_id(world, &row.id)?;
        let (current_action, current_task) = canonical_task_binding(world, &current_row, producer)?;
        if current_task.trust != Trust::Trusted {
            return Err(TaskArgsError::TrustTransitionChanged);
        }
        if current_task.definition_path != file || current_action != self.action {
            return Err(TaskArgsError::TrustTransitionChanged);
        }
        if ContextSnapshot::from(&*world) != self.context {
            return Err(TaskArgsError::StaleContext);
        }
        self.trust_approved = true;
        Ok(current_action)
    }

    pub fn accept(
        &self,
        world: &World,
        row: &Action,
        producer: &TaskArgsProducer,
        values: &[(&str, &str)],
    ) -> Result<InvocationPlan, TaskArgsAcceptError> {
        self.validate_current(world, row, producer)
            .map_err(TaskArgsAcceptError::Binding)?;
        let accepted = producer
            .accept(values)
            .map_err(TaskArgsAcceptError::Input)?;
        prepare_nextest_invocation(world, row, producer, &accepted)
            .map_err(TaskArgsAcceptError::Binding)
    }

    fn validate_current(
        &self,
        world: &World,
        row: &Action,
        producer: &TaskArgsProducer,
    ) -> Result<(), TaskArgsError> {
        if !self.trust_approved {
            return Err(TaskArgsError::NeedsTrust);
        }
        self.validate_snapshot(world, row, producer, Trust::Trusted)
    }

    fn validate_snapshot(
        &self,
        world: &World,
        row: &Action,
        producer: &TaskArgsProducer,
        expected_trust: Trust,
    ) -> Result<(), TaskArgsError> {
        if !self.producer.same_adapter_binding(producer) || self.producer.schema != producer.schema
        {
            return Err(TaskArgsError::StaleProducer);
        }
        if ContextSnapshot::from(world) != self.context {
            return Err(TaskArgsError::StaleContext);
        }
        let (normalized, task) = canonical_task_binding(world, row, producer)?;
        if task.trust != expected_trust {
            return Err(if expected_trust == Trust::Trusted {
                TaskArgsError::TrustTransitionChanged
            } else {
                TaskArgsError::NeedsTrust
            });
        }
        if normalized != self.action {
            return Err(TaskArgsError::StaleAction);
        }
        if self.task.id != task.id
            || self.task.definition_path != task.definition_path
            || self.task.body != task.body
            || (self.initially_trusted && self.task.trust != task.trust)
        {
            return Err(TaskArgsError::TaskDefinitionChanged);
        }
        Ok(())
    }
}

fn ensure_task_binding(task: &MiseTask, producer: &TaskArgsProducer) -> Result<(), TaskArgsError> {
    if task.id != producer.task_id
        || task.defined_in != producer.definition_path
        || task.command != producer.task_body
    {
        return Err(TaskArgsError::TaskDefinitionChanged);
    }
    Ok(())
}

fn current_row_by_id(world: &World, row_id: &str) -> Result<Action, TaskArgsError> {
    let mut matching = crate::catalogue::catalogue(world)
        .into_iter()
        .filter(|action| action.id == row_id);
    let Some(row) = matching.next() else {
        return Err(TaskArgsError::StaleAction);
    };
    if matching.next().is_some() {
        return Err(TaskArgsError::StaleAction);
    }
    Ok(row)
}

fn canonical_task_binding(
    world: &World,
    row: &Action,
    producer: &TaskArgsProducer,
) -> Result<(Action, TaskSnapshot), TaskArgsError> {
    let declared = nextest_task_producer().map_err(|_| TaskArgsError::StaleProducer)?;
    if producer != &declared {
        return Err(TaskArgsError::StaleProducer);
    }

    let task = current_task(world, producer)?;
    ensure_task_binding(task, producer)?;

    let expected_id = format!("task:{}", producer.task_id);
    if row.intent_id() != Some(expected_id.as_str()) {
        return Err(TaskArgsError::UnsupportedTask);
    }
    let current_row = current_row_by_id(world, &row.id)?;
    if current_row.intent() != row.intent()
        || current_row.command != row.command
        || current_row.target != row.target
        || current_row.workdir != row.workdir
    {
        return Err(TaskArgsError::StaleAction);
    }

    let resolved = match resolve_intent(world, row) {
        Ok(action) => action,
        Err(IntentError::NeedsTrust) if task.trust == Trust::Untrusted => {
            unique_canonical_task_action(world, &expected_id)?
        }
        Err(error) => return Err(map_intent_error(error)),
    };
    let canonical = unique_canonical_task_action(world, &expected_id)?;
    if resolved != canonical {
        return Err(TaskArgsError::StaleAction);
    }

    let expected_command = format!("mise run {}", producer.task_id);
    let expected_workdir = producer
        .definition_path
        .rsplit_once('/')
        .map_or_else(|| world.cwd.clone(), |(directory, _)| directory.to_owned());
    if canonical.id != expected_id
        || canonical.intent_id() != Some(expected_id.as_str())
        || canonical.kind != ActionKind::Task
        || canonical.command != expected_command
        || canonical.target != expected_workdir
        || canonical.workdir != expected_workdir
    {
        return Err(TaskArgsError::StaleAction);
    }

    let normalized = normalized_task_action(&canonical, task.trust, &task.defined_in)?;
    Ok((normalized, TaskSnapshot::from(task)))
}

fn unique_canonical_task_action(world: &World, expected_id: &str) -> Result<Action, TaskArgsError> {
    let mut matching = crate::catalogue::catalogue(world)
        .into_iter()
        .filter(|action| action.id == expected_id);
    let Some(action) = matching.next() else {
        return Err(TaskArgsError::StaleAction);
    };
    if matching.next().is_some() || action.intent_id() != Some(expected_id) {
        return Err(TaskArgsError::StaleAction);
    }
    Ok(action)
}

fn normalized_task_action(
    action: &Action,
    trust: Trust,
    definition_path: &str,
) -> Result<Action, TaskArgsError> {
    const UNTRUSTED_SUFFIX: &str = " · untrusted config";
    let mut normalized = action.clone();
    if action.risk != Risk::ReadOnly {
        return Err(TaskArgsError::UnsupportedRisk);
    }
    match trust {
        Trust::Trusted => {
            if action.availability != Availability::Ready {
                return Err(TaskArgsError::StaleAction);
            }
        }
        Trust::Untrusted => {
            let Availability::NeedsTrust(file) = &action.availability else {
                return Err(TaskArgsError::NeedsTrust);
            };
            if file != definition_path || !action.reason.ends_with(UNTRUSTED_SUFFIX) {
                return Err(TaskArgsError::StaleAction);
            }
            normalized.reason.truncate(
                normalized
                    .reason
                    .len()
                    .saturating_sub(UNTRUSTED_SUFFIX.len()),
            );
            normalized.availability = Availability::Ready;
        }
    }
    Ok(normalized)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    program: String,
    argv: Vec<String>,
    workdir: String,
    target: String,
    risk: Risk,
}

impl Invocation {
    pub(crate) fn read_only(program: &str, argv: Vec<String>, workdir: &str, target: &str) -> Self {
        Self {
            program: program.to_owned(),
            argv,
            workdir: workdir.to_owned(),
            target: target.to_owned(),
            risk: Risk::ReadOnly,
        }
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    pub fn workdir(&self) -> &str {
        &self.workdir
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn risk(&self) -> Risk {
        self.risk
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvocationPlan {
    action: Action,
    task: TaskSnapshot,
    context: ContextSnapshot,
    producer: ProducerSnapshot,
    accepted: AcceptedInputs,
    invocation: Invocation,
}

impl InvocationPlan {
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }

    pub fn accepted_inputs(&self) -> &AcceptedInputs {
        &self.accepted
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeReviewFacts {
    pub invocation: Invocation,
    pub action_title: String,
    pub host_name: String,
    pub environment: Environment,
    pub effect_revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    NothingExecuted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulatedReport {
    pub invocation: Invocation,
    pub status: ExecutionStatus,
    pub notice: &'static str,
}

pub fn prepare_nextest_invocation(
    world: &World,
    row: &Action,
    producer: &TaskArgsProducer,
    accepted: &AcceptedInputs,
) -> Result<InvocationPlan, TaskArgsError> {
    if accepted.schema() != &producer.schema {
        return Err(TaskArgsError::StaleSchema);
    }
    let (action, task, context) = bind_current(world, row, producer)?;
    let invocation = invocation_for(&action, producer, accepted)?;
    Ok(InvocationPlan {
        action,
        task,
        context,
        producer: ProducerSnapshot::from(producer),
        accepted: accepted.clone(),
        invocation,
    })
}

pub fn nextest_review_facts(
    world: &World,
    producer: &TaskArgsProducer,
    plan: &InvocationPlan,
) -> Result<SafeReviewFacts, TaskArgsError> {
    validate_plan(world, producer, plan)?;
    Ok(SafeReviewFacts {
        invocation: plan.invocation.clone(),
        action_title: plan.action.title.clone(),
        host_name: plan.context.host_name.clone(),
        environment: plan.context.environment,
        effect_revision: plan.context.effect_revision,
    })
}

pub fn simulate_nextest_invocation(
    world: &World,
    producer: &TaskArgsProducer,
    plan: &InvocationPlan,
) -> Result<SimulatedReport, TaskArgsError> {
    validate_plan(world, producer, plan)?;
    Ok(SimulatedReport {
        invocation: plan.invocation.clone(),
        status: ExecutionStatus::NothingExecuted,
        notice: "simulated invocation only · nothing was executed",
    })
}

fn bind_current(
    world: &World,
    row: &Action,
    producer: &TaskArgsProducer,
) -> Result<(Action, TaskSnapshot, ContextSnapshot), TaskArgsError> {
    let (action, task) = canonical_task_binding(world, row, producer)?;
    if task.trust != Trust::Trusted {
        return Err(TaskArgsError::NeedsTrust);
    }

    Ok((action, task, ContextSnapshot::from(world)))
}

fn current_task<'a>(
    world: &'a World,
    producer: &TaskArgsProducer,
) -> Result<&'a MiseTask, TaskArgsError> {
    let Some(mise) = &world.mise else {
        return Err(TaskArgsError::TaskDefinitionChanged);
    };
    let mut matching = mise.tasks.iter().filter(|task| task.id == producer.task_id);
    let Some(task) = matching.next() else {
        return Err(TaskArgsError::TaskDefinitionChanged);
    };
    if matching.next().is_some() {
        return Err(TaskArgsError::AmbiguousTask);
    }
    Ok(task)
}

fn invocation_for(
    action: &Action,
    producer: &TaskArgsProducer,
    accepted: &AcceptedInputs,
) -> Result<Invocation, TaskArgsError> {
    let Some(profile) = accepted.value(PROFILE_FIELD_ID) else {
        return Err(TaskArgsError::InvalidAcceptedRequest);
    };
    let mut argv = vec!["run".to_owned(), producer.task_id.clone()];
    if let Some(filter) = accepted.value(FILTER_FIELD_ID)
        && !filter.is_empty()
    {
        argv.push(format!("--filterset={filter}"));
    }
    argv.push("-P".to_owned());
    argv.push(profile.to_owned());

    Ok(Invocation {
        program: "mise".to_owned(),
        argv,
        workdir: action.workdir.clone(),
        target: action.target.clone(),
        risk: action.risk,
    })
}

fn validate_plan(
    world: &World,
    producer: &TaskArgsProducer,
    plan: &InvocationPlan,
) -> Result<(), TaskArgsError> {
    if !plan.producer.same_adapter_binding(producer) {
        return Err(TaskArgsError::StaleProducer);
    }
    if plan.producer.schema != producer.schema || plan.accepted.schema() != &producer.schema {
        return Err(TaskArgsError::StaleSchema);
    }
    if ContextSnapshot::from(world) != plan.context {
        return Err(TaskArgsError::StaleContext);
    }

    let (action, task, context) = bind_current(world, &plan.action, producer)?;
    if action != plan.action || task != plan.task || context != plan.context {
        return Err(TaskArgsError::StaleBinding);
    }
    if invocation_for(&action, producer, &plan.accepted)? != plan.invocation {
        return Err(TaskArgsError::StaleRequest);
    }
    Ok(())
}

fn map_intent_error(error: IntentError) -> TaskArgsError {
    match error {
        IntentError::NeedsTrust => TaskArgsError::NeedsTrust,
        IntentError::StaleTarget | IntentError::Unavailable => TaskArgsError::StaleAction,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskArgsError {
    UnsupportedTask,
    TaskDefinitionChanged,
    AmbiguousTask,
    NeedsTrust,
    TrustTransitionChanged,
    UnsupportedRisk,
    StaleAction,
    StaleContext,
    StaleProducer,
    StaleSchema,
    StaleBinding,
    StaleRequest,
    InvalidAcceptedRequest,
}

impl fmt::Display for TaskArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTask => f.write_str("task does not declare this input producer"),
            Self::TaskDefinitionChanged => f.write_str("task definition changed · review again"),
            Self::AmbiguousTask => f.write_str("task identity is ambiguous"),
            Self::NeedsTrust => f.write_str("task file needs trust before accepting inputs"),
            Self::TrustTransitionChanged => {
                f.write_str("task Trust state changed · start the input review again")
            }
            Self::UnsupportedRisk => {
                f.write_str("task input producer requires a read-only action policy")
            }
            Self::StaleAction => f.write_str("task action changed · review again"),
            Self::StaleContext => f.write_str("host or working context changed · review again"),
            Self::StaleProducer => f.write_str("task input producer changed · review again"),
            Self::StaleSchema => f.write_str("task input schema changed · review again"),
            Self::StaleBinding => f.write_str("task binding changed · review again"),
            Self::StaleRequest => f.write_str("accepted task inputs changed · review again"),
            Self::InvalidAcceptedRequest => {
                f.write_str("accepted request is missing a required producer value")
            }
        }
    }
}

impl std::error::Error for TaskArgsError {}
