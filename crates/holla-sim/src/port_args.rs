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

/// Fixed producer declaration for the authored, read-only `system.port` action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortArgsProducer {
    adapter_id: String,
    adapter_version: u16,
    schema: InputSchema,
}

impl PortArgsProducer {
    /// Builds the producer's fixed public port schema.
    ///
    /// # Errors
    /// Returns a schema error if the built-in field declaration is invalid.
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

    /// Returns the validated schema owned by this producer.
    pub fn schema(&self) -> &InputSchema {
        &self.schema
    }

    fn same_binding(&self, other: &Self) -> bool {
        self.adapter_id == other.adapter_id
            && self.adapter_version == other.adapter_version
            && self.schema == other.schema
    }
}

/// Returns the fixed producer for the port action.
///
/// # Errors
/// Returns a schema error if the built-in declaration is invalid.
pub fn port_args_producer() -> Result<PortArgsProducer, SchemaError> {
    PortArgsProducer::new()
}

/// Builds the one ordinary authored canonical action before recent and pin rows are derived.
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

/// Identifies the reserved canonical producer before generic command dispatch.
///
/// A stale canonical ID remains reserved and therefore fails closed in `begin`; command text on
/// an unrelated or unavailable-memory row never creates a producer.
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

/// Unaccepted input state bound to one current canonical action and host context.
#[derive(Debug)]
pub struct PortArgsSession {
    producer: PortArgsProducer,
    row: Action,
    action: Action,
    context: ContextSnapshot,
    accepted: bool,
}

/// An error found while binding or revalidating port input.
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

/// Error returned when a bound port input accepts or validates typed values.
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
    /// Binds the canonical producer row before the native editor opens.
    ///
    /// # Errors
    /// Returns an error if the row is stale, ambiguous, blocked, or has changed policy.
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

    /// Validates required/default values once and captures one immutable typed review plan.
    ///
    /// The schema supplies the default only when `port` is omitted. Explicit empty input is
    /// rejected; malformed non-empty input retains the original untrimmed `u16` parse fallback.
    ///
    /// # Errors
    /// Returns a binding error if the captured action/context changed, or an input error if values
    /// violate the schema.
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

/// Non-cloneable accepted request, captured parse disposition, and structured numeric argv.
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
    /// Returns the one structured argv vector captured when input was accepted.
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }

    /// Returns the accepted bytes without reparsing or trimming them.
    pub fn accepted_inputs(&self) -> &AcceptedInputs {
        &self.accepted
    }

    /// Returns the parse disposition recorded at acceptance.
    pub fn disposition(&self) -> PortParseDisposition {
        self.disposition
    }

    /// Returns the effective numeric port captured at acceptance.
    pub fn effective_port(&self) -> u16 {
        self.effective_port
    }
}

/// Non-secret facts revalidated from the exact captured plan for the review screen.
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

/// Revalidates the same captured plan and projects facts for explicit user review.
///
/// # Errors
/// Returns an error if the current action, producer, schema, or world context differs.
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

/// Revalidates a plan and returns only a pure, structured simulation report.
///
/// This does not run `lsof`, inspect processes, mutate activities, or change `World`.
///
/// # Errors
/// Returns an error if the current action, producer, schema, or world context differs.
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

/// Revalidates the accepted port request and projects optional deterministic listener data.
///
/// The request's effective numeric port is authoritative; accepted text, action labels, command
/// strings, and argv are never reparsed to select model rows. Nothing is executed.
///
/// # Errors
/// Returns a port binding error or a stale model-context error.
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

/// Revalidates one suggestion against the same accepted request and exact immutable model.
///
/// # Errors
/// Returns a port binding error, unavailable-model error, or stale model/suggestion error.
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

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "Finite typed-port controls assert fixed producer and fixture contracts"
)]
mod tests {
    use super::*;
    use crate::listeners::{ListenerAvailability, ListenerProvenance, ListenerRecord};
    use crate::scenario::Scenario;

    #[derive(Clone, Copy, Debug)]
    enum ContextDrift {
        HostName,
        HostKind,
        Environment,
        Cwd,
        EffectRevision,
    }

