//! JA-057: capsule tab/focus substrate (takeover ground truth).
//!
//! Source audit: no takeover queue, emitter, overlay, or compose API exists
//! in the pinned source — no `takeover` symbol in the app, the crates, or
//! the tests, and the tab struct carries no activity/badge/unread marker.
//! There is nothing to emit, order, evict, or dismiss, so this scenario
//! captures the observable substrate instead: tabbar order, same-tab focus
//! follow, other-tab focus preservation, detach round-trip survival, and
//! the launch-leg capsule with its transcript.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA057_ID: &str = "JA-057";
/// JA-057 sizes.
pub const JA057_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One daemon's tab/focus projection: key, tab count, active index, focus per tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonTabs {
    /// Daemon key.
    pub key: String,
    /// Open tab count.
    pub tabs: usize,
    /// Active tab index.
    pub active: usize,
    /// Focused pane per tab.
    pub focused: Vec<u64>,
}

/// One size of JA-057.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja057Capture {
    /// Painted tabbar row at baseline.
    pub tab_row: String,
    /// Clicking a second pane moved focus within the same tab.
    pub same_tab_followed: bool,
    /// The first tab kept its focus while tab two was active.
    pub other_tab_preserved: bool,
    /// Daemon tab/focus projection before the detach round-trip.
    pub before_detach: Vec<DaemonTabs>,
    /// Daemon tab/focus projection after reattach.
    pub after_detach: Vec<DaemonTabs>,
    /// Launch-leg route after settling.
    pub launch_route: String,
    /// Launch-leg capsule shows transcript output.
    pub launch_transcript: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn daemon_tabs(session: &DirectSession) -> Vec<DaemonTabs> {
    session
        .app()
        .world
        .daemons
        .iter()
        .map(|(key, daemon)| DaemonTabs {
            key: key.clone(),
            tabs: daemon.tabs.len(),
            active: daemon.active,
            focused: daemon.tabs.iter().map(|tab| tab.focused).collect(),
        })
        .collect()
}

fn tab_row_of(frame: &ObservedFrame) -> String {
    frame
        .text
        .lines()
        .find(|line| line.contains("Mix") || line.contains("Shell"))
        .unwrap_or("")
        .to_owned()
}

fn capture_size(viewport: Viewport) -> Ja057Capture {
    let mut session = DirectSession::fresh(
        JA057_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let mut frames = Vec::new();
    let base = session.observe("base");
    let tab_row = tab_row_of(&base);
    frames.push(base);

    // Same-tab focus follow: click the Shell pane, focus must move within tab 0.
    let focused_before = daemon_tabs(&session);
    if let Some((x, y)) = session.find("ls docs/adr") {
        session.click(x, y);
    }
    frames.push(session.observe("same-tab"));
    let focused_after_click = daemon_tabs(&session);
    let same_tab_followed = focused_before != focused_after_click
        && focused_after_click
            .first()
            .is_some_and(|daemon| daemon.active == 0);

    // Other-tab focus preservation: activate tab two, then return.
    if let Some((x, y)) = session.find("2 Shell") {
        session.click(x, y);
    }
    frames.push(session.observe("tab-two"));
    let tab_two = daemon_tabs(&session);
    let tab_zero_focus_while_away = tab_two
        .first()
        .and_then(|daemon| daemon.focused.first().copied());
    if let Some((x, y)) = session.find("1 Mix") {
        session.click(x, y);
    }
    frames.push(session.observe("tab-one"));
    let tab_back = daemon_tabs(&session);
    let home_focus_before = focused_after_click
        .first()
        .and_then(|daemon| daemon.focused.first().copied());
    let home_focus_after = tab_back
        .first()
        .and_then(|daemon| daemon.focused.first().copied());
    let other_tab_preserved = tab_two.first().is_some_and(|daemon| daemon.active == 1)
        && tab_zero_focus_while_away == home_focus_before
        && tab_back.first().is_some_and(|daemon| daemon.active == 0)
        && home_focus_after == home_focus_before;

    // Detach round-trip survival.
    let before_detach = daemon_tabs(&session);
    session.ctrl('b');
    session.key(KeyCode::Char('d'));
    frames.push(session.observe("detached"));
    session.key(KeyCode::Enter);
    frames.push(session.observe("reattached"));
    let after_detach = daemon_tabs(&session);

    // Launch leg: a launched instance settles into a capsule with transcript.
    let mut launch = DirectSession::fresh(
        JA057_ID,
        Scenario::LaunchRunning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    for _ in 0..80 {
        launch.ticks(10);
        if launch.app().route() == jackin_app::Route::Capsule {
            break;
        }
    }
    launch.ticks(15);
    let launch_frame = launch.observe("launch-capsule");
    let launch_route = launch_frame.route.clone();
    let launch_transcript = launch_frame.text.contains("Refactor")
        || launch_frame.text.contains("cargo")
        || launch_frame.text.contains("batch 4001");
    frames.push(launch_frame);

    Ja057Capture {
        tab_row,
        same_tab_followed,
        other_tab_preserved,
        before_detach,
        after_detach,
        launch_route,
        launch_transcript,
        frames,
    }
}

/// Capture JA-057 at both listed sizes.
#[must_use]
pub fn ja057_takeover_substrate() -> Vec<Ja057Capture> {
    JA057_SIZES.iter().map(|size| capture_size(*size)).collect()
}
