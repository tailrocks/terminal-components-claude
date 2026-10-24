use std::fmt;

use crate::catalogue::{IntentError, resolve_intent};
use crate::domain::action::{Action, Risk};
use crate::domain::args::{AcceptedInputs, InputError, InputField, InputSchema, SchemaError};
use crate::domain::host::{Environment, HostKind};
use crate::domain::mise::{MiseTask, Trust};
use crate::world::World;

const NEXTTEST_TASK_ID: &str = "test-nextest";
const NEXTTEST_DEFINITION_PATH: &str = "~/work/pave/mise.toml";
const NEXTTEST_TASK_BODY: &str = "cargo nextest run";
const NEXTTEST_ADAPTER_ID: &str = "nextest-filter-profile";
const NEXTTEST_ADAPTER_VERSION: u16 = 1;
const NEXTTEST_SCHEMA_VERSION: u16 = 1;
const FILTER_FIELD_ID: &str = "filter";
const PROFILE_FIELD_ID: &str = "profile";

/// Explicit producer declaration for the dedicated nextest-capable fixture task.
///
/// This producer is opt-in by exact task ID, definition path, and full task body. It does not
/// infer argument support from the existing `test` task or its display name.
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
    /// Returns the explicit filter/profile producer for `test-nextest`.
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

    /// Returns the expected task identity.
    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    /// Returns the exact task definition path bound by this producer.
    pub fn definition_path(&self) -> &str {
        &self.definition_path
    }

    /// Returns the validated input schema.
    pub fn schema(&self) -> &InputSchema {
        &self.schema
    }

    /// Validates submitted values and captures defaults into one accepted request.
    pub fn accept(&self, values: &[(&str, &str)]) -> Result<AcceptedInputs, InputError> {
        self.schema.accept(values)
    }
}

/// Returns the producer declaration for the dedicated nextest task fixture.
///
/// # Errors
/// Returns a schema validation error if the producer's built-in declaration is malformed.
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

/// Structured invocation data. `argv` is an argument vector, never shell text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    program: String,
    argv: Vec<String>,
    workdir: String,
    target: String,
    risk: Risk,
}

impl Invocation {
    /// Returns the executable name.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Returns the exact ordered argument vector.
    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    /// Returns the task's working directory.
    pub fn workdir(&self) -> &str {
        &self.workdir
    }

    /// Returns the action target bound by the current catalogue row.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns the action risk bound by the current catalogue row.
    pub fn risk(&self) -> Risk {
        self.risk
    }
}

/// Immutable, freshly bound invocation plan for one accepted request.
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
    /// Returns the structured invocation produced by the bound request.
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }

    /// Returns the request values accepted against the bound schema.
    pub fn accepted_inputs(&self) -> &AcceptedInputs {
        &self.accepted
    }
}

/// Safe, non-secret facts derived from a freshly revalidated invocation plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeReviewFacts {
    pub invocation: Invocation,
    pub action_title: String,
    pub host_name: String,
    pub environment: Environment,
    pub effect_revision: u64,
}

/// Explicit execution disposition of the deterministic simulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    NothingExecuted,
}

/// A modeled result from the simulator; it is not a process launch or execution claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulatedReport {
    pub invocation: Invocation,
    pub status: ExecutionStatus,
    pub notice: &'static str,
}

/// Constructs the immutable typed invocation plan for one current task row.
///
/// The action's canonical `command` remains unchanged. The producer owns the argument mapping,
/// including omission of an empty optional filter.
///
/// # Errors
/// Returns an error when the row is stale, untrusted, unsupported, or bound to a changed task,
/// context, schema, or accepted request.
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

/// Revalidates a plan and projects safe facts from its same structured argv vector.
///
/// # Errors
/// Returns an error when the action, task definition, trust, context, producer, schema, or
/// accepted request no longer matches the bound plan.
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

