use std::collections::HashSet;
use std::fmt;
use std::num::{NonZeroU16, NonZeroU32, NonZeroU64};
use std::sync::Arc;

use crate::domain::activity::{Activity, ActivityState};
use crate::domain::host::{Environment, HostKind};
use crate::port_args::{PortArgsError, PortInvocationPlan, PortParseDisposition};
use crate::task_args::{ExecutionStatus, Invocation, SimulatedReport};
use crate::world::World;

/// Construction and validation errors for deterministic listener inventories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerModelError {
    /// A synthetic process identifier was zero.
    InvalidPid,
    /// A synthetic process generation token was zero.
    InvalidGeneration,
    /// A TCP-listening port was zero.
    InvalidPort,
    /// A synthetic process name was blank.
    InvalidName,
    /// A synthetic process state was blank.
    InvalidState,
    /// A synthetic CPU percentage exceeded 100.
    InvalidCpuPercent,
    /// The same PID/generation pair appeared more than once.
    DuplicateInstance,
    /// A PID appeared more than once, even with another generation token.
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

/// Complete deterministic process facts held by a simulator-owned listener inventory.
///
/// Generation is a synthetic identity token. It is not an operating-system start time.
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
    /// Builds one synthetic TCP-listener row.
    ///
    /// # Errors
    /// Rejects zero identity/port values, blank display facts, or CPU percentages above 100.
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

/// Immutable synthetic listener inventory bound to one World context.
///
/// A newly constructed model gets a fresh allocation-backed identity, even when all rows match.
#[derive(Debug)]
pub struct ListenerModel {
    context: WorldContext,
    epoch: Arc<()>,
    records: Vec<ListenerRecord>,
}

impl ListenerModel {
    /// Validates and captures one deterministic listener inventory for this World.
    ///
    /// # Errors
    /// Rejects ambiguous process identities. Each PID and each PID/generation pair is unique.
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

/// Distinguishes no supplied inventory from a valid inventory with no matching rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerAvailability {
    /// No synthetic inventory was supplied.
    Unavailable,
    /// A valid supplied inventory contains no row for the accepted port.
    KnownEmpty,
    /// A valid supplied inventory contains one or more rows for the accepted port.
    Listeners,
}

/// Provenance for every result from this model-only API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerProvenance {
    /// Facts came only from this deterministic simulator model.
    DeterministicHollaSimulation,
}

/// Identity-bound proposal to inspect one synthetic listener row.
///
/// Its fields stay private; callers must revalidate it against the same accepted request and model.
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

/// Safe synthetic facts returned only after suggestion revalidation.
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
    /// Returns the synthetic process identifier recorded in this fixture.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Returns the synthetic generation token, not an operating-system start time.
    pub fn synthetic_generation(&self) -> u64 {
        self.synthetic_generation
    }

    /// Returns the modeled TCP-listening port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the synthetic process display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the synthetic process display state.
    pub fn state(&self) -> &str {
        &self.state
    }

    /// Returns the synthetic CPU percentage.
    pub fn cpu_percent(&self) -> u32 {
        self.cpu_percent
    }

    /// Returns the synthetic resident-memory estimate in MiB.
    pub fn memory_mb(&self) -> u32 {
        self.memory_mb
    }

    /// Returns the source classification for these facts.
    pub fn provenance(&self) -> ListenerProvenance {
        self.provenance
    }

    /// Returns the fixed no-execution status of this projection.
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

/// Pure simulated invocation plus optional synthetic listener suggestions.
#[derive(Debug)]
pub struct ModeledPortReport {
    simulation: SimulatedReport,
    availability: ListenerAvailability,
    suggestions: Vec<ListenerSuggestion>,
}

impl ModeledPortReport {
    /// Returns the existing structured no-execution simulation report.
    pub fn simulation(&self) -> &SimulatedReport {
        &self.simulation
    }

    /// Returns whether modeled listener data was supplied and matched.
    pub fn availability(&self) -> ListenerAvailability {
        self.availability
    }

    /// Returns opaque suggestions that require explicit revalidation before use.
    pub fn suggestions(&self) -> &[ListenerSuggestion] {
        &self.suggestions
    }

    /// Returns the fixed deterministic-simulation provenance.
    pub fn provenance(&self) -> ListenerProvenance {
        ListenerProvenance::DeterministicHollaSimulation
    }
}

/// Error returned when an accepted request or identity-bound suggestion is stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeledPortError {
    /// The existing accepted PortArgs plan failed its own validation.
    PortArgs(PortArgsError),
    /// Suggestion revalidation was requested without a supplied model.
    ModelUnavailable,
    /// The model or captured host context differs from the current World.
    StaleModelContext,
    /// The request, model epoch, instance key, port, or captured row changed.
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

/// Structured synthetic failure tied to one unchanged failed Activity and World context.
///
/// The retained output is evidence text only; the typed port is never parsed from that text.
#[derive(Debug)]
pub struct PortConflict {
    port: u16,
    context: WorldContext,
    activity: ActivitySnapshot,
}

impl PortConflict {
    pub(crate) fn capture(world: &World, port: u16, activity: &Activity) -> Self {
        Self {
            port,
            context: WorldContext::from(world),
            activity: ActivitySnapshot::from(activity),
        }
    }
}