    fn apply_context_drift(world: &mut World, drift: ContextDrift) {
        match drift {
            ContextDrift::HostName => world.host.name.push_str("-changed"),
            ContextDrift::HostKind => world.host.kind = HostKind::Remote,
            ContextDrift::Environment => world.host.env = Environment::Production,
            ContextDrift::Cwd => world.cwd.push_str("-changed"),
            ContextDrift::EffectRevision => world.effect_revision += 1,
        }
    }

    fn bound() -> (World, PortArgsProducer, Action) {
        let world = crate::fixtures::world_for(Scenario::RustDirty);
        let producer = port_args_producer().unwrap();
        let row = catalogue::catalogue(&world)
            .into_iter()
            .find(|action| action.id == ACTION_ID)
            .unwrap();
        (world, producer, row)
    }

    fn accept(raw: Option<&str>) -> Result<PortInvocationPlan, PortArgsAcceptError> {
        let (world, producer, row) = bound();
        let mut session = PortArgsSession::begin(&world, &row, &producer)
            .map_err(PortArgsAcceptError::Binding)?;
        match raw {
            Some(value) => session.accept(&world, &row, &producer, &[(PORT_FIELD_ID, value)]),
            None => session.accept(&world, &row, &producer, &[]),
        }
    }

    #[test]
    fn omitted_default_is_captured_once_and_builds_ordered_numeric_argv() {
        let plan = accept(None).unwrap();
        assert_eq!(plan.accepted_inputs().value(PORT_FIELD_ID), Some("5173"));
        assert_eq!(plan.effective_port(), 5173);
        assert_eq!(plan.disposition(), PortParseDisposition::Parsed);
        assert_eq!(plan.invocation().program(), "lsof");
        assert_eq!(
            plan.invocation().argv(),
            &["-nP", "-iTCP:5173", "-sTCP:LISTEN"]
        );
        assert_eq!(plan.invocation().risk(), Risk::ReadOnly);

        let (world, producer, row) = bound();
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        assert!(session.accept(&world, &row, &producer, &[]).is_ok());
        assert_eq!(
            session.accept(&world, &row, &producer, &[]).err(),
            Some(PortArgsAcceptError::Binding(PortArgsError::StaleRequest))
        );
    }

    #[test]
    fn numeric_boundaries_and_parser_normalization_are_preserved() {
        for (raw, expected) in [("0", 0), ("65535", u16::MAX), ("00080", 80), ("+80", 80)] {
            let plan = accept(Some(raw)).unwrap();
            assert_eq!(plan.accepted_inputs().value(PORT_FIELD_ID), Some(raw));
            assert_eq!(plan.effective_port(), expected);
            assert_eq!(plan.disposition(), PortParseDisposition::Parsed);
            assert_eq!(plan.invocation().argv()[1], format!("-iTCP:{expected}"));
        }
    }

    #[test]
    fn malformed_and_out_of_range_nonempty_input_keep_legacy_fallback_and_raw_bytes() {
        for raw in ["abc", "51735173", "65536", " 80", "80 "] {
            let plan = accept(Some(raw)).unwrap();
            assert_eq!(plan.accepted_inputs().value(PORT_FIELD_ID), Some(raw));
            assert_eq!(plan.effective_port(), LEGACY_PORT);
            assert_eq!(
                plan.disposition(),
                PortParseDisposition::LegacyFallbackTo5173
            );
            assert_eq!(plan.invocation().argv()[1], "-iTCP:5173");
        }
    }

    #[test]
    fn required_empty_and_whitespace_values_do_not_receive_the_omitted_default() {
        for raw in ["", " ", "\t"] {
            assert!(matches!(
                accept(Some(raw)),
                Err(PortArgsAcceptError::Input(InputError::MissingRequired(field)))
                    if field == PORT_FIELD_ID
            ));
        }
    }

