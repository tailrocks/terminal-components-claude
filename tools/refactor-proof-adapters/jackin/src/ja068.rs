//! JA-068: the CLI contract.
//!
//! In-process capture of the CLI's pure policy: every scenario and motion
//! name round-trips through `from_name`, unknown names parse to nothing,
//! the default construction is first-use/full/frame-zero on the intro,
//! and `--frame 45` constructs exactly frame 45. The spawn half — help,
//! invalid and missing reference options, unknown-argument tolerance,
//! help/error precedence, `NO_COLOR` equivalence, and the q/Ctrl-C
//! terminal-restore lane — runs in the test against the real
//! `jackin-preview` binary (built on demand), mirroring the pinned
//! `cli_contract` suite; the PTY restore lane itself belongs to the
//! visual suite.

use jackin_app::{Motion, Scenario};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA068_ID: &str = "JA-068";
/// JA-068 sizes.
pub const JA068_SIZES: [Viewport; 1] = [Viewport::new(120, 40)];

/// The `--frame` value pinned by the reference CLI tests.
pub const JA068_PINNED_FRAME: u64 = 45;

/// One size of JA-068.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja068Capture {
    /// All eight scenario names round-trip; unknown names fail.
    pub scenario_names: bool,
    /// All three motion names round-trip; unknown names fail.
    pub motion_names: bool,
    /// Default construction is first-use/full/zero on the intro.
    pub default_construction: bool,
    /// The pinned frame was constructed exactly.
    pub pinned_frame: u64,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn scenario_names_ok() -> bool {
    let mut names: Vec<&str> = Scenario::ALL
        .iter()
        .map(|scenario| scenario.name())
        .collect();
    names.sort_unstable();
    names.dedup();
    names.len() == Scenario::ALL.len()
        && Scenario::ALL
            .iter()
            .all(|scenario| Scenario::from_name(scenario.name()) == Some(*scenario))
        && Scenario::from_name("bogus-scenario").is_none()
        && Scenario::from_name("").is_none()
}

fn motion_names_ok() -> bool {
    use Motion::{Full, Paused, Reduced};
    Motion::from_name("full") == Some(Full)
        && Motion::from_name("reduced") == Some(Reduced)
        && Motion::from_name("paused") == Some(Paused)
        && Motion::from_name("parpaused").is_none()
        && Motion::from_name("").is_none()
}

fn capture_size(viewport: Viewport) -> Ja068Capture {
    let mut frames = Vec::new();
    let default = DirectSession::fresh(
        JA068_ID,
        Scenario::FirstUse,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let default_frame = default.observe("default");
    let default_construction = default_frame.route == "intro"
        && default_frame.construct_frame == 0
        && default_frame.scenario == "first-use"
        && default_frame.motion == "full";
    frames.push(default_frame);

    let pinned = DirectSession::fresh(
        JA068_ID,
        Scenario::FirstUse,
        Motion::Paused,
        JA068_PINNED_FRAME,
        viewport,
        CaptureColor::TrueColor,
    );
    let pinned_frame = pinned.observe("pinned");
    let constructed = pinned_frame.construct_frame;
    frames.push(pinned_frame);

    Ja068Capture {
        scenario_names: scenario_names_ok(),
        motion_names: motion_names_ok(),
        default_construction,
        pinned_frame: constructed,
        frames,
    }
}

/// Capture JA-068 at all listed sizes.
#[must_use]
pub fn ja068_cli_contract() -> Vec<Ja068Capture> {
    JA068_SIZES.iter().map(|size| capture_size(*size)).collect()
}
