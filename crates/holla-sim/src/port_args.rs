use std::fmt;

use crate::catalogue::{self, IntentError};
use crate::domain::action::{Action, ActionKind, Availability, Risk, Scope};
use crate::domain::args::{AcceptedInputs, InputError, InputField, InputSchema, SchemaError};
use crate::domain::host::{Environment, HostKind};
use crate::listeners::{
    self, ListenerFacts, ListenerModel, ListenerSuggestion, ModeledPortError, ModeledPortReport,
};
use crate::task_args::{ExecutionStatus, Invocation, SimulatedReport};
use crate::world::World;

const ACTION_ID: &str = "system.port";
const ACTION_TITLE: &str = "Inspect port (simulated)";
const ACTION_COMMAND: &str = "lsof -nP -iTCP:<port> -sTCP:LISTEN";
const ADAPTER_ID: &str = "port-lsof";
const ADAPTER_VERSION: u16 = 1;
const SCHEMA_VERSION: u16 = 1;
const PORT_FIELD_ID: &str = "port";
const PORT_FIELD_LABEL: &str = "TCP port";
const PORT_FIELD_HELP: &str =
    "Required text; malformed non-empty input preserves the historical fallback to 5173";
const LEGACY_PORT: u16 = 5173;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortArgsProducer {
    adapter_id: String,
    adapter_version: u16,
    schema: InputSchema,
}

impl PortArgsProducer {
    pub fn new() -> Result<Self, SchemaError> {
        let field = InputField::text(PORT_FIELD_ID, PORT_FIELD_LABEL, true)?
            .with_help(PORT_FIELD_HELP)
            .with_default("5173");
        let schema = InputSchema::new(SCHEMA_VERSION, vec![field])?;
        Ok(Self {
            adapter_id: ADAPTER_ID.to_owned(),
            adapter_version: ADAPTER_VERSION,
            schema,
        })
    }

    pub fn schema(&self) -> &InputSchema {
        &self.schema
    }

    fn same_binding(&self, other: &Self) -> bool {
        self.adapter_id == other.adapter_id
            && self.adapter_version == other.adapter_version
            && self.schema == other.schema
    }
}

pub fn port_args_producer() -> Result<PortArgsProducer, SchemaError> {
    PortArgsProducer::new()
}

pub(crate) fn catalogue_action(world: &World) -> Action {
    let mut action = Action::new(
        ACTION_ID,
        ACTION_TITLE,
        ActionKind::System,
        Scope::Host,
        "available on this host · simulated lookup only; listener output is not modeled",
        Risk::ReadOnly,
        &world.host.name,
        ACTION_COMMAND,
    );
    action.workdir.clone_from(&world.cwd);
    action.keywords =
        "port listen eaddrinuse address already in use lsof process on port TCP".to_owned();
    action
}

pub fn port_args_producer_for(row: &Action) -> Result<Option<PortArgsProducer>, SchemaError> {
    if row.id == ACTION_ID || row.intent_id() == Some(ACTION_ID) {
        port_args_producer().map(Some)
    } else {
        Ok(None)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortParseDisposition {
    Parsed,
    LegacyFallbackTo5173,
}

#[derive(Debug)]
pub struct PortArgsSession {
    producer: PortArgsProducer,
    row: Action,
    action: Action,
    context: ContextSnapshot,
    accepted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortArgsError {
    StaleAction,
    AmbiguousAction,
    UnsupportedPolicy,
    StaleContext,
    StaleProducer,
    StaleSchema,
    StaleBinding,
    StaleRequest,
}

impl fmt::Display for PortArgsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::StaleAction => "port action changed · review it again",
            Self::AmbiguousAction => "port action is ambiguous · review it again",
            Self::UnsupportedPolicy => "port action is not a ready read-only host action",
            Self::StaleContext => "port input context changed · review it again",
            Self::StaleProducer => "port input producer changed · review it again",
            Self::StaleSchema => "port input schema changed · review it again",
            Self::StaleBinding => "port input binding changed · review it again",
            Self::StaleRequest => "port input request changed · review it again",
        })
    }
}

impl std::error::Error for PortArgsError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortArgsAcceptError {
    Binding(PortArgsError),
    Input(InputError),
}