/// Revalidates a plan and returns a pure simulation report from its exact argv and target.
///
/// This function does not execute a process or mutate `World`.
///
/// # Errors
/// Returns an error when the action, task definition, trust, context, producer, schema, or
/// accepted request no longer matches the bound plan.
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
    if row.id != format!("task:{}", producer.task_id) {
        return Err(TaskArgsError::UnsupportedTask);
    }
    let task = current_task(world, producer)?;
    if task.id != producer.task_id
        || task.defined_in != producer.definition_path
        || task.command != producer.task_body
    {
        return Err(TaskArgsError::TaskDefinitionChanged);
    }
    if task.trust != Trust::Trusted {
        return Err(TaskArgsError::NeedsTrust);
    }

    let action = resolve_intent(world, row).map_err(map_intent_error)?;
    let expected_command = format!("mise run {}", producer.task_id);
    let expected_workdir = producer
        .definition_path
        .rsplit_once('/')
        .map_or_else(|| world.cwd.clone(), |(directory, _)| directory.to_owned());
    if action.id != format!("task:{}", producer.task_id)
        || action.command != expected_command
        || action.target != expected_workdir
        || action.workdir != expected_workdir
    {
        return Err(TaskArgsError::StaleAction);
    }

    Ok((
        action,
        TaskSnapshot::from(task),
        ContextSnapshot::from(world),
    ))
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

/// Errors from constructing or consuming a task-backed typed invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskArgsError {
    UnsupportedTask,
    TaskDefinitionChanged,
    AmbiguousTask,
    NeedsTrust,
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

#[cfg(test)]
mod tests {
    use super::{
        ExecutionStatus, InvocationPlan, TaskArgsError, TaskArgsProducer, nextest_review_facts,
        nextest_schema, nextest_task_producer, prepare_nextest_invocation,
        simulate_nextest_invocation,
    };
    use crate::catalogue;
    use crate::domain::mise::Trust;
    use crate::fixtures::nextest_args_world;
    use crate::world::World;

    const NEXTTEST_ROW_ID: &str = "task:test-nextest";

    fn action_for(world: &World, id: &str) -> Result<crate::domain::action::Action, TaskArgsError> {
        catalogue::catalogue(world)
            .into_iter()
            .find(|action| action.id == id)
            .ok_or(TaskArgsError::StaleAction)
    }

    fn argv_strings(invocation: &super::Invocation) -> Vec<&str> {
        invocation.argv().iter().map(String::as_str).collect()
    }

    fn plan_for(
        world: &World,
        producer: &TaskArgsProducer,
        values: &[(&str, &str)],
    ) -> Result<InvocationPlan, TaskArgsError> {
        let row = action_for(world, NEXTTEST_ROW_ID)?;
        let accepted = producer
            .accept(values)
            .map_err(|_| TaskArgsError::InvalidAcceptedRequest)?;
        prepare_nextest_invocation(world, &row, producer, &accepted)
    }

    fn base_case(
        values: &[(&str, &str)],
    ) -> Result<(World, TaskArgsProducer, InvocationPlan), TaskArgsError> {
        let world = nextest_args_world();
        let producer =
            nextest_task_producer().map_err(|_| TaskArgsError::InvalidAcceptedRequest)?;
        let plan = plan_for(&world, &producer, values)?;
        Ok((world, producer, plan))
    }

    #[test]
    fn dedicated_fixture_keeps_cargo_test_and_adds_explicit_nextest_task() {
        let world = nextest_args_world();
        assert!(
            world.mise.is_some(),
            "fixture should include mise task data"
        );
        if let Some(mise) = &world.mise {
            let existing_test = mise.tasks.iter().find(|task| task.id == "test");
            let nextest = mise.tasks.iter().find(|task| task.id == "test-nextest");
            assert_eq!(
                existing_test.map(|task| task.command.as_str()),
                Some("cargo test --workspace")
            );
            assert_eq!(
                nextest.map(|task| task.command.as_str()),
                Some("cargo nextest run")
            );
            assert_eq!(
                nextest.map(|task| task.defined_in.as_str()),
                Some("~/work/pave/mise.toml")
            );
        }
    }

    #[test]
    fn one_bound_plan_supplies_exact_argv_to_review_and_simulator() {
        let case = base_case(&[("filter", "test(tests::parser)"), ("profile", "default")]);
        assert!(case.is_ok(), "dedicated task should produce a bound plan");
        if let Ok((world, producer, plan)) = case {
            let invocation = plan.invocation();
            assert_eq!(invocation.program(), "mise");
            assert_eq!(
                argv_strings(invocation),
                vec![
                    "run",
                    "test-nextest",
                    "--filterset=test(tests::parser)",
                    "-P",
                    "default"
                ]
            );
            assert_eq!(invocation.target(), "~/work/pave");
            assert_eq!(invocation.workdir(), "~/work/pave");
            assert_eq!(invocation.risk(), crate::domain::action::Risk::ReadOnly);

            let review = nextest_review_facts(&world, &producer, &plan);
            assert!(review.is_ok(), "bound plan should revalidate for review");
            if let Ok(review) = review {
                assert_eq!(review.invocation, *invocation);
                assert_eq!(review.host_name, "devbox");
                assert_eq!(review.effect_revision, 0);
            }

            let report = simulate_nextest_invocation(&world, &producer, &plan);
            assert!(
                report.is_ok(),
                "bound plan should revalidate for simulation"
            );
            if let Ok(report) = report {
                assert_eq!(report.invocation, *invocation);
                assert_eq!(report.status, ExecutionStatus::NothingExecuted);
                assert!(report.notice.contains("nothing was executed"));
            }
            assert_eq!(world.effect_revision, 0);
            assert!(world.activities.is_empty());
        }
    }

