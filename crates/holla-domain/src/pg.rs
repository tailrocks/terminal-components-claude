#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgSession {
    pub pid: u32,
    pub user: String,
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

pub fn blockers(sessions: &[PgSession]) -> Vec<&PgSession> {
    let waited_on: std::collections::BTreeSet<u32> =
        sessions.iter().filter_map(|s| s.blocked_by).collect();
    sessions
        .iter()
        .filter(|s| waited_on.contains(&s.pid))
        .collect()
}

pub fn blocked_by(sessions: &[PgSession], pid: u32) -> Vec<&PgSession> {
    sessions
        .iter()
        .filter(|s| s.blocked_by == Some(pid))
        .collect()
}
