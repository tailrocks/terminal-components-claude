//! JA-066: the terminal-too-small state.
//!
//! Below 72x20 the app paints a centered too-small notice with the exact
//! have/want dimensions instead of the route, and route keys are blocked:
//! Down/Enter move nothing. Only `q` quits from the small state — Ctrl-C
//! is inert there — and growing to exactly 72x20 (or back to 80x24)
//! restores the manager.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA066_ID: &str = "JA-066";
/// JA-066 sizes.
pub const JA066_SIZES: [Viewport; 1] = [Viewport::new(120, 40)];

/// One size of JA-066.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja066Capture {
    /// The 60x18 notice names the exact dimensions.
    pub notice_60x18: bool,
    /// The 71x19 notice names the exact dimensions.
    pub notice_71x19: bool,
    /// Down/Enter moved nothing while small.
    pub keys_blocked: bool,
    /// `q` quit from the small state.
    pub quit_on_q: bool,
    /// Ctrl-C left the small state alone.
    pub ctrl_c_inert: bool,
    /// Exactly 72x20 restores the manager.
    pub exact_72x20: bool,
    /// Growing back to 80x24 restores the manager.
    pub restored_80x24: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn fresh() -> DirectSession {
    DirectSession::fresh(
        JA066_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        Viewport::new(120, 40),
        CaptureColor::TrueColor,
    )
}

fn is_small(frame: &ObservedFrame, want: &str) -> bool {
    frame.text.contains("Terminal too small") && frame.text.contains(want)
}

fn capture_size(viewport: Viewport) -> Ja066Capture {
    let _ = viewport;
    let mut frames = Vec::new();

    let mut session = fresh();
    session.resize(60, 18);
    let small_60 = session.observe("small-60x18");
    let notice_60x18 = is_small(&small_60, "Need 72×20, have 60×18");
    frames.push(small_60);

    session.resize(71, 19);
    let small_71 = session.observe("small-71x19");
    let notice_71x19 = is_small(&small_71, "Need 72×20, have 71×19");
    frames.push(small_71);

    session.resize(60, 18);
    let selected_before = session.observe("before-keys").selected_row;
    session.key(KeyCode::Down);
    session.key(KeyCode::Enter);
    let after_keys = session.observe("blocked-keys");
    let keys_blocked = after_keys.selected_row == selected_before && after_keys.route == "manager";
    frames.push(after_keys);

    session.resize(72, 20);
    let exact = session.observe("exact-72x20");
    let exact_72x20 = exact.route == "manager" && exact.text.contains("Workspaces");
    frames.push(exact);

    session.resize(80, 24);
    let restored = session.observe("restored-80x24");
    let restored_80x24 = restored.route == "manager" && restored.text.contains("Workspaces");
    frames.push(restored);

    let mut quitter = fresh();
    quitter.resize(60, 18);
    quitter.key(KeyCode::Char('q'));
    let quit_q = quitter.observe("quit-q");
    let quit_on_q = quit_q.quit;
    frames.push(quit_q);

    let mut interrupter = fresh();
    interrupter.resize(60, 18);
    interrupter.ctrl('c');
    let quit_c = interrupter.observe("quit-ctrl-c");
    let ctrl_c_inert =
        !quit_c.quit && quit_c.route == "manager" && quit_c.text.contains("Terminal too small");
    frames.push(quit_c);

    Ja066Capture {
        notice_60x18,
        notice_71x19,
        keys_blocked,
        quit_on_q,
        ctrl_c_inert,
        exact_72x20,
        restored_80x24,
        frames,
    }
}

/// Capture JA-066 at all listed sizes.
#[must_use]
pub fn ja066_too_small() -> Vec<Ja066Capture> {
    JA066_SIZES.iter().map(|size| capture_size(*size)).collect()
}
