//! JA-040: full launch stage walk with per-stage checkpoints.
//!
//! Ticks a fresh `launch-running` world one helper tick at a time, capturing
//! a frame at every stage-frontier change, the transient handoff, and the
//! final capsule arrival.

use jackin_app::{Motion, Route, Scenario};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA040_ID: &str = "JA-040";

/// Bound on single-tick iterations (the clean run finishes far earlier).
pub const JA040_TICK_BOUND: usize = 400;

/// One size of JA-040.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja040Capture {
    /// Initial cockpit frame.
    pub initial: ObservedFrame,
    /// One frame per stage-frontier change, in order.
    pub stage_frames: Vec<ObservedFrame>,
    /// Stage frontier trace (`index:label:state`) at each change.
    pub stage_trace: Vec<String>,
    /// The transient handoff frame, when observed.
    pub handoff: Option<ObservedFrame>,
    /// Capsule arrival plus `T(12)`.
    pub arrived: ObservedFrame,
    /// Whether the run reached the capsule within the bound.
    pub completed: bool,
}

/// Capture JA-040 at every JA-001 size.
#[must_use]
pub fn ja040_launch_stages() -> Vec<Ja040Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn frontier(session: &DirectSession) -> Option<(usize, String, u64)> {
    let run = session.app().launch()?;
    let index = run.current?;
    Some((index, format!("{:?}", run.states), run.tick))
}

fn capture_size(viewport: Viewport) -> Ja040Capture {
    let mut session = DirectSession::fresh(
        JA040_ID,
        Scenario::LaunchRunning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let initial = session.observe("initial");
    let mut stage_frames = Vec::new();
    let mut stage_trace = Vec::new();
    let mut handoff = None;
    let mut last_frontier: Option<(usize, String)> = None;
    let mut completed = false;
    for step in 0..JA040_TICK_BOUND {
        session.ticks(1);
        let route = session.app().route();
        if route == Route::Handoff && handoff.is_none() {
            handoff = Some(session.observe("handoff"));
        }
        if route == Route::Capsule {
            completed = true;
            break;
        }
        if route != Route::Cockpit && route != Route::Launch && route != Route::Handoff {
            break;
        }
        if route == Route::Handoff {
            continue;
        }
        if let Some((index, states, tick)) = frontier(&session) {
            let key = (index, states.clone());
            if last_frontier.as_ref() != Some(&key) {
                stage_trace.push(format!("{index}:{tick}:{states}"));
                stage_frames.push(session.observe(&format!("stage-{step}")));
                last_frontier = Some(key);
            }
        }
    }
    session.ticks(12);
    let arrived = session.observe("arrived");
    Ja040Capture {
        initial,
        stage_frames,
        stage_trace,
        handoff,
        arrived,
        completed,
    }
}
