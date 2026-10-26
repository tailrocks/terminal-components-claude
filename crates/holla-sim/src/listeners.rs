use std::collections::HashSet;
use std::fmt;
use std::num::{NonZeroU16, NonZeroU32, NonZeroU64};
use std::sync::Arc;

use crate::domain::activity::{Activity, ActivityState};
use crate::domain::host::{Environment, HostKind};
use crate::port_args::{PortArgsError, PortInvocationPlan, PortParseDisposition};
use crate::task_args::{ExecutionStatus, Invocation, SimulatedReport};
use crate::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerModelError {
    InvalidPid,

    InvalidGeneration,

    InvalidPort,

    InvalidName,

    InvalidState,

    InvalidCpuPercent,

    DuplicateInstance,

    DuplicatePid,
}

impl fmt::Display for ListenerModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidPid => "synthetic listener PID must be nonzero",
            Self::InvalidGeneration => "synthetic listener generation must be nonzero",
            Self::InvalidPort => "TCP listener port must be nonzero",
            Self::InvalidName => "synthetic listener name must not be empty",
            Self::InvalidState => "synthetic listener state must not be empty",
            Self::InvalidCpuPercent => "synthetic listener CPU percent must be at most 100",
            Self::DuplicateInstance => "listener inventory contains a duplicate process instance",
            Self::DuplicatePid => "listener inventory contains a duplicate PID",
        })
    }
}

impl std::error::Error for ListenerModelError {}

#[derive(Debug, PartialEq, Eq)]
pub struct ListenerRecord {
    pid: NonZeroU32,
    synthetic_generation: NonZeroU64,
    port: NonZeroU16,
    name: String,
    state: String,
    cpu_percent: u32,
    memory_mb: u32,
}