    #[test]
    fn unknown_duplicate_and_invalid_fields_are_rejected_by_the_declared_schema() {
        let (world, producer, row) = bound();
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        assert!(matches!(
            session.accept(&world, &row, &producer, &[("other", "80")]),
            Err(PortArgsAcceptError::Input(InputError::UnknownField(_)))
        ));
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        assert!(matches!(
            session.accept(&world, &row, &producer, &[("bad id", "80")]),
            Err(PortArgsAcceptError::Input(InputError::InvalidFieldId(_)))
        ));
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        assert!(matches!(
            session.accept(
                &world,
                &row,
                &producer,
                &[(PORT_FIELD_ID, "80"), (PORT_FIELD_ID, "81")]
            ),
            Err(PortArgsAcceptError::Input(InputError::DuplicateField(field)))
                if field == PORT_FIELD_ID
        ));
    }

    #[test]
    fn stale_context_action_and_wrong_command_fail_closed() {
        let (world, producer, row) = bound();
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        let mut changed = world;
        changed.cwd.push_str("/changed");
        assert!(matches!(
            session.accept(&changed, &row, &producer, &[]),
            Err(PortArgsAcceptError::Binding(PortArgsError::StaleContext))
        ));

        let (world, producer, row) = bound();
        let mut stale = row.clone();
        stale.command = "lsof -nP".to_owned();
        assert!(matches!(
            PortArgsSession::begin(&world, &stale, &producer),
            Err(PortArgsError::StaleAction)
        ));
    }

    #[test]
    fn every_captured_context_dimension_rejects_acceptance_review_and_simulation() {
        for drift in [
            ContextDrift::HostName,
            ContextDrift::HostKind,
            ContextDrift::Environment,
            ContextDrift::Cwd,
            ContextDrift::EffectRevision,
        ] {
            let (world, producer, row) = bound();
            let mut changed = crate::fixtures::world_for(Scenario::RustDirty);
            apply_context_drift(&mut changed, drift);

            let mut unaccepted = PortArgsSession::begin(&world, &row, &producer).unwrap();
            assert!(
                matches!(
                    unaccepted.accept(&changed, &row, &producer, &[]),
                    Err(PortArgsAcceptError::Binding(PortArgsError::StaleContext))
                ),
                "acceptance must reject {drift:?}"
            );

            let mut fresh = PortArgsSession::begin(&world, &row, &producer).unwrap();
            let plan = fresh.accept(&world, &row, &producer, &[]).unwrap();
            assert_eq!(
                port_review_facts(&changed, &producer, &plan).err(),
                Some(PortArgsError::StaleContext),
                "review must reject {drift:?}"
            );
            assert_eq!(
                simulate_port_invocation(&changed, &producer, &plan).err(),
                Some(PortArgsError::StaleContext),
                "simulation must reject {drift:?}"
            );
        }
    }

    #[test]
    fn adapter_and_schema_drift_reject_acceptance_review_and_simulation() {
        let (world, producer, row) = bound();
        let mut changed_adapter = producer.clone();
        changed_adapter.adapter_version = ADAPTER_VERSION + 1;
        let mut adapter_session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        assert!(matches!(
            adapter_session.accept(&world, &row, &changed_adapter, &[]),
            Err(PortArgsAcceptError::Binding(PortArgsError::StaleProducer))
        ));

        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        let plan = session.accept(&world, &row, &producer, &[]).unwrap();
        assert_eq!(
            port_review_facts(&world, &changed_adapter, &plan).err(),
            Some(PortArgsError::StaleProducer)
        );
        assert_eq!(
            simulate_port_invocation(&world, &changed_adapter, &plan).err(),
            Some(PortArgsError::StaleProducer)
        );

        let mut changed_schema = producer.clone();
        changed_schema.schema = InputSchema::new(
            SCHEMA_VERSION + 1,
            vec![
                InputField::text(PORT_FIELD_ID, PORT_FIELD_LABEL, true)
                    .unwrap()
                    .with_help(PORT_FIELD_HELP)
                    .with_default("5173"),
            ],
        )
        .unwrap();
        let mut schema_session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        assert!(matches!(
            schema_session.accept(&world, &row, &changed_schema, &[]),
            Err(PortArgsAcceptError::Binding(PortArgsError::StaleSchema))
        ));
        assert_eq!(
            port_review_facts(&world, &changed_schema, &plan).err(),
            Some(PortArgsError::StaleSchema)
        );
        assert_eq!(
            simulate_port_invocation(&world, &changed_schema, &plan).err(),
            Some(PortArgsError::StaleSchema)
        );
    }

