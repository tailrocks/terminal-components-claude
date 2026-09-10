//! Activities: long-lived work that stays reachable after launch (CONCEPT
//! §8.14). The output is produced by a deterministic script that emits lines
//! at fixture ticks, so a frame is reproducible.
//!
//! The simulation owns the task lifecycle the way a real supervisor would:
//! queued work never spawns after a cancel, a stop request enters
//! `Cancelling` and only settles once the modeled process group is gone
//! (SIGTERM, then SIGKILL after 750 ms for a resistant child), prompts pause
//! the script until exact input bytes arrive, secret input is never echoed
//! or retained, and output retention is bounded with a visible drop count.

use crate::domain::context::ScopeTag;
use crate::domain::effect::Effect;
use crate::domain::exec::Command;

/// Retained output lines per activity; older lines are dropped and counted.
pub const RETAIN_LINES: usize = 4000;
/// Ticks between SIGTERM and SIGKILL for a child that ignores TERM (750 ms).
pub const KILL_AFTER_TICKS: u64 = 10;
/// Ticks a cooperative child needs to exit after SIGTERM.
pub const TERM_EXIT_TICKS: u64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityState {
    /// Owned but not started: waits for a predecessor in a sequential batch.
    Queued,
    Running,
    /// Stop requested; the process group is being torn down.
    Cancelling,
    Succeeded,
    Failed,
    /// Stopped by holla (cancelled) or exited on its own quit key.
    Stopped,
    /// Running without this terminal attached (a monitor left open).
    Detached,
}

impl ActivityState {
    pub fn label(self) -> &'static str {
        match self {
            ActivityState::Queued => "queued",
            ActivityState::Running => "running",
            ActivityState::Cancelling => "cancelling",
            ActivityState::Succeeded => "succeeded",
            ActivityState::Failed => "failed",
            ActivityState::Stopped => "stopped",
            ActivityState::Detached => "detached",
        }
    }
    /// Still owned: running, detached, queued or being torn down.
    pub fn live(self) -> bool {
        matches!(
            self,
            ActivityState::Running
                | ActivityState::Detached
                | ActivityState::Queued
                | ActivityState::Cancelling
        )
    }
    pub fn finished(self) -> bool {
        !self.live()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineTone {
    Normal,
    Muted,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineKind {
    Output,
    /// The program waits for input after printing this text without a
    /// newline. `secret` prompts are answered without echo.
    Prompt {
        secret: bool,
    },
}

/// One scripted output line: emitted `at` ticks after start, with an
/// optional service prefix for merged logs, and a tone hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptLine {
    pub at: u64,
    pub service: Option<String>,
    pub text: String,
    pub tone: LineTone,
    pub kind: LineKind,
}

/// What a prompt does with the input it receives. `matches: None` is the
/// fallback branch; `Some("\u{4}")` matches end of input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub matches: Option<String>,
    /// Lines relative to the tick the input arrived.
    pub lines: Vec<ScriptLine>,
    /// Ends the activity `ticks` after the input with `exit`.
    pub end: Option<(u64, i32)>,
}

/// The deterministic script behind an activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub lines: Vec<ScriptLine>,
    /// Ticks until the activity ends; `None` keeps it running.
    pub ends_at: Option<u64>,
    pub exit: i32,
    /// Branches per prompt, in prompt order.
    pub branches: Vec<Vec<Branch>>,
    /// The child ignores SIGTERM: cancellation escalates to SIGKILL.
    pub resistant: bool,
    /// The child cannot be started at all (missing executable, cwd).
    pub spawn_failure: Option<String>,
}

