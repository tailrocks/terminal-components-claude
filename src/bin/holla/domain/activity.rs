//! Activities: long-lived work that stays reachable after launch (CONCEPT
//! §8.14). The output is produced by a deterministic script that emits lines
//! at fixture ticks, so a frame is reproducible.

use crate::domain::context::ScopeTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityState {
    Running,
    Succeeded,
    Failed,
    Stopped,
    /// Running without this terminal attached (a monitor left open).
    Detached,
}

impl ActivityState {
    pub fn label(self) -> &'static str {
        match self {
            ActivityState::Running => "running",
            ActivityState::Succeeded => "succeeded",
            ActivityState::Failed => "failed",
            ActivityState::Stopped => "stopped",
            ActivityState::Detached => "detached",
        }
    }
    pub fn live(self) -> bool {
        matches!(self, ActivityState::Running | ActivityState::Detached)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityKind {
    /// A project task or command with streaming output.
    Task,
    /// Merged logs from several services; each keeps its identity.
    Logs { services: Vec<String> },
    /// A specialist TUI kept open (`btm`, `pg_activity`).
    Monitor { tool: String },
    /// An SSH session.
    Ssh { alias: String },
}

/// One scripted output line: emitted `at` ticks after start, with an
/// optional service prefix for merged logs, and a tone hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptLine {
    pub at: u64,
    pub service: Option<String>,
    pub text: String,
    pub tone: LineTone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineTone {
    Normal,
    Muted,
    Warning,
    Error,
    Success,
}

/// The deterministic script behind an activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub lines: Vec<ScriptLine>,
    /// Ticks until the activity ends; `None` keeps it running.
    pub ends_at: Option<u64>,
    pub exit: i32,
}

impl Script {
    pub fn running(lines: Vec<ScriptLine>) -> Self {
        Self {
            lines,
            ends_at: None,
            exit: 0,
        }
    }
    pub fn ending(lines: Vec<ScriptLine>, at: u64, exit: i32) -> Self {
        Self {
            lines,
            ends_at: Some(at),
            exit,
        }
    }
}

pub fn line(at: u64, text: &str) -> ScriptLine {
    ScriptLine {
        at,
        service: None,
        text: text.into(),
        tone: LineTone::Normal,
    }
}

pub fn toned(at: u64, text: &str, tone: LineTone) -> ScriptLine {
    ScriptLine {
        at,
        service: None,
        text: text.into(),
        tone,
    }
}

pub fn service(at: u64, svc: &str, text: &str) -> ScriptLine {
    ScriptLine {
        at,
        service: Some(svc.into()),
        text: text.into(),
        tone: LineTone::Normal,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub id: String,
    pub name: String,
    /// The action that started it.
    pub origin: String,
    pub kind: ActivityKind,
    pub scope: ScopeTag,
    pub host: String,
    pub state: ActivityState,
    pub started_tick: u64,
    pub ended_tick: Option<u64>,
    pub exit: Option<i32>,
    /// Lines emitted so far: (service, text, tone).
    pub output: Vec<(Option<String>, String, LineTone)>,
    /// Whether input may be attached safely.
    pub attachable: bool,
    pub script: Script,
    /// Follow-up suggestions after the activity ends.
    pub follow_ups: Vec<String>,
    /// Program insight lines (ports, PIDs, hot files) for the header.
    pub insights: Vec<(String, String)>,
    /// Services hidden in a merged log view.
    pub hidden_services: Vec<String>,
    /// Next script line to emit.
    pub cursor: usize,
}

impl Activity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        name: &str,
        origin: &str,
        kind: ActivityKind,
        scope: ScopeTag,
        host: &str,
        script: Script,
        started_tick: u64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            origin: origin.into(),
            kind,
            scope,
            host: host.into(),
            state: ActivityState::Running,
            started_tick,
            ended_tick: None,
            exit: None,
            output: vec![],
            attachable: false,
            script,
            follow_ups: vec![],
            insights: vec![],
            hidden_services: vec![],
            cursor: 0,
        }
    }

    /// Advance to `tick`: emit every line whose time has come and settle
    /// the end state. Returns true when anything changed.
    pub fn advance(&mut self, tick: u64) -> bool {
        if !matches!(self.state, ActivityState::Running | ActivityState::Detached) {
            return false;
        }
        let elapsed = tick.saturating_sub(self.started_tick);
        let mut changed = false;
        while let Some(l) = self.script.lines.get(self.cursor) {
            if l.at > elapsed {
                break;
            }
            self.output
                .push((l.service.clone(), l.text.clone(), l.tone));
            self.cursor += 1;
            changed = true;
        }
        if let Some(end) = self.script.ends_at
            && elapsed >= end
            && self.state != ActivityState::Detached
        {
            self.state = if self.script.exit == 0 {
                ActivityState::Succeeded
            } else {
                ActivityState::Failed
            };
            self.exit = Some(self.script.exit);
            self.ended_tick = Some(self.started_tick + end);
            changed = true;
        }
        changed
    }

    pub fn stop(&mut self, tick: u64) {
        if self.state.live() {
            self.state = ActivityState::Stopped;
            self.ended_tick = Some(tick);
            self.exit = Some(130);
            self.output
                .push((None, "stopped by holla (SIGINT)".into(), LineTone::Muted));
        }
    }

    pub fn restart(&mut self, tick: u64) {
        self.state = ActivityState::Running;
        self.started_tick = tick;
        self.ended_tick = None;
        self.exit = None;
        self.cursor = 0;
        self.output.clear();
    }

    pub fn duration_ticks(&self, now: u64) -> u64 {
        self.ended_tick
            .unwrap_or(now)
            .saturating_sub(self.started_tick)
    }

    pub fn services(&self) -> Vec<String> {
        match &self.kind {
            ActivityKind::Logs { services } => services.clone(),
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripts_emit_in_order_and_settle_the_end_state() {
        let script = Script::ending(
            vec![
                line(0, "start"),
                line(3, "middle"),
                toned(5, "boom", LineTone::Error),
            ],
            6,
            1,
        );
        let mut a = Activity::new(
            "a",
            "A",
            "Run",
            ActivityKind::Task,
            ScopeTag::here("/x"),
            "mbp",
            script,
            10,
        );
        assert!(a.advance(10));
        assert_eq!(a.output.len(), 1);
        assert!(!a.advance(11));
        a.advance(13);
        assert_eq!(a.output.len(), 2);
        a.advance(16);
        assert_eq!(a.state, ActivityState::Failed);
        assert_eq!(a.exit, Some(1));
        assert_eq!(a.duration_ticks(40), 6);
        assert!(!a.advance(50), "finished activities are inert");
    }

    #[test]
    fn stop_and_restart_keep_the_identity() {
        let script = Script::running(vec![line(0, "serving")]);
        let mut a = Activity::new(
            "d",
            "dev",
            "Run dev",
            ActivityKind::Task,
            ScopeTag::here("/x"),
            "mbp",
            script,
            0,
        );
        a.advance(0);
        a.stop(4);
        assert_eq!(a.state, ActivityState::Stopped);
        assert_eq!(a.exit, Some(130));
        a.restart(9);
        assert_eq!(a.output.len(), 0);
        a.advance(9);
        assert_eq!(a.output.len(), 1);
        assert_eq!(a.state, ActivityState::Running);
    }
}
