//! Target-bound `PostgreSQL` fixture review. No query or process is executed.
use super::world::World;
use crate::domain::{
    host::{Environment, HostKind},
    pg::{self, PgSession},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operation {
    Cancel,
    Terminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReviewError {
    NoBlocker,
    InvalidInventory,
    StaleTarget,
    RevisionExhausted,
}
impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoBlocker => "No blockers · sessions healthy",
            Self::InvalidInventory => "Invalid session identities · nothing changed",
            Self::StaleTarget => "Revalidated: session target changed · review it again",
            Self::RevisionExhausted => "Simulation revision exhausted · nothing changed",
        })
    }
}
impl std::error::Error for ReviewError {}

/// Owned review is consumed on confirmation or cancellation. Activity duration
/// is display-only; PID, query, user and wait relationship identify the target.
pub(crate) struct Review {
    target: PgSession,
    operation: Operation,
    host: (String, HostKind, Environment),
    revision: u64,
}

pub(crate) struct Report {
    pub(crate) pid: u32,
    pub(crate) resumed: usize,
    pub(crate) operation: Operation,
}
impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.operation {
            Operation::Cancel => write!(
                f,
                "Cancelled pid {} · {} waiting sessions resumed",
                self.pid, self.resumed
            ),
            Operation::Terminate => write!(
                f,
                "Terminated pid {} after revalidation · {} waiting sessions resumed",
                self.pid, self.resumed
            ),
        }
    }
}

fn valid_sessions(sessions: &[PgSession]) -> bool {
    let mut ids = std::collections::BTreeSet::new();
    sessions
        .iter()
        .all(|session| session.pid != 0 && ids.insert(session.pid))
}

impl Review {
    pub(crate) fn new(world: &World, operation: Operation) -> Result<Self, ReviewError> {
        let sessions = world.pg.as_ref().ok_or(ReviewError::NoBlocker)?;
        if !valid_sessions(sessions) {
            return Err(ReviewError::InvalidInventory);
        }
        let target = pg::blockers(sessions)
            .first()
            .copied()
            .ok_or(ReviewError::NoBlocker)?
            .clone();
        Ok(Self {
            target,
            operation,
            host: (world.host.name.clone(), world.host.kind, world.host.env),
            revision: world.effect_revision,
        })
    }
    pub(crate) fn operation(&self) -> Operation {
        self.operation
    }

    pub(crate) fn target(&self) -> &PgSession {
        &self.target
    }