    #[test]
    fn review_and_simulation_reuse_the_plan_without_changing_world_effects() {
        let (world, producer, row) = bound();
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        let plan = session
            .accept(&world, &row, &producer, &[(PORT_FIELD_ID, "abc")])
            .unwrap();
        let before = (
            world.effect_revision,
            world.activities.len(),
            world.cwd.clone(),
        );
        let facts = port_review_facts(&world, &producer, &plan).unwrap();
        let report = simulate_port_invocation(&world, &producer, &plan).unwrap();
        assert_eq!(facts.invocation, *plan.invocation());
        assert_eq!(facts.effective_port, 5173);
        assert_eq!(
            facts.disposition,
            PortParseDisposition::LegacyFallbackTo5173
        );
        assert_eq!(report.invocation, *plan.invocation());
        assert_eq!(report.status, ExecutionStatus::NothingExecuted);
        assert_eq!(
            (
                world.effect_revision,
                world.activities.len(),
                world.cwd.clone()
            ),
            before
        );
    }

    #[test]
    fn removed_remembered_row_stales_the_same_accepted_plan() {
        let (mut world, producer, canonical) = bound();
        world.memory.pins.push(crate::domain::ranking::Pin {
            path: world.cwd.clone(),
            command: canonical.command.clone(),
        });
        let expected_pin_id = format!("pin:{}", canonical.command);
        let row = catalogue::catalogue(&world)
            .into_iter()
            .find(|action| action.id == expected_pin_id)
            .unwrap();
        assert_eq!(row.id, expected_pin_id);
        assert_eq!(row.intent_id(), Some("system.port"));
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        let plan = session
            .accept(&world, &row, &producer, &[(PORT_FIELD_ID, "8080")])
            .unwrap();
        world.memory.pins.clear();
        assert!(matches!(
            port_review_facts(&world, &producer, &plan),
            Err(PortArgsError::StaleAction)
        ));
    }

    #[test]
    fn unchanged_alias_origin_keeps_its_canonical_binding_at_acceptance() {
        let (mut world, producer, canonical) = bound();
        world.memory.set_alias("ports", &canonical.command).unwrap();
        let row = catalogue::catalogue(&world)
            .into_iter()
            .find(|action| action.id == ACTION_ID)
            .unwrap();
        assert_eq!(row.intent_id(), Some(ACTION_ID));
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        let plan = session
            .accept(&world, &row, &producer, &[(PORT_FIELD_ID, "8080")])
            .unwrap();
        assert_eq!(plan.row.id, row.id);
        assert_eq!(plan.action.id, ACTION_ID);
        assert_eq!(
            plan.invocation.argv(),
            &["-nP", "-iTCP:8080", "-sTCP:LISTEN"]
        );
    }

    #[test]
    fn acceptance_rejects_removed_pin_even_if_caller_substitutes_canonical_row() {
        let (mut world, producer, canonical) = bound();
        let pin = crate::domain::ranking::Pin {
            path: world.cwd.clone(),
            command: canonical.command.clone(),
        };
        world.memory.pins.push(pin);
        let captured_pin = catalogue::catalogue(&world)
            .into_iter()
            .find(|action| action.id == format!("pin:{}", canonical.command))
            .unwrap();
        let mut session = PortArgsSession::begin(&world, &captured_pin, &producer).unwrap();
        world.memory.pins.clear();
        assert!(matches!(
            session.accept(&world, &canonical, &producer, &[(PORT_FIELD_ID, "8080")]),
            Err(PortArgsAcceptError::Binding(PortArgsError::StaleAction))
        ));
    }

