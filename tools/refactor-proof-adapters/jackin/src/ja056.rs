//! JA-056: pane/tab close attempts (ground truth).
//!
//! Source audit: `Ctrl-B,x` and `Ctrl-B,&` are unbound in the pinned source —
//! they open no confirmation, close nothing, and leave the topology and route
//! unchanged, including in the single-instance world. Tab closing is served by
//! the tab context menu (`JA-052`).

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA056_ID: &str = "JA-056";
/// JA-056 sizes.
pub const JA056_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Close variants in contract order.
pub const JA056_VARIANTS: [char; 2] = ['x', '&'];

/// One size of JA-056.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja056Capture {
    /// Tabs/panes before the attempts.
    pub before: (usize, usize),
    /// One frame per close variant.
    pub variants: Vec<ObservedFrame>,
    /// Tabs/panes after each variant.
    pub after: Vec<(usize, usize)>,
    /// One frame per solo-world close variant.
    pub solo_variants: Vec<ObservedFrame>,
    /// Tabs/panes after each solo variant.
    pub solo_after: Vec<(usize, usize)>,
    /// Tabs/panes of the solo world before the attempts.
    pub solo_before: (usize, usize),
}

/// Capture JA-056 at both listed sizes.
#[must_use]
pub fn ja056_close_attempts() -> Vec<Ja056Capture> {
    JA056_SIZES.into_iter().map(capture_size).collect()
}

fn topology(session: &DirectSession) -> (usize, usize) {
    session
        .app()
        .world
        .instances
        .iter()
        .find(|instance| instance.status == jackin_app::domain::instance::InstanceStatus::Running)
        .and_then(|instance| session.app().world.daemons.get(&instance.id))
        .map_or((0, 0), |daemon| (daemon.tabs.len(), daemon.panes.len()))
}

fn attempt(
    scenario: Scenario,
    viewport: Viewport,
    key: char,
    name: &str,
) -> (ObservedFrame, (usize, usize)) {
    let mut session = DirectSession::fresh(
        JA056_ID,
        scenario,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.ctrl('b');
    session.key(KeyCode::Char(key));
    let frame = session.observe(name);
    (frame, topology(&session))
}

fn capture_size(viewport: Viewport) -> Ja056Capture {
    let before = topology(&DirectSession::fresh(
        JA056_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    ));
    let mut variants = Vec::new();
    let mut after = Vec::new();
    for (index, key) in JA056_VARIANTS.into_iter().enumerate() {
        let (frame, topo) = attempt(
            Scenario::CapsuleMulti,
            viewport,
            key,
            &format!("variant-{index}"),
        );
        variants.push(frame);
        after.push(topo);
    }
    let solo_before = topology(&DirectSession::fresh(
        JA056_ID,
        Scenario::OutroLast,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    ));
    let mut solo_variants = Vec::new();
    let mut solo_after = Vec::new();
    for (index, key) in JA056_VARIANTS.into_iter().enumerate() {
        let (frame, topo) = attempt(Scenario::OutroLast, viewport, key, &format!("solo-{index}"));
        solo_variants.push(frame);
        solo_after.push(topo);
    }
    Ja056Capture {
        before,
        variants,
        after,
        solo_variants,
        solo_after,
        solo_before,
    }
}