impl Script {
    pub fn running(lines: Vec<ScriptLine>) -> Self {
        Self {
            lines,
            ends_at: None,
            exit: 0,
            branches: vec![],
            resistant: false,
            spawn_failure: None,
        }
    }
    pub fn ending(lines: Vec<ScriptLine>, at: u64, exit: i32) -> Self {
        Self {
            lines,
            ends_at: Some(at),
            exit,
            branches: vec![],
            resistant: false,
            spawn_failure: None,
        }
    }
    /// A child that cannot start: the failure is the output.
    pub fn spawn_failed(reason: &str) -> Self {
        Self {
            lines: vec![],
            ends_at: Some(0),
            exit: 127,
            branches: vec![],
            resistant: false,
            spawn_failure: Some(reason.into()),
        }
    }
    /// The truthful outcome for a command this preview does not model.
    pub fn unmodeled(display: &str) -> Self {
        Self::ending(
            vec![
                line(0, &format!("$ {display}")),
                toned(
                    1,
                    &format!("holla: no simulated outcome for `{display}` in this preview"),
                    LineTone::Error,
                ),
            ],
            2,
            127,
        )
    }
    pub fn branches(mut self, b: Vec<Vec<Branch>>) -> Self {
        self.branches = b;
        self
    }
    pub fn resistant(mut self) -> Self {
        self.resistant = true;
        self
    }
}

#[cfg(test)]
impl Script {
    pub fn prompts(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| matches!(l.kind, LineKind::Prompt { .. }))
            .count()
    }
}

pub fn line(at: u64, text: &str) -> ScriptLine {
    ScriptLine {
        at,
        service: None,
        text: text.into(),
        tone: LineTone::Normal,
        kind: LineKind::Output,
    }
}

pub fn toned(at: u64, text: &str, tone: LineTone) -> ScriptLine {
    ScriptLine {
        at,
        service: None,
        text: text.into(),
        tone,
        kind: LineKind::Output,
    }
}

pub fn service(at: u64, svc: &str, text: &str) -> ScriptLine {
    ScriptLine {
        at,
        service: Some(svc.into()),
        text: text.into(),
        tone: LineTone::Normal,
        kind: LineKind::Output,
    }
}

pub fn prompt(at: u64, text: &str, secret: bool) -> ScriptLine {
    ScriptLine {
        at,
        service: None,
        text: text.into(),
        tone: LineTone::Normal,
        kind: LineKind::Prompt { secret },
    }
}

/// Normalise one raw output line the way the executor does: a CR before the
/// newline is dropped, ANSI bytes are kept (the viewer shows them visibly),
/// invalid UTF-8 becomes U+FFFD.
pub fn normalize_line(raw: &[u8]) -> String {
    let raw = raw.strip_suffix(b"\n").unwrap_or(raw);
    let raw = raw.strip_suffix(b"\r").unwrap_or(raw);
    String::from_utf8_lossy(raw).into_owned()
}

/// Split a raw stream into lines, emitting the final unterminated fragment.
pub fn split_stream(raw: &[u8]) -> Vec<String> {
    let mut out = vec![];
    let mut start = 0;
    for (i, b) in raw.iter().enumerate() {
        if *b == b'\n' {
            out.push(normalize_line(&raw[start..=i]));
            start = i + 1;
        }
    }
    if start < raw.len() {
        out.push(normalize_line(&raw[start..]));
    }
    out
}

