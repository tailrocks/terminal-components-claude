use super::world::World;
use crate::domain::{
    host::{Environment, HostKind},
    pg::{self, PgSession},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Cancel,
    Terminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewError {
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

pub struct Review {
    target: PgSession,
    operation: Operation,
    host: (String, HostKind, Environment),
    revision: u64,
}

pub struct Report {
    pub pid: u32,
    pub resumed: usize,
    pub operation: Operation,
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
    pub fn new(world: &World, operation: Operation) -> Result<Self, ReviewError> {
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
    pub fn operation(&self) -> Operation {
        self.operation
    }

    pub fn target(&self) -> &PgSession {
        &self.target
    }

    pub fn execute(self, world: &mut World) -> Result<Report, ReviewError> {
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
