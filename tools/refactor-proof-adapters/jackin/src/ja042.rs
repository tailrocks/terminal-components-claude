//! JA-042: replay the launch-failure journey, plus the no-instance variant.
//!
//! With another instance running, the failure returns to the manager with
//! `still running` status. With no running instance, the failure stays in the
//! cockpit showing the stage details; `Esc` acknowledges it.

use jackin_app::domain::instance::InstanceStatus;
use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA042_ID: &str = "JA-042";

/// One size of JA-042.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja042Capture {
    /// Failure details in the cockpit.
    pub failed: ObservedFrame,
    /// After `Esc` acknowledges the failure.
    pub acknowledged: ObservedFrame,
    /// Failure details with no running instance.
    pub solo_failed: ObservedFrame,
    /// After `Esc` acknowledges the solo failure.
    pub solo_acknowledged: ObservedFrame,
    /// Running-instance count in the solo variant.
    pub solo_running: usize,
}

/// Replay JA-042 at every JA-001 size.
#[must_use]
pub fn ja042_launch_failure() -> Vec<Ja042Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn run_to_failure(session: &mut DirectSession) {
    for _ in 0..60 {
        session.ticks(10);
        if session.app().route() != jackin_app::Route::Cockpit || session.count("Launch failed") > 0
        {
            break;
        }
    }
}

fn capture_size(viewport: Viewport) -> Ja042Capture {
    let mut session = DirectSession::fresh(
        JA042_ID,
        Scenario::LaunchFailure,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    run_to_failure(&mut session);
    let failed = session.observe("failed");
    session.key(KeyCode::Esc);
    let acknowledged = session.observe("acknowledged");

    let mut solo = DirectSession::fresh(
        JA042_ID,
        Scenario::LaunchFailure,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    for instance in solo.app_mut().world.instances.iter_mut() {
        instance.status = InstanceStatus::CleanExited;
    }
    solo.app_mut().world.sync_arbiter();
    let solo_running = solo.app().world.running_count();
    run_to_failure(&mut solo);
    let solo_failed = solo.observe("solo-failed");
    solo.key(KeyCode::Esc);
    let solo_acknowledged = solo.observe("solo-acknowledged");

    Ja042Capture {
        failed,
        acknowledged,
        solo_failed,
        solo_acknowledged,
        solo_running,
    }
}
