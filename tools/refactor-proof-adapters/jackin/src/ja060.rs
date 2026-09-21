//! JA-060: motion policies (full, reduced, paused).
//!
//! Explicit CLI motion always wins; otherwise `JACKIN_NO_MOTION` selects
//! the reduced path and the default is full motion. Reduced motion reaches
//! the same routes with fewer painted frames, paused motion never advances
//! on ticks, and `--frame` pins the exact construction frame. The
//! `JACKIN_NO_MOTION` empty/`"0"`-is-falsy string rule lives in the
//! private CLI parser and is pinned by its in-crate tests plus the
//! `cli_contract` spawn suite; the pure [`Motion::resolve`] matrix below
//! is the adapter-owned half.

use jackin_app::{Motion, Scenario};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA060_ID: &str = "JA-060";
/// JA-060 sizes.
pub const JA060_SIZES: [Viewport; 1] = [Viewport::new(120, 40)];

/// Pinned construction frame for the `--frame` leg.
pub const JA060_PINNED_FRAME: u64 = 45;

/// One size of JA-060.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja060Capture {
    /// The full [`Motion::resolve`] matrix matches the reference.
    pub resolve_matrix: bool,
    /// Full and reduced motion reach the manager from the intro.
    pub routes_match: bool,
    /// Full and reduced motion paint different intro frames.
    pub frames_differ: bool,
    /// Paused motion ignores ticks entirely.
    pub paused_frozen: bool,
    /// The pinned frame was constructed exactly.
    pub pinned_frame: u64,
    /// Frame 45 paints the same pre-phrase canvas as frame zero.
    pub pin45_static: bool,
    /// A second-phrase frame paints a different message.
    pub pin_phrase_differs: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn resolve_matrix_ok() -> bool {
    use Motion::{Full, Paused, Reduced};
    let matrix = [
        (Some(Full), false, Full),
        (Some(Full), true, Full),
        (Some(Reduced), false, Reduced),
        (Some(Reduced), true, Reduced),
        (Some(Paused), false, Paused),
        (Some(Paused), true, Paused),
        (None, false, Full),
        (None, true, Reduced),
    ];
    matrix
        .iter()
        .all(|(cli, env, expected)| Motion::resolve(*cli, *env) == *expected)
}

fn capture_size(viewport: Viewport) -> Ja060Capture {
    let mut frames = Vec::new();

    let mut full = DirectSession::fresh(
        JA060_ID,
        Scenario::FirstUse,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let mut reduced = DirectSession::fresh(
        JA060_ID,
        Scenario::FirstUse,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let full_frame = full.observe("full-intro");
    let reduced_frame = reduced.observe("reduced-intro");
    let frames_differ = full_frame.digest != reduced_frame.digest;
    frames.push(full_frame);
    frames.push(reduced_frame);
    full.ticks(3);
    reduced.ticks(3);
    // Full motion skips one phase per Enter; reduced motion clears at once.
    full.key(junie_tui::KeyCode::Enter);
    full.key(junie_tui::KeyCode::Enter);
    reduced.key(junie_tui::KeyCode::Enter);
    let (full_skipped, reduced_skipped) = (
        full.observe("full-skipped"),
        reduced.observe("reduced-skipped"),
    );
    let routes_match = full_skipped.route == "manager" && reduced_skipped.route == "manager";
    frames.push(full_skipped);

    let mut paused = DirectSession::fresh(
        JA060_ID,
        Scenario::FirstUse,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let paused_zero = paused.observe("paused-zero");
    let digest_zero = paused_zero.digest;
    frames.push(paused_zero);
    paused.ticks(50);
    let paused_ticked = paused.observe("paused-ticked");
    let paused_frozen = paused_ticked.route == "intro" && paused_ticked.digest == digest_zero;
    frames.push(paused_ticked);

    let pinned = DirectSession::fresh(
        JA060_ID,
        Scenario::FirstUse,
        Motion::Paused,
        JA060_PINNED_FRAME,
        viewport,
        CaptureColor::TrueColor,
    );
    let pinned_frame = pinned.observe("pinned");
    let pin45_static = pinned_frame.digest == digest_zero;
    let constructed = pinned_frame.construct_frame;
    frames.push(pinned_frame);

    let phrase_two = DirectSession::fresh(
        JA060_ID,
        Scenario::FirstUse,
        Motion::Paused,
        jackin_app::rain::P1_LEN,
        viewport,
        CaptureColor::TrueColor,
    );
    let phrase_frame = phrase_two.observe("pinned-phrase-two");
    let pin_phrase_differs =
        phrase_frame.digest != digest_zero && phrase_frame.text.contains("Host stays outside…");
    frames.push(phrase_frame);

    Ja060Capture {
        resolve_matrix: resolve_matrix_ok(),
        routes_match,
        frames_differ,
        paused_frozen,
        pinned_frame: constructed,
        pin45_static,
        pin_phrase_differs,
        frames,
    }
}

/// Capture JA-060 at all listed sizes.
#[must_use]
pub fn ja060_motion_policies() -> Vec<Ja060Capture> {
    JA060_SIZES.iter().map(|size| capture_size(*size)).collect()
}
