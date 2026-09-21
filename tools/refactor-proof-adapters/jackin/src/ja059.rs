//! JA-059: the intro entry ritual.
//!
//! The first-use world opens on the intro, which is deterministic for a
//! pinned seed and frame: two fresh sessions paint identical digests and
//! text, including after identical ticks. Enter skips to the manager once
//! the ritual has started, `q` quits from the intro, an idle intro
//! advances on its own, and `?` is ignored while the ritual owns the
//! screen.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA059_ID: &str = "JA-059";
/// JA-059 sizes.
pub const JA059_SIZES: [Viewport; 3] = [
    Viewport::new(72, 20),
    Viewport::new(80, 24),
    Viewport::new(120, 40),
];

/// One size of JA-059.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja059Capture {
    /// Enter after three ticks reached the manager.
    pub enter_skips: bool,
    /// Two fresh sessions paint identical frames, before and after ticks.
    pub deterministic: bool,
    /// `q` on the intro requested quit.
    pub quit_on_q: bool,
    /// An idle intro advanced to the manager without input.
    pub idle_advances: bool,
    /// `?` left the intro frame bit-identical.
    pub question_ignored: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn fresh(viewport: Viewport) -> DirectSession {
    DirectSession::fresh(
        JA059_ID,
        Scenario::FirstUse,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    )
}

fn capture_size(viewport: Viewport) -> Ja059Capture {
    let mut frames = Vec::new();

    let mut session = fresh(viewport);
    session.ticks(3);
    // Full motion skips one phase per Enter: phrases, then the warp.
    session.key(KeyCode::Enter);
    session.key(KeyCode::Enter);
    let skipped = session.observe("skipped");
    let enter_skips = skipped.route == "manager";
    frames.push(skipped);

    let first = fresh(viewport);
    let second = fresh(viewport);
    let (frame_a, frame_b) = (first.observe("det-a"), second.observe("det-b"));
    let mut ticked_a = fresh(viewport);
    let mut ticked_b = fresh(viewport);
    ticked_a.ticks(7);
    ticked_b.ticks(7);
    let (ticked_frame_a, ticked_frame_b) = (
        ticked_a.observe("det-ticks-a"),
        ticked_b.observe("det-ticks-b"),
    );
    let deterministic = frame_a.digest == frame_b.digest
        && frame_a.text == frame_b.text
        && ticked_frame_a.digest == ticked_frame_b.digest
        && ticked_frame_a.text == ticked_frame_b.text;
    frames.push(frame_a);

    let mut quitter = fresh(viewport);
    quitter.key(KeyCode::Char('q'));
    let quit_frame = quitter.observe("quit");
    let quit_on_q = quit_frame.quit;
    frames.push(quit_frame);

    let mut idle = fresh(viewport);
    for _ in 0..400 {
        idle.ticks(1);
        if idle.app().route() != jackin_app::Route::Intro {
            break;
        }
    }
    let idle_frame = idle.observe("idle");
    let idle_advances = idle_frame.route == "manager";
    frames.push(idle_frame);

    let mut question = fresh(viewport);
    let before = question.observe("before-question");
    let digest_before = before.digest;
    question.key(KeyCode::Char('?'));
    let after = question.observe("after-question");
    let question_ignored = after.digest == digest_before && after.route == "intro";
    frames.push(after);

    Ja059Capture {
        enter_skips,
        deterministic,
        quit_on_q,
        idle_advances,
        question_ignored,
        frames,
    }
}

/// Capture JA-059 at all listed sizes.
#[must_use]
pub fn ja059_intro_ritual() -> Vec<Ja059Capture> {
    JA059_SIZES.iter().map(|size| capture_size(*size)).collect()
}