/// Error returned when the structured synthetic failure no longer matches the World.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortConflictError {
    /// Host identity, working directory, or effect revision changed.
    StaleContext,
    /// The captured Activity ID is no longer present.
    MissingActivity,
    /// The captured Activity ID no longer identifies exactly one Activity.
    DuplicateActivity,
    /// Captured Activity content differs from the current failed Activity.
    ChangedActivity,
    /// The captured Activity is no longer in the failed state.
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

/// Revalidates one authored conflict against its captured failed Activity and host context.
///
/// # Errors
/// Rejects context drift, missing/duplicate activity identity, changed content, or a nonfailed
/// Activity. The port comes from the typed simulation event, never from display text.
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

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "Focused model tests assert fixed synthetic fixture and identity controls"
)]
mod tests {
    use super::*;
    use crate::scenario::Scenario;

    fn record(pid: u32, generation: u64, port: u16, name: &str) -> ListenerRecord {
        ListenerRecord::new(pid, generation, port, name, "sleeping", 0, 310).unwrap()
    }

    #[test]
    fn synthetic_port_conflict_is_typed_and_bound_to_full_failed_activity() {
        let case = crate::fixtures::port_conflict_case().unwrap();
        assert_eq!(
            validate_port_conflict(&case.world, &case.conflict),
            Ok(5173)
        );
        assert!(
            case.world.activities[0]
                .lines
                .iter()
                .any(|line| line.contains("EADDRINUSE"))
        );

        let mut changed = crate::fixtures::port_conflict_case().unwrap();
        changed.world.activities[0].lines[1].push_str(" · changed");
        assert_eq!(
            validate_port_conflict(&changed.world, &changed.conflict),
            Err(PortConflictError::ChangedActivity)
        );

        let old_launch_failure = crate::fixtures::world_for(Scenario::LaunchFailure);
        assert_eq!(old_launch_failure.activities.len(), 1);
        assert_eq!(old_launch_failure.activities[0].name, "seed db");
        assert!(old_launch_failure.activities[0].lines[0].contains("psql"));
    }

    #[test]
    fn port_conflict_rejects_missing_duplicate_nonfailed_and_context_drift() {
        let case = crate::fixtures::port_conflict_case().unwrap();
        let mut missing = crate::fixtures::port_conflict_case().unwrap();
        missing.world.activities.clear();
        assert_eq!(
            validate_port_conflict(&missing.world, &missing.conflict),
            Err(PortConflictError::MissingActivity)
        );

        let mut duplicate = crate::fixtures::port_conflict_case().unwrap();
        let duplicate_activity = duplicate.world.activities[0].clone();
        duplicate.world.activities.push(duplicate_activity);
        assert_eq!(
            validate_port_conflict(&duplicate.world, &duplicate.conflict),
            Err(PortConflictError::DuplicateActivity)
        );

        let mut nonfailed = crate::fixtures::port_conflict_case().unwrap();
        nonfailed.world.activities[0].state = ActivityState::Running;
        assert_eq!(
            validate_port_conflict(&nonfailed.world, &nonfailed.conflict),
            Err(PortConflictError::ActivityNotFailed)
        );

        for drift in 0..5 {
            let mut drifted = crate::fixtures::port_conflict_case().unwrap();
            match drift {
                0 => drifted.world.host.name.push_str("-changed"),
                1 => drifted.world.host.kind = HostKind::Remote,
                2 => drifted.world.host.env = Environment::Production,
                3 => drifted.world.cwd.push_str("/elsewhere"),
                _ => drifted.world.effect_revision += 1,
            }
            assert_eq!(
                validate_port_conflict(&drifted.world, &drifted.conflict),
                Err(PortConflictError::StaleContext)
            );
        }

        assert_eq!(
            validate_port_conflict(&case.world, &case.conflict),
            Ok(5173)
        );
    }

    #[test]
    fn record_and_inventory_reject_invalid_or_ambiguous_identity() {
        assert_eq!(
            ListenerRecord::new(0, 1, 5173, "node", "sleeping", 0, 310).err(),
            Some(ListenerModelError::InvalidPid)
        );
        assert_eq!(
            ListenerRecord::new(5120, 0, 5173, "node", "sleeping", 0, 310).err(),
            Some(ListenerModelError::InvalidGeneration)
        );
        assert_eq!(
            ListenerRecord::new(5120, 1, 0, "node", "sleeping", 0, 310).err(),
            Some(ListenerModelError::InvalidPort)
        );
        assert_eq!(
            ListenerRecord::new(5120, 1, 5173, " ", "sleeping", 0, 310).err(),
            Some(ListenerModelError::InvalidName)
        );
        assert_eq!(
            ListenerRecord::new(5120, 1, 5173, "node", "", 0, 310).err(),
            Some(ListenerModelError::InvalidState)
        );
        assert_eq!(
            ListenerRecord::new(5120, 1, 5173, "node", "sleeping", 101, 310).err(),
            Some(ListenerModelError::InvalidCpuPercent)
        );

        let world = crate::fixtures::world_for(Scenario::RustDirty);
        assert_eq!(
            ListenerModel::new(
                &world,
                vec![record(5120, 1, 5173, "node"), record(5120, 1, 5173, "node")],
            )
            .err(),
            Some(ListenerModelError::DuplicateInstance)
        );
        assert_eq!(
            ListenerModel::new(
                &world,
                vec![
                    record(5120, 1, 5173, "node"),
                    record(5120, 2, 5174, "worker")
                ]
            )
            .err(),
            Some(ListenerModelError::DuplicatePid)
        );
    }
}