    #[test]
    fn empty_optional_filter_is_accepted_and_omits_filter_operands() {
        let case = base_case(&[("filter", ""), ("profile", "default")]);
        assert!(
            case.is_ok(),
            "empty optional filter should remain an accepted request"
        );
        if let Ok((world, producer, plan)) = case {
            assert_eq!(plan.accepted_inputs().value("filter"), Some(""));
            assert_eq!(
                argv_strings(plan.invocation()),
                vec!["run", "test-nextest", "-P", "default"]
            );
            let review = nextest_review_facts(&world, &producer, &plan);
            assert!(review.is_ok());
            if let Ok(review) = review {
                assert_eq!(review.invocation, *plan.invocation());
            }
        }

        let omitted = base_case(&[]);
        assert!(
            omitted.is_ok(),
            "omitted optional filter still accepts a request"
        );
        if let Ok((world, producer, plan)) = omitted {
            assert!(!plan.accepted_inputs().contains("filter"));
            assert_eq!(plan.accepted_inputs().value("profile"), Some("default"));
            assert_eq!(
                argv_strings(plan.invocation()),
                vec!["run", "test-nextest", "-P", "default"]
            );
            assert!(nextest_review_facts(&world, &producer, &plan).is_ok());
        }
    }

    #[test]
    fn filter_and_profile_edits_change_argv_and_review_facts_without_rewriting_action() {
        let first = base_case(&[("filter", "test(tests::old)"), ("profile", "ci")]);
        let second = base_case(&[("filter", "test(tests::new)"), ("profile", "default")]);
        assert!(first.is_ok());
        assert!(second.is_ok());
        if let (Ok((world_a, producer_a, plan_a)), Ok((world_b, producer_b, plan_b))) =
            (first, second)
        {
            assert_ne!(
                argv_strings(plan_a.invocation()),
                argv_strings(plan_b.invocation())
            );
            let review_a = nextest_review_facts(&world_a, &producer_a, &plan_a);
            let review_b = nextest_review_facts(&world_b, &producer_b, &plan_b);
            assert!(review_a.is_ok());
            assert!(review_b.is_ok());
            if let (Ok(review_a), Ok(review_b)) = (review_a, review_b) {
                assert_ne!(
                    argv_strings(&review_a.invocation),
                    argv_strings(&review_b.invocation)
                );
                assert_eq!(review_a.invocation, *plan_a.invocation());
                assert_eq!(review_b.invocation, *plan_b.invocation());
            }
            let row = action_for(&world_a, NEXTTEST_ROW_ID);
            assert_eq!(
                row.map(|row| row.command),
                Ok("mise run test-nextest".to_owned())
            );
        }
    }

    #[test]
    fn filter_operand_boundary_preserves_separator_options_and_shell_text_as_data() {
        for filter in [
            ":::",
            "-E test(name)",
            "suite name; $(touch /tmp/no-execution)",
        ] {
            let case = base_case(&[("filter", filter), ("profile", "ci")]);
            assert!(case.is_ok(), "filter text should remain accepted data");
            if let Ok((_, _, plan)) = case {
                let argv = plan.invocation().argv();
                assert_eq!(argv.len(), 5);
                assert_eq!(argv[0], "run");
                assert_eq!(argv[1], "test-nextest");
                assert_eq!(argv[2], format!("--filterset={filter}"));
                assert_eq!(argv[3], "-P");
                assert_eq!(argv[4], "ci");
                assert_eq!(plan.invocation().target(), "~/work/pave");
            }
        }
    }