    pub(crate) fn execute(self, world: &mut World) -> Result<Report, ReviewError> {
        if world.effect_revision != self.revision
            || self.host != (world.host.name.clone(), world.host.kind, world.host.env)
        {
            return Err(ReviewError::StaleTarget);
        }
        let sessions = world.pg.as_ref().ok_or(ReviewError::StaleTarget)?;
        if !valid_sessions(sessions) {
            return Err(ReviewError::InvalidInventory);
        }
        let current = sessions
            .iter()
            .find(|session| session.pid == self.target.pid)
            .ok_or(ReviewError::StaleTarget)?;
        if current.user != self.target.user
            || current.query != self.target.query
            || current.wait_event != self.target.wait_event
            || current.blocked_by != self.target.blocked_by
        {
            return Err(ReviewError::StaleTarget);
        }
        if !sessions
            .iter()
            .any(|session| session.blocked_by == Some(self.target.pid))
        {
            return Err(ReviewError::StaleTarget);
        }
        let revision = world
            .effect_revision
            .checked_add(1)
            .ok_or(ReviewError::RevisionExhausted)?;
        let sessions = world.pg.as_mut().ok_or(ReviewError::StaleTarget)?;
        let resumed = sessions
            .iter()
            .filter(|session| session.blocked_by == Some(self.target.pid))
            .count();
        sessions.retain(|session| session.pid != self.target.pid);
        for session in sessions {
            if session.blocked_by == Some(self.target.pid) {
                session.blocked_by = None;
            }
        }
        world.effect_revision = revision;
        Ok(Report {
            pid: self.target.pid,
            resumed,
            operation: self.operation,
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "Exact deterministic review fixtures must be present"
)]
mod tests {
    use super::*;
    use crate::{domain::fixtures, scenario::Scenario};
    fn world() -> World {
        let mut world = fixtures::world_for(Scenario::RemoteHost);
        world.pg = Some(vec![
            session(10, None),
            session(11, Some(10)),
            session(20, None),
            session(21, Some(20)),
        ]);
        world
    }
    fn session(pid: u32, blocked_by: Option<u32>) -> PgSession {
        PgSession {
            pid,
            user: "fixture".into(),
            query: format!("query {pid}"),
            wait_event: None,
            blocked_by,
            duration_ms: 0,
        }
    }
    #[test]
    fn original_target_disappearance_never_selects_another_blocker() {
        for operation in [Operation::Cancel, Operation::Terminate] {
            let mut world = world();
            let review = Review::new(&world, operation).unwrap();
            world
                .pg
                .as_mut()
                .unwrap()
                .retain(|session| session.pid != 10);
            let before = world.pg.clone();
            assert!(matches!(
                review.execute(&mut world),
                Err(ReviewError::StaleTarget)
            ));
            assert_eq!(world.pg, before);
            assert_eq!(world.effect_revision, 0);
        }
    }
    #[test]
    fn target_query_host_and_revision_changes_refuse_without_effects() {
        for mutation in 0..4 {
            let mut world = world();
            let review = Review::new(&world, Operation::Cancel).unwrap();
            match mutation {
                0 => {
                    world
                        .pg
                        .as_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|session| session.pid == 10)
                        .unwrap()
                        .query = "different query".into();
                }
                1 => world.host.name = "different-host".into(),
                2 => world.effect_revision = 1,
                _ => world.host.env = Environment::Local,
            }
            let before = world.pg.clone();
            assert!(matches!(
                review.execute(&mut world),
                Err(ReviewError::StaleTarget)
            ));
            assert_eq!(world.pg, before);
        }
    }
    #[test]
    fn valid_cancel_and_terminate_preserve_exact_target_and_waiter_outputs() {
        for operation in [Operation::Cancel, Operation::Terminate] {
            let mut world = world();
            let review = Review::new(&world, operation).unwrap();
            assert_eq!(review.target().pid, 10);
            assert_eq!(review.operation(), operation);
            let report = review.execute(&mut world).unwrap();
            assert_eq!(report.pid, 10);
            assert_eq!(report.resumed, 1);
            assert_eq!(world.effect_revision, 1);
            let sessions = world.pg.unwrap();
            assert!(sessions.iter().all(|session| session.pid != 10));
            assert_eq!(
                sessions
                    .iter()
                    .find(|session| session.pid == 11)
                    .unwrap()
                    .blocked_by,
                None
            );
            assert_eq!(
                sessions
                    .iter()
                    .find(|session| session.pid == 21)
                    .unwrap()
                    .blocked_by,
                Some(20)
            );
            assert_eq!(
                report.to_string(),
                match operation {
                    Operation::Cancel => "Cancelled pid 10 · 1 waiting sessions resumed",
                    Operation::Terminate =>
                        "Terminated pid 10 after revalidation · 1 waiting sessions resumed",
                }
            );
        }
    }
    #[test]
    fn duplicate_pids_and_revision_overflow_never_mutate() {
        let mut world = world();
        world.pg.as_mut().unwrap().push(session(10, None));
        assert!(matches!(
            Review::new(&world, Operation::Cancel),
            Err(ReviewError::InvalidInventory)
        ));
        world.pg.as_mut().unwrap().pop();
        world.effect_revision = u64::MAX;
        let review = Review::new(&world, Operation::Cancel).unwrap();
        let before = world.pg.clone();
        assert!(matches!(
            review.execute(&mut world),
            Err(ReviewError::RevisionExhausted)
        ));
        assert_eq!(world.pg, before);
    }
}
