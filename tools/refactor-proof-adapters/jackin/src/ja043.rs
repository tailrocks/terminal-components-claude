//! JA-043: recoverable credentials and modeled-only blocked sidecar.
//!
//! Both plans are driven through the production
//! [`LaunchRun`](jackin_app::sim::launch::LaunchRun) state machine. The
//! `BlockedSidecar` plan is modeled-only: no runtime producer constructs it,
//! so the capture labels the machine trace accordingly and maps the
//! contract's `Enter`/`Esc` dismissal steps onto the machine's terminal hold
//! (further ticks change nothing) and `cancel()`.

use jackin_app::domain::agent::Agent;
use jackin_app::sim::launch::{LaunchPlan, LaunchRun};
use jackin_app::{Motion, RunId, Scenario};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA043_ID: &str = "JA-043";
/// JA-043 sizes.
pub const JA043_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Bound on machine ticks (both plans settle far earlier).
pub const JA043_TICK_BOUND: usize = 2_000;

/// One size of JA-043.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja043Capture {
    /// Debug event trace of the credentials-locked run.
    pub locked_trace: Vec<String>,
    /// Whether the locked run reached `Ready` after the retry.
    pub locked_done: bool,
    /// Whether the run held on the credential error before the retry.
    pub locked_held: bool,
    /// Debug event trace of the blocked-sidecar run.
    pub blocked_trace: Vec<String>,
    /// Stage at which the modeled block occurred.
    pub blocked_at: Option<String>,
    /// Whether further ticks leave the blocked run unchanged.
    pub blocked_terminal: bool,
    /// Whether `cancel()` dismissed the blocked run.
    pub blocked_cancelled: bool,
    /// In-app cockpit context frame from the clean launch world.
    pub cockpit: ObservedFrame,
}

/// Drive a credentials-locked run to completion through the retry.
#[must_use]
pub fn ja043_locked_run() -> (Vec<String>, bool, bool) {
    let mut run = LaunchRun::new(
        LaunchPlan::CredentialsLocked,
        Agent::ClaudeCode,
        "c",
        RunId::new(1),
    );
    let mut trace = Vec::new();
    let mut held = false;
    for _ in 0..JA043_TICK_BOUND {
        if run.credential_hold {
            held = true;
            run.retry_credentials();
        }
        for event in run.advance() {
            trace.push(format!("{event:?}"));
        }
        if run.is_terminal() {
            break;
        }
    }
    let done = run.done;
    (trace, done, held)
}

/// Drive a blocked-sidecar run to its modeled terminal state.
#[must_use]
pub fn ja043_blocked_run() -> (Vec<String>, Option<String>, bool, bool) {
    let mut run = LaunchRun::new(
        LaunchPlan::BlockedSidecar,
        Agent::ClaudeCode,
        "c",
        RunId::new(1),
    );
    let mut trace = Vec::new();
    for _ in 0..JA043_TICK_BOUND {
        for event in run.advance() {
            trace.push(format!("{event:?}"));
        }
        if run.is_terminal() {
            break;
        }
    }
    let blocked_at = run.blocked_at.map(|stage| format!("{stage:?}"));
    // The contract's dismissal keys have no machine binding; further ticks
    // must change nothing, then `cancel()` dismisses the modeled run.
    let before = format!("{run:?}");
    for _ in 0..10 {
        let _ = run.advance();
    }
    let blocked_terminal = format!("{run:?}") == before;
    run.cancel();
    let blocked_cancelled = run.cancelled;
    (trace, blocked_at, blocked_terminal, blocked_cancelled)
}

/// Capture JA-043 at both listed sizes.
#[must_use]
pub fn ja043_launch_blocks() -> Vec<Ja043Capture> {
    JA043_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja043Capture {
    let (locked_trace, locked_done, locked_held) = ja043_locked_run();
    let (blocked_trace, blocked_at, blocked_terminal, blocked_cancelled) = ja043_blocked_run();
    let mut session = DirectSession::fresh(
        JA043_ID,
        Scenario::LaunchRunning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.ticks(10);
    let cockpit = session.observe("cockpit");
    Ja043Capture {
        locked_trace,
        locked_done,
        locked_held,
        blocked_trace,
        blocked_at,
        blocked_terminal,
        blocked_cancelled,
        cockpit,
    }
}