    #[test]
    fn duplicate_remembered_raw_ids_are_ambiguous_and_unknown_memory_stays_unavailable() {
        let (mut world, producer, canonical) = bound();
        let pin = crate::domain::ranking::Pin {
            path: world.cwd.clone(),
            command: canonical.command.clone(),
        };
        world.memory.pins.extend([pin.clone(), pin]);
        let duplicate = catalogue::catalogue(&world)
            .into_iter()
            .find(|action| action.id == format!("pin:{}", canonical.command))
            .unwrap();
        assert_eq!(
            PortArgsSession::begin(&world, &duplicate, &producer).err(),
            Some(PortArgsError::AmbiguousAction)
        );

        world.memory.pins.clear();
        world.memory.pins.push(crate::domain::ranking::Pin {
            path: world.cwd.clone(),
            command: "lsof -nP -iTCP:8080 -sTCP:LISTEN".to_owned(),
        });
        let unknown = catalogue::catalogue(&world)
            .into_iter()
            .find(|action| action.id == "pin:lsof -nP -iTCP:8080 -sTCP:LISTEN")
            .unwrap();
        assert!(port_args_producer_for(&unknown).unwrap().is_none());
        assert_eq!(
            PortArgsSession::begin(&world, &unknown, &producer).err(),
            Some(PortArgsError::StaleAction)
        );
    }

    #[test]
    fn controls_unicode_and_long_raw_values_are_retained_without_becoming_shell_text() {
        let long = "x".repeat(4_096);
        for raw in ["\u{1b}[31m", "端口", long.as_str()] {
            let plan = accept(Some(raw)).unwrap();
            assert_eq!(plan.accepted_inputs().value(PORT_FIELD_ID), Some(raw));
            assert_eq!(
                plan.disposition(),
                PortParseDisposition::LegacyFallbackTo5173
            );
            assert_eq!(
                plan.invocation().argv(),
                &["-nP", "-iTCP:5173", "-sTCP:LISTEN"]
            );
            assert!(!plan.invocation().argv().iter().any(|arg| arg.contains(raw)));
        }
    }

    #[test]
    fn unrelated_identical_command_text_does_not_acquire_the_port_producer() {
        let action = Action::new(
            "misc.lookalike",
            "Lookalike",
            ActionKind::Task,
            Scope::Here,
            "stored command",
            Risk::ReadOnly,
            "~/work/pave",
            ACTION_COMMAND,
        );
        assert!(port_args_producer_for(&action).unwrap().is_none());
    }

    #[test]
    fn modeled_projection_keeps_exact_argv_and_returns_only_synthetic_fixture_facts() {
        let case = crate::fixtures::port_conflict_case().unwrap();
        let producer = port_args_producer().unwrap();
        let row = catalogue::catalogue(&case.world)
            .into_iter()
            .find(|action| action.id == ACTION_ID)
            .unwrap();
        let mut session = PortArgsSession::begin(&case.world, &row, &producer).unwrap();
        let plan = session
            .accept(&case.world, &row, &producer, &[(PORT_FIELD_ID, "5173")])
            .unwrap();
        let before = (
            case.world.effect_revision,
            case.world.cwd.clone(),
            case.world.activities.clone(),
            plan.accepted_inputs()
                .value(PORT_FIELD_ID)
                .map(str::to_owned),
        );

        let report =
            modeled_port_listeners(&case.world, &producer, &plan, case.listeners.as_ref()).unwrap();
        assert_eq!(report.availability(), ListenerAvailability::Listeners);
        assert_eq!(
            report.provenance(),
            ListenerProvenance::DeterministicHollaSimulation
        );
        assert_eq!(report.simulation().status, ExecutionStatus::NothingExecuted);
        assert_eq!(report.simulation().invocation, *plan.invocation());
        assert_eq!(
            report.simulation().invocation.argv(),
            &["-nP", "-iTCP:5173", "-sTCP:LISTEN"]
        );
        assert_eq!(report.suggestions().len(), 1);

        let facts = validate_listener_suggestion(
            &case.world,
            &producer,
            &plan,
            case.listeners.as_ref(),
            &report.suggestions()[0],
        )
        .unwrap();
        assert_eq!(facts.pid(), 5120);
        assert_eq!(facts.synthetic_generation(), 1);
        assert_eq!(facts.port(), 5173);
        assert_eq!(facts.name(), "node (stale vite)");
        assert_eq!(facts.state(), "sleeping");
        assert_eq!(facts.cpu_percent(), 0);
        assert_eq!(facts.memory_mb(), 310);
        assert_eq!(
            facts.provenance(),
            ListenerProvenance::DeterministicHollaSimulation
        );
        assert_eq!(facts.execution_status(), ExecutionStatus::NothingExecuted);
        assert_eq!(
            (
                case.world.effect_revision,
                case.world.cwd.clone(),
                case.world.activities.clone(),
                plan.accepted_inputs()
                    .value(PORT_FIELD_ID)
                    .map(str::to_owned),
            ),
            before
        );
        assert_eq!(
            simulate_port_invocation(&case.world, &producer, &plan).unwrap(),
            report.simulation().clone()
        );
    }

