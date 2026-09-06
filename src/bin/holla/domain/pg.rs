//! pg_activity: live session tree. Blockers form a tree via `blocked_by`;
//! kill semantics (cancel before terminate, revalidate PID + query) are P3.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgSession {
    pub pid: u32,
    pub user: String,
    /// The query text is fixture data, never a real statement.
    pub query: String,
    pub wait_event: Option<String>,
    pub blocked_by: Option<u32>,
    pub duration_ms: u64,
}

impl PgSession {
    pub fn blocking(&self) -> bool {
        self.blocked_by.is_none() && self.wait_event.is_none()
    }
}

/// Root blockers: sessions others wait on.
pub fn blockers(sessions: &[PgSession]) -> Vec<&PgSession> {
    let waited_on: std::collections::BTreeSet<u32> =
        sessions.iter().filter_map(|s| s.blocked_by).collect();
    sessions
        .iter()
        .filter(|s| waited_on.contains(&s.pid))
        .collect()
}

/// Sessions directly blocked by `pid`.
pub fn blocked_by(sessions: &[PgSession], pid: u32) -> Vec<&PgSession> {
    sessions
        .iter()
        .filter(|s| s.blocked_by == Some(pid))
        .collect()
}