/// Bytes forwarded to a task's stdin, retained for inspection. Secret
/// input keeps only its length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputRecord {
    pub at: u64,
    pub bytes: Vec<u8>,
    pub secret: bool,
    pub redacted_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub text: String,
    pub secret: bool,
    pub since_tick: u64,
    /// Index into `script.branches`.
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelState {
    pub requested: u64,
    pub kill_sent: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    /// The activity is not live: no bytes are delivered.
    Finished,
    /// Queued work has no stdin yet.
    NotStarted,
    /// The program has no writable stdin (a detached monitor).
    NoStdin,
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
    /// Lines dropped from the head of `output` by retention.
    pub dropped: usize,
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
    /// What a successful end changes in the world.
    pub effect: Option<Effect>,
    /// The exact executed specification.
    pub argv: Vec<Command>,
    /// Sequential batch predecessor: starts once it finished.
    pub after: Option<String>,
    pub batch: Option<String>,
    pub waiting: Option<Prompt>,
    /// Ticks spent waiting for input; the script clock excludes them.
    pub paused_ticks: u64,
    pub stdin: Vec<InputRecord>,
    pub cancel: Option<CancelState>,
    prompts_seen: usize,
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
            dropped: 0,
            attachable: false,
            script,
            follow_ups: vec![],
            insights: vec![],
            hidden_services: vec![],
            cursor: 0,
            effect: None,
            argv: vec![],
            after: None,
            batch: None,
            waiting: None,
            paused_ticks: 0,
            stdin: vec![],
            cancel: None,
            prompts_seen: 0,
        }
    }

    fn emit(&mut self, service: Option<String>, text: String, tone: LineTone) {
        self.output.push((service, text, tone));
        if self.output.len() > RETAIN_LINES {
            let drop = self.output.len() - RETAIN_LINES;
            self.output.drain(..drop);
            self.dropped += drop;
        }
    }

    /// Script time elapsed: wall ticks minus time spent waiting for input.
    pub fn elapsed(&self, tick: u64) -> u64 {
        let wall = tick.saturating_sub(self.started_tick);
        let paused = self.paused_ticks
            + self
                .waiting
                .as_ref()
                .map(|p| tick.saturating_sub(p.since_tick))
                .unwrap_or(0);
        wall.saturating_sub(paused)
    }

    /// The program accepts stdin: a task or logs stream that is running.
    pub fn accepts_input(&self) -> bool {
        matches!(self.kind, ActivityKind::Task | ActivityKind::Ssh { .. })
            && matches!(self.state, ActivityState::Running)
    }

    /// Advance to `tick`: emit every line whose time has come and settle
    /// the end state. Returns true when anything changed.
    pub fn advance(&mut self, tick: u64) -> bool {
        match self.state {
            ActivityState::Cancelling => return self.advance_cancel(tick),
            ActivityState::Running | ActivityState::Detached => {}
            _ => return false,
        }
        if let Some(reason) = self.script.spawn_failure.clone() {
            self.emit(
                None,
                format!("holla: cannot start: {reason}"),
                LineTone::Error,
            );
            self.state = ActivityState::Failed;
            self.exit = Some(127);
            self.ended_tick = Some(tick);
            self.script.spawn_failure = None;
            return true;
        }
        if self.waiting.is_some() {
            return false;
        }
        let elapsed = self.elapsed(tick);
        let mut changed = false;
        while let Some(l) = self.script.lines.get(self.cursor).cloned() {
            if l.at > elapsed {
                break;
            }
            self.cursor += 1;
            changed = true;
            match l.kind {
                LineKind::Output => self.emit(l.service, l.text, l.tone),
                LineKind::Prompt { secret } => {
                    self.emit(None, l.text.clone(), LineTone::Normal);
                    let index = self.prompts_seen;
                    self.prompts_seen += 1;
                    self.waiting = Some(Prompt {
                        text: l.text,
                        secret,
                        since_tick: tick,
                        index,
                    });
                    return true;
                }
            }
        }
        if let Some(end) = self.script.ends_at
            && elapsed >= end
            && self.state != ActivityState::Detached
        {
            // final buffered bytes arrive before the terminal state
            while let Some(l) = self.script.lines.get(self.cursor).cloned() {
                self.cursor += 1;
                if let LineKind::Output = l.kind {
                    self.emit(l.service, l.text, l.tone);
                }
            }
            self.state = if self.script.exit == 0 {
                ActivityState::Succeeded
            } else {
                ActivityState::Failed
            };
            self.exit = Some(self.script.exit);
            self.ended_tick = Some(self.started_tick + end + self.paused_ticks);
            changed = true;
        }
        changed
    }

    fn advance_cancel(&mut self, tick: u64) -> bool {
        let Some(c) = self.cancel.clone() else {
            return false;
        };
        let since = tick.saturating_sub(c.requested);
        if !self.script.resistant {
            if since >= TERM_EXIT_TICKS {
                self.emit(
                    None,
                    "terminated (SIGTERM) · process group reaped".into(),
                    LineTone::Muted,
                );
                self.finish_cancel(tick, 130);
                return true;
            }
            return false;
        }
        if c.kill_sent.is_none() && since >= KILL_AFTER_TICKS {
            self.emit(
                None,
                "no exit after SIGTERM for 750 ms · SIGKILL to the process group".into(),
                LineTone::Warning,
            );
            self.cancel = Some(CancelState {
                requested: c.requested,
                kill_sent: Some(tick),
            });
            return true;
        }
        if let Some(k) = c.kill_sent
            && tick.saturating_sub(k) >= 2
        {
            self.emit(
                None,
                "killed (SIGKILL) · descendants reaped".into(),
                LineTone::Muted,
            );
            self.finish_cancel(tick, 137);
            return true;
        }
        false
    }

    fn finish_cancel(&mut self, tick: u64, exit: i32) {
        self.state = ActivityState::Stopped;
        self.ended_tick = Some(tick);
        self.exit = Some(exit);
        self.waiting = None;
    }

    /// Request a stop. Queued work is cancelled before it ever starts; live
    /// work enters `Cancelling` and settles through `advance`.
    pub fn stop(&mut self, tick: u64) {
        match self.state {
            ActivityState::Queued => {
                self.emit(None, "cancelled before start".into(), LineTone::Muted);
                self.started_tick = tick;
                self.finish_cancel(tick, 130);
            }
            ActivityState::Running | ActivityState::Detached => {
                self.state = ActivityState::Cancelling;
                self.cancel = Some(CancelState {
                    requested: tick,
                    kill_sent: None,
                });
                self.emit(
                    None,
                    "stop requested by holla · SIGTERM to the process group".into(),
                    LineTone::Muted,
                );
            }
            _ => {}
        }
    }

    /// Start queued work now.
    pub fn start(&mut self, tick: u64) {
        if self.state == ActivityState::Queued {
            self.state = ActivityState::Running;
            self.started_tick = tick;
        }
    }

    pub fn restart(&mut self, tick: u64) {
        self.state = ActivityState::Running;
        self.started_tick = tick;
        self.ended_tick = None;
        self.exit = None;
        self.cursor = 0;
        self.output.clear();
        self.dropped = 0;
        self.waiting = None;
        self.paused_ticks = 0;
        self.stdin.clear();
        self.cancel = None;
        self.prompts_seen = 0;
    }

    /// Deliver exact bytes to the program's stdin. The bytes are recorded
    /// (redacted while a secret prompt is active); a waiting prompt chooses
    /// its branch by the text before the CR. Ctrl-C (0x03) is a stop
    /// request; Ctrl-D (0x04) is end of input.
    pub fn send_input(&mut self, bytes: &[u8], tick: u64) -> Result<(), InputError> {
        match self.state {
            ActivityState::Queued => return Err(InputError::NotStarted),
            ActivityState::Running => {}
            ActivityState::Detached | ActivityState::Cancelling => return Err(InputError::NoStdin),
            _ => return Err(InputError::Finished),
        }
        if !matches!(self.kind, ActivityKind::Task | ActivityKind::Ssh { .. }) {
            return Err(InputError::NoStdin);
        }
        let secret = self.waiting.as_ref().is_some_and(|p| p.secret);
        self.stdin.push(InputRecord {
            at: tick,
            bytes: if secret { vec![] } else { bytes.to_vec() },
            secret,
            redacted_len: bytes.len(),
        });
        if bytes == [0x03] {
            self.emit(None, "^C".into(), LineTone::Muted);
            self.stop(tick);
            return Ok(());
        }
        let Some(p) = self.waiting.clone() else {
            // no prompt: the program reads it later; echo the line
            let text = String::from_utf8_lossy(bytes)
                .trim_end_matches(['\r', '\n'])
                .to_owned();
            self.emit(None, text, LineTone::Muted);
            return Ok(());
        };
        let answer = if bytes == [0x04] {
            "\u{4}".to_owned()
        } else {
            String::from_utf8_lossy(bytes)
                .trim_end_matches(['\r', '\n'])
                .to_owned()
        };
        // echo: the prompt row shows the answer, masked when secret
        if let Some(last) = self.output.last_mut() {
            // EOF is a control action, never a secret: it stays visible
            let shown = if answer == "\u{4}" {
                "^D".to_owned()
            } else if secret {
                String::new()
            } else {
                answer.clone()
            };
            last.1 = format!("{}{shown}", p.text);
        }
        self.paused_ticks += tick.saturating_sub(p.since_tick);
        self.waiting = None;
        let elapsed = self.elapsed(tick);
        let branches = self
            .script
            .branches
            .get(p.index)
            .cloned()
            .unwrap_or_default();
        let chosen = branches
            .iter()
            .find(|b| b.matches.as_deref() == Some(answer.as_str()))
            .or_else(|| branches.iter().find(|b| b.matches.is_none()))
            .cloned();
        if let Some(b) = chosen {
            let mut spliced: Vec<ScriptLine> = b
                .lines
                .iter()
                .map(|l| ScriptLine {
                    at: l.at + elapsed,
                    ..l.clone()
                })
                .collect();
            let rest = self.script.lines.split_off(self.cursor);
            self.script.lines.append(&mut spliced);
            self.script.lines.extend(rest);
            if let Some((ticks, exit)) = b.end {
                self.script.ends_at = Some(elapsed + ticks);
                self.script.exit = exit;
            }
        }
        Ok(())
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

    /// Output lines that the program itself produced (excluding holla's
    /// supervisor notes) for a result check.
    #[cfg(test)]
    pub fn last_line(&self) -> Option<&str> {
        self.output.last().map(|(_, t, _)| t.as_str())
    }
}