impl ListenerRecord {
    pub fn new(
        pid: u32,
        synthetic_generation: u64,
        port: u16,
        name: impl Into<String>,
        state: impl Into<String>,
        cpu_percent: u32,
        memory_mb: u32,
    ) -> Result<Self, ListenerModelError> {
        let pid = NonZeroU32::new(pid).ok_or(ListenerModelError::InvalidPid)?;
        let synthetic_generation =
            NonZeroU64::new(synthetic_generation).ok_or(ListenerModelError::InvalidGeneration)?;
        let port = NonZeroU16::new(port).ok_or(ListenerModelError::InvalidPort)?;
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ListenerModelError::InvalidName);
        }
        let state = state.into();
        if state.trim().is_empty() {
            return Err(ListenerModelError::InvalidState);
        }
        if cpu_percent > 100 {
            return Err(ListenerModelError::InvalidCpuPercent);
        }

        Ok(Self {
            pid,
            synthetic_generation,
            port,
            name,
            state,
            cpu_percent,
            memory_mb,
        })
    }

    fn instance_key(&self) -> ProcessInstanceKey {
        ProcessInstanceKey {
            pid: self.pid,
            generation: self.synthetic_generation,
        }
    }

    fn snapshot(&self) -> RecordSnapshot {
        RecordSnapshot {
            key: self.instance_key(),
            port: self.port.get(),
            name: self.name.clone(),
            state: self.state.clone(),
            cpu_percent: self.cpu_percent,
            memory_mb: self.memory_mb,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProcessInstanceKey {
    pid: NonZeroU32,
    generation: NonZeroU64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordSnapshot {
    key: ProcessInstanceKey,
    port: u16,
    name: String,
    state: String,
    cpu_percent: u32,
    memory_mb: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorldContext {
    host_name: String,
    host_kind: HostKind,
    environment: Environment,
    cwd: String,
    effect_revision: u64,
}

impl From<&World> for WorldContext {
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

#[derive(Debug)]
pub struct ListenerModel {
    context: WorldContext,
    epoch: Arc<()>,
    records: Vec<ListenerRecord>,
}

impl ListenerModel {
    pub fn new(world: &World, records: Vec<ListenerRecord>) -> Result<Self, ListenerModelError> {
        let mut pids = HashSet::with_capacity(records.len());
        let mut instances = HashSet::with_capacity(records.len());
        for record in &records {
            if !instances.insert(record.instance_key()) {
                return Err(ListenerModelError::DuplicateInstance);
            }
            if !pids.insert(record.pid) {
                return Err(ListenerModelError::DuplicatePid);
            }
        }

        Ok(Self {
            context: WorldContext::from(world),
            epoch: Arc::new(()),
            records,
        })
    }

    fn check_context(&self, world: &World) -> Result<(), ModeledPortError> {
        if self.context != WorldContext::from(world) {
            return Err(ModeledPortError::StaleModelContext);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerAvailability {
    Unavailable,

    KnownEmpty,

    Listeners,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerProvenance {
    DeterministicHollaSimulation,
}

#[derive(Debug)]
pub struct ListenerSuggestion {
    port: u16,
    key: ProcessInstanceKey,
    record: RecordSnapshot,
    model_epoch: Arc<()>,
    context: WorldContext,
    request: PortRequestSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PortRequestSnapshot {
    raw_port: Option<String>,
    disposition: PortParseDisposition,
    effective_port: u16,
    invocation: Invocation,
}

impl From<&PortInvocationPlan> for PortRequestSnapshot {
    fn from(plan: &PortInvocationPlan) -> Self {
        Self {
            raw_port: plan.accepted_inputs().value("port").map(str::to_owned),
            disposition: plan.disposition(),
            effective_port: plan.effective_port(),
            invocation: plan.invocation().clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerFacts {
    pid: u32,
    synthetic_generation: u64,
    port: u16,
    name: String,
    state: String,
    cpu_percent: u32,
    memory_mb: u32,
    provenance: ListenerProvenance,
    execution_status: ExecutionStatus,
}

impl ListenerFacts {
    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn synthetic_generation(&self) -> u64 {
        self.synthetic_generation
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn cpu_percent(&self) -> u32 {
        self.cpu_percent
    }

    pub fn memory_mb(&self) -> u32 {
        self.memory_mb
    }

    pub fn provenance(&self) -> ListenerProvenance {
        self.provenance
    }

    pub fn execution_status(&self) -> ExecutionStatus {
        self.execution_status
    }
}

impl From<&RecordSnapshot> for ListenerFacts {
    fn from(record: &RecordSnapshot) -> Self {
        Self {
            pid: record.key.pid.get(),
            synthetic_generation: record.key.generation.get(),
            port: record.port,
            name: record.name.clone(),
            state: record.state.clone(),
            cpu_percent: record.cpu_percent,
            memory_mb: record.memory_mb,
            provenance: ListenerProvenance::DeterministicHollaSimulation,
            execution_status: ExecutionStatus::NothingExecuted,
        }
    }
}

#[derive(Debug)]
pub struct ModeledPortReport {
    simulation: SimulatedReport,
    availability: ListenerAvailability,
    suggestions: Vec<ListenerSuggestion>,
}

impl ModeledPortReport {
    pub fn simulation(&self) -> &SimulatedReport {
        &self.simulation
    }

    pub fn availability(&self) -> ListenerAvailability {
        self.availability
    }

    pub fn suggestions(&self) -> &[ListenerSuggestion] {
        &self.suggestions
    }

    pub fn provenance(&self) -> ListenerProvenance {
        ListenerProvenance::DeterministicHollaSimulation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeledPortError {
    PortArgs(PortArgsError),

    ModelUnavailable,

    StaleModelContext,

    StaleSuggestion,
}

impl fmt::Display for ModeledPortError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PortArgs(error) => return fmt::Display::fmt(error, formatter),
            Self::ModelUnavailable => "synthetic listener inventory is unavailable",
            Self::StaleModelContext => "synthetic listener context changed · review it again",
            Self::StaleSuggestion => "synthetic listener suggestion changed · review it again",
        })
    }
}

impl std::error::Error for ModeledPortError {}

#[derive(Debug)]
pub struct PortConflict {
    port: u16,
    context: WorldContext,
    activity: ActivitySnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortConflictError {
    StaleContext,

    MissingActivity,

    DuplicateActivity,

    ChangedActivity,

    ActivityNotFailed,
}

impl fmt::Display for PortConflictError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::StaleContext => "port conflict context changed · review it again",
            Self::MissingActivity => "failed activity is no longer present",
            Self::DuplicateActivity => "failed activity identity is ambiguous",
            Self::ChangedActivity => "failed activity changed · review it again",
            Self::ActivityNotFailed => "port conflict activity is not failed",
        })
    }
}

impl std::error::Error for PortConflictError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActivitySnapshot {
    id: u32,
    name: String,
    scope: String,
    state: ActivityState,
    started_ms: i64,
    lines: Vec<String>,
}

impl From<&Activity> for ActivitySnapshot {
    fn from(activity: &Activity) -> Self {
        Self {
            id: activity.id,
            name: activity.name.clone(),
            scope: activity.scope.clone(),
            state: activity.state,
            started_ms: activity.started_ms,
            lines: activity.lines.clone(),
        }
    }
}

pub fn validate_port_conflict(
    world: &World,
    conflict: &PortConflict,
) -> Result<u16, PortConflictError> {
    if conflict.context != WorldContext::from(world) {
        return Err(PortConflictError::StaleContext);
    }

    let mut matches = world
        .activities
        .iter()
        .filter(|activity| activity.id == conflict.activity.id);
    let Some(activity) = matches.next() else {
        return Err(PortConflictError::MissingActivity);
    };
    if matches.next().is_some() {
        return Err(PortConflictError::DuplicateActivity);
    }
    if activity.state != ActivityState::Failed {
        return Err(PortConflictError::ActivityNotFailed);
    }
    if ActivitySnapshot::from(activity) != conflict.activity {
        return Err(PortConflictError::ChangedActivity);
    }
    Ok(conflict.port)
}

pub(crate) fn project_port_listeners(
    world: &World,
    plan: &PortInvocationPlan,
    simulation: SimulatedReport,
    model: Option<&ListenerModel>,
) -> Result<ModeledPortReport, ModeledPortError> {
    let request = PortRequestSnapshot::from(plan);
    let port = request.effective_port;
    let Some(model) = model else {
        return Ok(ModeledPortReport {
            simulation,
            availability: ListenerAvailability::Unavailable,
            suggestions: Vec::new(),
        });
    };
    model.check_context(world)?;

    let suggestions = model
        .records
        .iter()
        .filter(|record| record.port.get() == port)
        .map(|record| ListenerSuggestion {
            port,
            key: record.instance_key(),
            record: record.snapshot(),
            model_epoch: Arc::clone(&model.epoch),
            context: model.context.clone(),
            request: request.clone(),
        })
        .collect::<Vec<_>>();
    let availability = if suggestions.is_empty() {
        ListenerAvailability::KnownEmpty
    } else {
        ListenerAvailability::Listeners
    };
    Ok(ModeledPortReport {
        simulation,
        availability,
        suggestions,
    })
}

pub(crate) fn validate_suggestion(
    world: &World,
    plan: &PortInvocationPlan,
    model: &ListenerModel,
    suggestion: &ListenerSuggestion,
) -> Result<ListenerFacts, ModeledPortError> {
    model.check_context(world)?;
    if suggestion.context != WorldContext::from(world) {
        return Err(ModeledPortError::StaleModelContext);
    }
    if !Arc::ptr_eq(&model.epoch, &suggestion.model_epoch) {
        return Err(ModeledPortError::StaleSuggestion);
    }

    let request = PortRequestSnapshot::from(plan);
    if suggestion.request != request || suggestion.port != request.effective_port {
        return Err(ModeledPortError::StaleSuggestion);
    }

    let mut matches = model
        .records
        .iter()
        .filter(|record| record.instance_key() == suggestion.key);
    let Some(record) = matches.next() else {
        return Err(ModeledPortError::StaleSuggestion);
    };
    if matches.next().is_some()
        || record.port.get() != request.effective_port
        || record.snapshot() != suggestion.record
    {
        return Err(ModeledPortError::StaleSuggestion);
    }

    Ok(ListenerFacts::from(&suggestion.record))
}