    #[test]
    fn unavailable_and_known_empty_are_distinct_and_invalid_plans_fail_first() {
        let (world, producer, row) = bound();
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        let plan = session
            .accept(&world, &row, &producer, &[(PORT_FIELD_ID, "8080")])
            .unwrap();

        let unavailable = modeled_port_listeners(&world, &producer, &plan, None).unwrap();
        assert_eq!(
            unavailable.availability(),
            ListenerAvailability::Unavailable
        );
        assert_eq!(
            unavailable.simulation().status,
            ExecutionStatus::NothingExecuted
        );
        assert!(unavailable.suggestions().is_empty());

        let empty = ListenerModel::new(&world, vec![]).unwrap();
        let known_empty = modeled_port_listeners(&world, &producer, &plan, Some(&empty)).unwrap();
        assert_eq!(known_empty.availability(), ListenerAvailability::KnownEmpty);
        assert!(known_empty.suggestions().is_empty());

        let mut changed = crate::fixtures::world_for(Scenario::RustDirty);
        changed.cwd.push_str("/changed");
        assert_eq!(
            modeled_port_listeners(&changed, &producer, &plan, None).err(),
            Some(ModeledPortError::PortArgs(PortArgsError::StaleContext))
        );
    }

    #[test]
    fn same_port_rows_remain_distinct_and_suggestions_bind_model_and_request_identity() {
        let (world, producer, row) = bound();
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        let plan = session
            .accept(&world, &row, &producer, &[(PORT_FIELD_ID, "5173")])
            .unwrap();
        let model = ListenerModel::new(
            &world,
            vec![
                ListenerRecord::new(5120, 1, 5173, "node A", "sleeping", 0, 310).unwrap(),
                ListenerRecord::new(5121, 1, 5173, "node B", "running", 1, 320).unwrap(),
            ],
        )
        .unwrap();
        let report = modeled_port_listeners(&world, &producer, &plan, Some(&model)).unwrap();
        assert_eq!(report.suggestions().len(), 2);
        assert_eq!(
            validate_listener_suggestion(
                &world,
                &producer,
                &plan,
                Some(&model),
                &report.suggestions()[1],
            )
            .unwrap()
            .pid(),
            5121
        );

        let replacement = ListenerModel::new(
            &world,
            vec![
                ListenerRecord::new(5120, 1, 5173, "node A", "sleeping", 0, 310).unwrap(),
                ListenerRecord::new(5121, 1, 5173, "node B", "running", 1, 320).unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(
            validate_listener_suggestion(
                &world,
                &producer,
                &plan,
                Some(&replacement),
                &report.suggestions()[0],
            ),
            Err(ModeledPortError::StaleSuggestion)
        );
        assert_eq!(
            validate_listener_suggestion(&world, &producer, &plan, None, &report.suggestions()[0]),
            Err(ModeledPortError::ModelUnavailable)
        );

        let mut other_session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        let same_port_different_raw = other_session
            .accept(&world, &row, &producer, &[(PORT_FIELD_ID, "05173")])
            .unwrap();
        assert_eq!(
            validate_listener_suggestion(
                &world,
                &producer,
                &same_port_different_raw,
                Some(&model),
                &report.suggestions()[0],
            ),
            Err(ModeledPortError::StaleSuggestion)
        );
    }

    #[test]
    fn model_context_drift_and_replacement_records_fail_closed() {
        for drift in [
            ContextDrift::HostName,
            ContextDrift::HostKind,
            ContextDrift::Environment,
            ContextDrift::Cwd,
            ContextDrift::EffectRevision,
        ] {
            let (world, producer, row) = bound();
            let model = ListenerModel::new(
                &world,
                vec![ListenerRecord::new(5120, 1, 5173, "node", "sleeping", 0, 310).unwrap()],
            )
            .unwrap();
            let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
            let plan = session
                .accept(&world, &row, &producer, &[(PORT_FIELD_ID, "5173")])
                .unwrap();
            let report = modeled_port_listeners(&world, &producer, &plan, Some(&model)).unwrap();
            let mut changed = crate::fixtures::world_for(Scenario::RustDirty);
            apply_context_drift(&mut changed, drift);
            assert_eq!(
                modeled_port_listeners(&changed, &producer, &plan, Some(&model)).err(),
                Some(ModeledPortError::PortArgs(PortArgsError::StaleContext)),
                "projection must reject {drift:?}"
            );
            assert_eq!(
                validate_listener_suggestion(
                    &changed,
                    &producer,
                    &plan,
                    Some(&model),
                    &report.suggestions()[0],
                ),
                Err(ModeledPortError::PortArgs(PortArgsError::StaleContext)),
                "suggestion validation must reject {drift:?}"
            );

            let changed_row = catalogue::catalogue(&changed)
                .into_iter()
                .find(|action| action.id == ACTION_ID)
                .unwrap();
            let mut changed_session =
                PortArgsSession::begin(&changed, &changed_row, &producer).unwrap();
            let changed_plan = changed_session
                .accept(
                    &changed,
                    &changed_row,
                    &producer,
                    &[(PORT_FIELD_ID, "5173")],
                )
                .unwrap();
            let changed_simulation = simulate_port_invocation(&changed, &producer, &changed_plan);
            assert!(
                changed_simulation.is_ok(),
                "fresh plan must pass the existing PortArgs gate for {drift:?}: {changed_simulation:?}"
            );
            assert_eq!(
                modeled_port_listeners(&changed, &producer, &changed_plan, Some(&model)).err(),
                Some(ModeledPortError::StaleModelContext),
                "fresh plan must reach the model-context guard for {drift:?}"
            );
            assert_eq!(
                validate_listener_suggestion(
                    &changed,
                    &producer,
                    &changed_plan,
                    Some(&model),
                    &report.suggestions()[0],
                ),
                Err(ModeledPortError::StaleModelContext),
                "fresh plan must reach suggestion model-context guard for {drift:?}"
            );
        }

        let (world, producer, row) = bound();
        let model = ListenerModel::new(
            &world,
            vec![ListenerRecord::new(5120, 1, 5173, "node", "sleeping", 0, 310).unwrap()],
        )
        .unwrap();
        let mut session = PortArgsSession::begin(&world, &row, &producer).unwrap();
        let plan = session
            .accept(&world, &row, &producer, &[(PORT_FIELD_ID, "5173")])
            .unwrap();
        let report = modeled_port_listeners(&world, &producer, &plan, Some(&model)).unwrap();
        let replacement = ListenerModel::new(
            &world,
            vec![ListenerRecord::new(5120, 2, 5173, "node", "sleeping", 0, 310).unwrap()],
        )
        .unwrap();
        assert_eq!(
            validate_listener_suggestion(
                &world,
                &producer,
                &plan,
                Some(&replacement),
                &report.suggestions()[0],
            ),
            Err(ModeledPortError::StaleSuggestion)
        );

        let removed = ListenerModel::new(&world, vec![]).unwrap();
        assert_eq!(
            validate_listener_suggestion(
                &world,
                &producer,
                &plan,
                Some(&removed),
                &report.suggestions()[0],
            ),
            Err(ModeledPortError::StaleSuggestion)
        );
    }
}