/// Map a key event to the bytes a terminal would forward (E34).
pub fn key_bytes(key: &junie_tui::core::event::Key) -> Option<Vec<u8>> {
    use ratatui::crossterm::event::KeyCode;
    let ctrl = key.ctrl();
    Some(match key.code {
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Char(c) if ctrl => {
            let lower = c.to_ascii_lowercase();
            if lower.is_ascii_lowercase() {
                vec![(lower as u8) - b'a' + 1]
            } else if c == '@' || c == ' ' {
                vec![0]
            } else if c == '[' {
                vec![0x1b]
            } else {
                return None;
            }
        }
        KeyCode::Char(c) if !key.alt() => c.to_string().into_bytes(),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn act(script: Script, at: u64) -> Activity {
        Activity::new(
            "a",
            "A",
            "Run",
            ActivityKind::Task,
            ScopeTag::here("/x"),
            "mbp",
            script,
            at,
        )
    }

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
        let mut a = act(script, 10);
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
    fn final_bytes_land_before_the_terminal_state_and_fast_exits_keep_output() {
        let script = Script::ending(vec![line(0, "$ true"), line(0, "fast")], 0, 0);
        let mut a = act(script, 0);
        a.advance(0);
        assert_eq!(a.state, ActivityState::Succeeded);
        assert_eq!(a.output.len(), 2, "fast task output is drained before Done");
        let script = Script::ending(vec![line(0, "a"), line(9, "late buffered")], 4, 0);
        let mut b = act(script, 0);
        b.advance(4);
        assert_eq!(b.state, ActivityState::Succeeded);
        assert_eq!(b.last_line(), Some("late buffered"));
    }

    #[test]
    fn stop_enters_cancelling_and_settles_through_the_supervisor() {
        let mut a = act(Script::running(vec![line(0, "serving")]), 0);
        a.advance(0);
        a.stop(4);
        assert_eq!(a.state, ActivityState::Cancelling);
        assert!(a.state.live(), "still owned while cancelling");
        a.advance(5);
        assert_eq!(a.state, ActivityState::Cancelling);
        a.advance(4 + TERM_EXIT_TICKS);
        assert_eq!(a.state, ActivityState::Stopped);
        assert_eq!(a.exit, Some(130));
        assert!(a.last_line().unwrap().contains("SIGTERM"));
        // a resistant child escalates to SIGKILL after 750 ms
        let mut r = act(
            Script::running(vec![line(0, "ignores TERM")]).resistant(),
            0,
        );
        r.advance(0);
        r.stop(10);
        r.advance(10 + TERM_EXIT_TICKS);
        assert_eq!(
            r.state,
            ActivityState::Cancelling,
            "TERM alone does not settle it"
        );
        r.advance(10 + KILL_AFTER_TICKS);
        assert!(r.output.iter().any(|(_, t, _)| t.contains("SIGKILL")));
        assert_eq!(r.state, ActivityState::Cancelling);
        r.advance(10 + KILL_AFTER_TICKS + 2);
        assert_eq!(r.state, ActivityState::Stopped);
        assert_eq!(r.exit, Some(137));
        // stop and restart keep the identity
        r.restart(40);
        assert_eq!(r.output.len(), 0);
        r.advance(40);
        assert_eq!(r.output.len(), 1);
        assert_eq!(r.state, ActivityState::Running);
        assert!(r.cancel.is_none());
    }

    #[test]
    fn queued_work_never_spawns_after_a_cancel() {
        let mut q = act(Script::ending(vec![line(0, "would run")], 3, 0), 0);
        q.state = ActivityState::Queued;
        assert!(!q.advance(5), "queued work does not emit");
        q.stop(6);
        assert_eq!(q.state, ActivityState::Stopped);
        assert_eq!(q.exit, Some(130));
        assert_eq!(q.output.len(), 1);
        assert!(!q.output[0].1.contains("would run"));
        assert!(!q.advance(20));
        assert_eq!(q.output.len(), 1);
    }

    #[test]
    fn spawn_failure_is_output_and_a_failed_done() {
        let mut a = act(Script::spawn_failed("cwd /gone does not exist"), 0);
        assert!(a.advance(0));
        assert_eq!(a.state, ActivityState::Failed);
        assert_eq!(a.exit, Some(127));
        assert!(a.output[0].1.contains("cannot start"));
        assert_eq!(
            a.send_input(b"x\r", 1),
            Err(InputError::Finished),
            "no input after completion"
        );
    }

    #[test]
    fn prompts_pause_the_script_and_take_exact_input() {
        let script = Script::ending(
            vec![
                line(0, "$ sudo systemctl restart payments"),
                prompt(2, "Password: ", true),
                line(3, "restarted"),
            ],
            4,
            0,
        )
        .branches(vec![vec![
            Branch {
                matches: Some("\u{4}".into()),
                lines: vec![toned(0, "sudo: no password was provided", LineTone::Error)],
                end: Some((1, 1)),
            },
            Branch {
                matches: None,
                lines: vec![],
                end: None,
            },
        ]]);
        let mut a = act(script, 0);
        a.advance(2);
        assert!(a.waiting.as_ref().is_some_and(|p| p.secret));
        assert_eq!(a.last_line(), Some("Password: "));
        // time passes without input: nothing else is emitted
        assert!(!a.advance(40));
        assert_eq!(a.output.len(), 2);
        // ordinary bytes while a prompt waits are never echoed or retained
        a.send_input(b"s3cret\r", 41).unwrap();
        assert_eq!(a.stdin.len(), 1);
        assert!(a.stdin[0].secret);
        assert!(a.stdin[0].bytes.is_empty());
        assert_eq!(a.stdin[0].redacted_len, 7);
        assert_eq!(a.last_line(), Some("Password: "), "no echo of a secret");
        assert!(a.waiting.is_none());
        // the script clock excludes the waiting time: line 3 lands one tick later
        assert!(!a.advance(41));
        a.advance(42);
        assert_eq!(a.last_line(), Some("restarted"));
        a.advance(43);
        assert_eq!(a.state, ActivityState::Succeeded);
        assert_eq!(a.ended_tick, Some(43));
        // EOF chooses the EOF branch
        let script = Script::ending(vec![prompt(0, "Password: ", true)], 9, 0).branches(vec![
            vec![Branch {
                matches: Some("\u{4}".into()),
                lines: vec![toned(0, "sudo: no password was provided", LineTone::Error)],
                end: Some((1, 1)),
            }],
        ]);
        let mut e = act(script, 0);
        e.advance(0);
        e.send_input(&[0x04], 3).unwrap();
        assert_eq!(
            e.last_line(),
            Some("Password: ^D"),
            "EOF is visible, not secret"
        );
        e.advance(3);
        e.advance(4);
        assert_eq!(e.state, ActivityState::Failed);
        assert_eq!(e.exit, Some(1));
        // non-secret prompts echo the answer on the prompt row
        let script =
            Script::ending(vec![prompt(0, "Overwrite? [y/N] ", false)], 5, 0).branches(vec![vec![
                Branch {
                    matches: Some("y".into()),
                    lines: vec![line(0, "overwritten")],
                    end: Some((1, 0)),
                },
                Branch {
                    matches: None,
                    lines: vec![line(0, "kept")],
                    end: Some((1, 2)),
                },
            ]]);
        let mut y = act(script.clone(), 0);
        y.advance(0);
        y.send_input(b"y\r", 1).unwrap();
        assert_eq!(y.last_line(), Some("Overwrite? [y/N] y"));
        assert_eq!(y.stdin[0].bytes, b"y\r");
        y.advance(2);
        assert_eq!(y.last_line(), Some("overwritten"));
        assert_eq!(y.state, ActivityState::Succeeded);
        let mut n = act(script, 0);
        n.advance(0);
        n.send_input(b"\r", 1).unwrap();
        n.advance(2);
        assert_eq!(n.exit, Some(2));
        assert_eq!(n.last_line(), Some("kept"));
    }

    #[test]
    fn ctrl_c_stops_and_input_is_refused_outside_ownership() {
        let mut a = act(Script::running(vec![line(0, "serving")]), 0);
        a.advance(0);
        a.send_input(&[0x03], 2).unwrap();
        assert_eq!(a.state, ActivityState::Cancelling);
        assert_eq!(a.send_input(b"x", 3), Err(InputError::NoStdin));
        let mut m = Activity::new(
            "m",
            "btm",
            "Open btm",
            ActivityKind::Monitor { tool: "btm".into() },
            ScopeTag::host("mbp"),
            "mbp",
            Script::running(vec![line(0, "cpu")]),
            0,
        );
        m.advance(0);
        assert_eq!(m.send_input(b"q", 1), Err(InputError::NoStdin));
        let mut q = act(Script::running(vec![]), 0);
        q.state = ActivityState::Queued;
        assert_eq!(q.send_input(b"x", 1), Err(InputError::NotStarted));
        // input without a prompt is delivered and echoed once
        let mut t = act(Script::running(vec![line(0, "reading stdin…")]), 0);
        t.advance(0);
        t.send_input("héllo\ttab\r".as_bytes(), 1).unwrap();
        assert_eq!(t.stdin[0].bytes, "héllo\ttab\r".as_bytes());
        assert_eq!(t.last_line(), Some("héllo\ttab"));
    }

    #[test]
    fn retention_is_bounded_and_counted() {
        let lines: Vec<ScriptLine> = (0..RETAIN_LINES + 25)
            .map(|i| line(0, &format!("l{i}")))
            .collect();
        let mut a = act(Script::running(lines), 0);
        a.advance(0);
        assert_eq!(a.output.len(), RETAIN_LINES);
        assert_eq!(a.dropped, 25);
        assert_eq!(
            a.output[0].1, "l25",
            "the oldest lines are the ones dropped"
        );
    }

    #[test]
    fn raw_streams_normalise_like_the_executor() {
        assert_eq!(normalize_line(b"done\r\n"), "done");
        assert_eq!(
            normalize_line(b"\x1b[31mred\x1b[0m\n"),
            "\u{1b}[31mred\u{1b}[0m"
        );
        assert_eq!(normalize_line(b"bad \xff byte"), "bad \u{fffd} byte");
        assert_eq!(
            split_stream(b"one\r\ntwo\nthree"),
            vec!["one", "two", "three"],
            "the final unterminated fragment is emitted"
        );
        assert!(split_stream(b"").is_empty());
    }

    #[test]
    fn key_mapping_matches_the_terminal_contract() {
        use junie_tui::core::event::Key;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers as M};
        let k = |code, mods| Key { code, mods };
        assert_eq!(key_bytes(&k(KeyCode::Enter, M::NONE)), Some(vec![b'\r']));
        assert_eq!(key_bytes(&k(KeyCode::Backspace, M::NONE)), Some(vec![0x7f]));
        assert_eq!(key_bytes(&k(KeyCode::Tab, M::NONE)), Some(vec![b'\t']));
        assert_eq!(key_bytes(&k(KeyCode::Char('c'), M::CONTROL)), Some(vec![3]));
        assert_eq!(key_bytes(&k(KeyCode::Char('d'), M::CONTROL)), Some(vec![4]));
        assert_eq!(key_bytes(&k(KeyCode::Char('@'), M::CONTROL)), Some(vec![0]));
        assert_eq!(
            key_bytes(&k(KeyCode::Char('A'), M::SHIFT)),
            Some(b"A".to_vec())
        );
        assert_eq!(
            key_bytes(&k(KeyCode::Char('é'), M::NONE)),
            Some("é".as_bytes().to_vec())
        );
        assert_eq!(
            key_bytes(&k(KeyCode::Up, M::NONE)),
            None,
            "navigation keys are not forwarded"
        );
        assert_eq!(
            key_bytes(&k(KeyCode::Esc, M::NONE)),
            None,
            "Escape is reserved"
        );
        assert_eq!(key_bytes(&k(KeyCode::Char('x'), M::ALT)), None);
    }
}
