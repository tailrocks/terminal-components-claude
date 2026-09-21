//! JA-045: replay `cockpit_resolves_every_effective_account_for_the_container`.

use jackin_app::{Motion, Route, Scenario};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA045_ID: &str = "JA-045";
/// JA-045 sizes.
pub const JA045_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-045.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja045Capture {
    /// Cockpit frame showing the two-account credential line.
    pub credentials: ObservedFrame,
    /// Whether the credential line listed both accounts.
    pub found_line: bool,
    /// Final capsule frame.
    pub arrived: ObservedFrame,
    /// Accounts attached to the running instance.
    pub instance_accounts: usize,
}

/// Replay JA-045 at both listed sizes.
#[must_use]
pub fn ja045_effective_accounts() -> Vec<Ja045Capture> {
    JA045_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja045Capture {
    let mut session = DirectSession::fresh(
        JA045_ID,
        Scenario::LaunchRunning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let mut found_line = false;
    for _ in 0..80 {
        session.ticks(5);
        if session.count("accounts · Claude ·") > 0
            && session.count("Claude · Work") > 0
            && session.count("Claude · Personal") > 0
        {
            found_line = true;
            break;
        }
        if session.app().route() != Route::Cockpit {
            break;
        }
    }
    let credentials = session.observe("credentials");
    for _ in 0..60 {
        session.ticks(10);
        if session.app().route() != Route::Cockpit {
            break;
        }
    }
    session.ticks(15);
    let arrived = session.observe("arrived");
    let instance_accounts = session
        .app()
        .world
        .instances
        .iter()
        .find(|instance| instance.status == jackin_app::domain::instance::InstanceStatus::Running)
        .map_or(0, |instance| instance.accounts.len());
    Ja045Capture {
        credentials,
        found_line,
        arrived,
        instance_accounts,
    }
}