impl fmt::Display for PortArgsAcceptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binding(error) => fmt::Display::fmt(error, formatter),
            Self::Input(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for PortArgsAcceptError {}

impl PortArgsSession {
    pub fn begin(
        world: &World,
        row: &Action,
        producer: &PortArgsProducer,
    ) -> Result<Self, PortArgsError> {
        let action = canonical_binding(world, row)?;
        validate_producer(producer)?;
        Ok(Self {
            producer: producer.clone(),
            row: row.clone(),
            action,
            context: ContextSnapshot::from(world),
            accepted: false,
        })
    }

    pub fn accept(
        &mut self,
        world: &World,
        row: &Action,
        producer: &PortArgsProducer,
        values: &[(&str, &str)],
    ) -> Result<PortInvocationPlan, PortArgsAcceptError> {
        if self.accepted {
            return Err(PortArgsAcceptError::Binding(PortArgsError::StaleRequest));
        }
        validate_session(world, row, producer, self).map_err(PortArgsAcceptError::Binding)?;
        let accepted = producer
            .schema
            .accept(values)
            .map_err(PortArgsAcceptError::Input)?;
        let raw = accepted
            .value(PORT_FIELD_ID)
            .ok_or(PortArgsAcceptError::Binding(PortArgsError::StaleRequest))?;
        let (effective_port, disposition) = match raw.parse::<u16>() {
            Ok(port) => (port, PortParseDisposition::Parsed),
            Err(_) => (LEGACY_PORT, PortParseDisposition::LegacyFallbackTo5173),
        };
        let invocation = port_invocation(&self.action, effective_port);
        self.accepted = true;
        Ok(PortInvocationPlan {
            producer: self.producer.clone(),
            row: self.row.clone(),
            action: self.action.clone(),
            context: self.context.clone(),
            accepted,
            disposition,
            effective_port,
            invocation,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PortInvocationPlan {
    producer: PortArgsProducer,
    row: Action,
    action: Action,
    context: ContextSnapshot,
    accepted: AcceptedInputs,
    disposition: PortParseDisposition,
    effective_port: u16,
    invocation: Invocation,
}

impl PortInvocationPlan {
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }

    pub fn accepted_inputs(&self) -> &AcceptedInputs {
        &self.accepted
    }

    pub fn disposition(&self) -> PortParseDisposition {
        self.disposition
    }

    pub fn effective_port(&self) -> u16 {
        self.effective_port
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortReviewFacts {
    pub invocation: Invocation,
    pub action_title: String,
    pub host_name: String,
    pub environment: Environment,
    pub effect_revision: u64,
    pub disposition: PortParseDisposition,
    pub effective_port: u16,
}

pub fn port_review_facts(
    world: &World,
    producer: &PortArgsProducer,
    plan: &PortInvocationPlan,
) -> Result<PortReviewFacts, PortArgsError> {
    validate_plan(world, producer, plan)?;
    Ok(PortReviewFacts {
        invocation: plan.invocation.clone(),
        action_title: plan.action.title.clone(),
        host_name: plan.context.host_name.clone(),
        environment: plan.context.environment,
        effect_revision: plan.context.effect_revision,
        disposition: plan.disposition,
        effective_port: plan.effective_port,
    })
}

pub fn simulate_port_invocation(
    world: &World,
    producer: &PortArgsProducer,
    plan: &PortInvocationPlan,
) -> Result<SimulatedReport, PortArgsError> {
    validate_plan(world, producer, plan)?;
    Ok(SimulatedReport {
        invocation: plan.invocation.clone(),
        status: ExecutionStatus::NothingExecuted,
        notice: "simulated invocation only · nothing was executed",
    })
}

fn validate_producer(producer: &PortArgsProducer) -> Result<(), PortArgsError> {
    let expected = port_args_producer().map_err(|_| PortArgsError::StaleSchema)?;
    if producer.adapter_id != ADAPTER_ID || producer.adapter_version != ADAPTER_VERSION {
        return Err(PortArgsError::StaleProducer);
    }
    if producer.schema != expected.schema {
        return Err(PortArgsError::StaleSchema);
    }
    Ok(())
}

fn canonical_binding(world: &World, row: &Action) -> Result<Action, PortArgsError> {
    let actions = catalogue::catalogue(world);
    let mut raw_matches = actions.iter().filter(|candidate| candidate.id == row.id);
    let Some(current_row) = raw_matches.next() else {
        return Err(PortArgsError::StaleAction);
    };
    if raw_matches.next().is_some() {
        return Err(PortArgsError::AmbiguousAction);
    }
    if current_row.intent() != row.intent()
        || current_row.command != row.command
        || current_row.target != row.target
        || current_row.workdir != row.workdir
    {
        return Err(PortArgsError::StaleAction);
    }
    let resolved = catalogue::resolve_intent(world, row).map_err(map_intent_error)?;
    let mut canonical_matches = actions.iter().filter(|candidate| candidate.id == ACTION_ID);
    let Some(canonical) = canonical_matches.next() else {
        return Err(PortArgsError::StaleAction);
    };
    if canonical_matches.next().is_some() {
        return Err(PortArgsError::AmbiguousAction);
    }
    if resolved != *canonical || canonical.intent_id() != Some(ACTION_ID) {
        return Err(PortArgsError::StaleAction);
    }
    if canonical.title != ACTION_TITLE
        || canonical.kind != ActionKind::System
        || canonical.scope != Scope::Host
        || canonical.risk != Risk::ReadOnly
        || canonical.availability != Availability::Ready
        || canonical.target != world.host.name
        || canonical.workdir != world.cwd
        || canonical.command != ACTION_COMMAND
    {
        return Err(PortArgsError::UnsupportedPolicy);
    }
    Ok(canonical.clone())
}

fn map_intent_error(error: IntentError) -> PortArgsError {
    match error {
        IntentError::StaleTarget | IntentError::Unavailable => PortArgsError::StaleAction,
        IntentError::NeedsTrust => PortArgsError::UnsupportedPolicy,
    }
}

fn validate_session(
    world: &World,
    row: &Action,
    producer: &PortArgsProducer,
    session: &PortArgsSession,
) -> Result<(), PortArgsError> {
    validate_producer(producer)?;
    if !session.producer.same_binding(producer) {
        return Err(PortArgsError::StaleProducer);
    }
    if ContextSnapshot::from(world) != session.context {
        return Err(PortArgsError::StaleContext);
    }
    let captured_action = canonical_binding(world, &session.row)?;
    if captured_action != session.action {
        return Err(PortArgsError::StaleBinding);
    }
    if row.id != session.row.id
        || row.intent() != session.row.intent()
        || row.command != session.row.command
        || row.target != session.row.target
        || row.workdir != session.row.workdir
    {
        return Err(PortArgsError::StaleBinding);
    }
    let action = canonical_binding(world, row)?;
    if action != session.action {
        return Err(PortArgsError::StaleBinding);
    }
    Ok(())
}

fn validate_plan(
    world: &World,
    producer: &PortArgsProducer,
    plan: &PortInvocationPlan,
) -> Result<(), PortArgsError> {
    validate_producer(producer)?;
    if !plan.producer.same_binding(producer) {
        return Err(PortArgsError::StaleProducer);
    }
    if plan.accepted.schema() != producer.schema() {
        return Err(PortArgsError::StaleSchema);
    }
    if ContextSnapshot::from(world) != plan.context {
        return Err(PortArgsError::StaleContext);
    }
    let action = canonical_binding(world, &plan.row)?;
    if action != plan.action {
        return Err(PortArgsError::StaleBinding);
    }
    let Some(raw) = plan.accepted.value(PORT_FIELD_ID) else {
        return Err(PortArgsError::StaleRequest);
    };
    if raw.is_empty() || raw.trim().is_empty() {
        return Err(PortArgsError::StaleRequest);
    }
    let expected_invocation = port_invocation(&plan.action, plan.effective_port);
    if expected_invocation != plan.invocation {
        return Err(PortArgsError::StaleRequest);
    }
    if matches!(plan.disposition, PortParseDisposition::LegacyFallbackTo5173)
        && plan.effective_port != LEGACY_PORT
    {
        return Err(PortArgsError::StaleRequest);
    }
    Ok(())
}

fn port_invocation(action: &Action, port: u16) -> Invocation {
    Invocation::read_only(
        "lsof",
        vec![
            "-nP".to_owned(),
            format!("-iTCP:{port}"),
            "-sTCP:LISTEN".to_owned(),
        ],
        &action.workdir,
        &action.target,
    )
}

pub fn modeled_port_listeners(
    world: &World,
    producer: &PortArgsProducer,
    plan: &PortInvocationPlan,
    model: Option<&ListenerModel>,
) -> Result<ModeledPortReport, ModeledPortError> {
    let simulation =
        simulate_port_invocation(world, producer, plan).map_err(ModeledPortError::PortArgs)?;
    listeners::project_port_listeners(world, plan, simulation, model)
}

pub fn validate_listener_suggestion(
    world: &World,
    producer: &PortArgsProducer,
    plan: &PortInvocationPlan,
    model: Option<&ListenerModel>,
    suggestion: &ListenerSuggestion,
) -> Result<ListenerFacts, ModeledPortError> {
    let _simulation =
        simulate_port_invocation(world, producer, plan).map_err(ModeledPortError::PortArgs)?;
    let model = model.ok_or(ModeledPortError::ModelUnavailable)?;
    listeners::validate_suggestion(world, plan, model, suggestion)
}