    #[test]
    fn invalid_choice_and_current_cargo_test_task_do_not_get_nextest_support() {
        let world = nextest_args_world();
        let producer_result = nextest_task_producer();
        assert!(producer_result.is_ok());
        if let Ok(producer) = producer_result {
            assert_eq!(
                producer.accept(&[("profile", "nightly")]),
                Err(crate::domain::args::InputError::InvalidChoice(
                    "profile".to_owned()
                ))
            );
            let row = action_for(&world, "task:test");
            assert!(row.is_ok());
            if let Ok(row) = row {
                let accepted = producer.accept(&[("profile", "ci")]);
                assert!(accepted.is_ok());
                if let Ok(accepted) = accepted {
                    assert_eq!(
                        prepare_nextest_invocation(&world, &row, &producer, &accepted),
                        Err(TaskArgsError::UnsupportedTask)
                    );
                }
            }
        }
    }

    #[test]
    fn task_body_path_identity_and_schema_changes_reject_the_bound_plan() {
        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((mut world, producer, plan)) = case {
            if let Some(mise) = &mut world.mise
                && let Some(task) = mise.tasks.iter_mut().find(|task| task.id == "test-nextest")
            {
                task.command = "cargo nextest run --workspace".to_owned();
            }
            assert_eq!(
                nextest_review_facts(&world, &producer, &plan),
                Err(TaskArgsError::TaskDefinitionChanged)
            );
        }

        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((mut world, producer, plan)) = case {
            if let Some(mise) = &mut world.mise
                && let Some(task) = mise.tasks.iter_mut().find(|task| task.id == "test-nextest")
            {
                task.defined_in = "~/work/other/mise.toml".to_owned();
            }
            assert_eq!(
                nextest_review_facts(&world, &producer, &plan),
                Err(TaskArgsError::TaskDefinitionChanged)
            );
        }

        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((mut world, producer, plan)) = case {
            if let Some(mise) = &mut world.mise
                && let Some(task) = mise.tasks.iter_mut().find(|task| task.id == "test-nextest")
            {
                task.id = "test-nextest-renamed".to_owned();
            }
            assert_eq!(
                nextest_review_facts(&world, &producer, &plan),
                Err(TaskArgsError::TaskDefinitionChanged)
            );
        }

        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((world, mut producer, plan)) = case {
            let changed_schema = nextest_schema(2);
            assert!(changed_schema.is_ok());
            if let Ok(schema) = changed_schema {
                producer.schema = schema;
                assert_eq!(
                    nextest_review_facts(&world, &producer, &plan),
                    Err(TaskArgsError::StaleSchema)
                );
            }
        }

        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((world, mut producer, plan)) = case {
            producer.adapter_version = producer.adapter_version.saturating_add(1);
            assert_eq!(
                nextest_review_facts(&world, &producer, &plan),
                Err(TaskArgsError::StaleProducer)
            );
        }
    }

    #[test]
    fn trust_and_context_changes_reject_review_and_simulation() {
        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((mut world, producer, plan)) = case {
            if let Some(mise) = &mut world.mise
                && let Some(task) = mise.tasks.iter_mut().find(|task| task.id == "test-nextest")
            {
                task.trust = Trust::Untrusted;
            }
            assert_eq!(
                nextest_review_facts(&world, &producer, &plan),
                Err(TaskArgsError::NeedsTrust)
            );
        }

        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((mut world, producer, plan)) = case {
            world.cwd = "~/work/other".to_owned();
            assert_eq!(
                simulate_nextest_invocation(&world, &producer, &plan),
                Err(TaskArgsError::StaleContext)
            );
        }

        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((mut world, producer, plan)) = case {
            world.host.name = "other-host".to_owned();
            assert_eq!(
                nextest_review_facts(&world, &producer, &plan),
                Err(TaskArgsError::StaleContext)
            );
        }

        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((mut world, producer, plan)) = case {
            world.effect_revision = world.effect_revision.saturating_add(1);
            assert_eq!(
                nextest_review_facts(&world, &producer, &plan),
                Err(TaskArgsError::StaleContext)
            );
        }
    }

    #[test]
    fn duplicate_task_identity_is_rejected() {
        let case = base_case(&[("profile", "ci")]);
        assert!(case.is_ok());
        if let Ok((mut world, producer, plan)) = case {
            if let Some(mise) = &mut world.mise {
                let duplicate = mise
                    .tasks
                    .iter()
                    .find(|task| task.id == "test-nextest")
                    .cloned();
                if let Some(task) = duplicate {
                    mise.tasks.push(task.clone());
                }
            }
            assert_eq!(
                nextest_review_facts(&world, &producer, &plan),
                Err(TaskArgsError::AmbiguousTask)
            );
        }
    }
}
