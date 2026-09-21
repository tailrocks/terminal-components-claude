//! JA-062: launch cockpit, handoff, and launch failure.
//!
//! A running launch opens the cockpit, passes through the transient
//! handoff ("Opening Capsule"), and settles into a capsule with
//! transcript. A failed launch returns to the manager with an exact
//! failure notice and no retry/cancel surface open; `r` and Esc leave
//! that state alone. The Launch choose-agent route itself is covered by
//! the JA-069 journey replay (steps 18-20), the only pinned path that
//! reaches it.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA062_ID: &str = "JA-062";
/// JA-062 sizes.
pub const JA062_SIZES: [Viewport; 1] = [Viewport::new(120, 40)];

/// Exact failure notice for the launch-failure world.
pub const JA062_FAILURE_NOTICE: &str = "Launch failed · Network · The Construct network could not be attached · another instance is still running";

/// One size of JA-062.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja062Capture {
    /// The running launch opened on the cockpit.
    pub cockpit_first: bool,
    /// The transient handoff showed its caption.
    pub handoff_caption: bool,
    /// The launch settled into a capsule with transcript.
    pub capsule_settled: bool,
    /// The failure notice is exact.
    pub failure_exact: bool,
    /// No dialog, retry, or cancel surface is open after the failure.
    pub no_retry_surface: bool,
    /// `r` and Esc left the failure state alone.
    pub failure_keys_inert: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn capture_size(viewport: Viewport) -> Ja062Capture {
    let mut frames = Vec::new();

    let mut running = DirectSession::fresh(
        JA062_ID,
        Scenario::LaunchRunning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let cockpit = running.observe("cockpit");
    let cockpit_first = cockpit.route == "cockpit";
    frames.push(cockpit);
    let mut handoff_caption = false;
    for _ in 0..300 {
        running.ticks(1);
        if running.app().route() == jackin_app::Route::Handoff {
            let handoff = running.observe("handoff");
            handoff_caption =
                handoff.route == "handoff" && handoff.text.contains("Opening Capsule");
            frames.push(handoff);
            break;
        }
        if running.app().route() == jackin_app::Route::Capsule {
            break;
        }
    }
    for _ in 0..80 {
        running.ticks(10);
        if running.app().route() == jackin_app::Route::Capsule {
            break;
        }
    }
    running.ticks(15);
    let settled = running.observe("settled");
    let capsule_settled = settled.route == "capsule"
        && (settled.text.contains("Refactor")
            || settled.text.contains("cargo")
            || settled.text.contains("batch 4001"));
    frames.push(settled);

    let mut failed = DirectSession::fresh(
        JA062_ID,
        Scenario::LaunchFailure,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    for _ in 0..60 {
        failed.ticks(10);
    }
    let failure = failed.observe("failure");
    let failure_exact = failure.route == "manager" && failure.text.contains(JA062_FAILURE_NOTICE);
    let no_retry_surface = !failed.is_open(jackin_app::LAUNCH_DIALOG)
        && !failed.is_open(jackin_app::LAUNCH_RETRY)
        && !failed.is_open(jackin_app::LAUNCH_CANCEL);
    frames.push(failure);
    failed.key(KeyCode::Char('r'));
    failed.key(KeyCode::Esc);
    let after_keys = failed.observe("failure-keys");
    let failure_keys_inert =
        after_keys.route == "manager" && after_keys.text.contains(JA062_FAILURE_NOTICE);
    frames.push(after_keys);

    Ja062Capture {
        cockpit_first,
        handoff_caption,
        capsule_settled,
        failure_exact,
        no_retry_surface,
        failure_keys_inert,
        frames,
    }
}

/// Capture JA-062 at all listed sizes.
#[must_use]
pub fn ja062_launch_and_failure() -> Vec<Ja062Capture> {
    JA062_SIZES.iter().map(|size| capture_size(*size)).collect()
}
