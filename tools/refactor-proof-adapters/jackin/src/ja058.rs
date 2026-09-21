//! JA-058: detach/reconnect with one outro, and still-inside feedback.
//!
//! Replays the pinned `detach_reconnect_and_final_exit_plays_one_outro` and
//! `still_inside_feedback_when_other_instances_remain` transcripts: detach
//! lands on the manager with a Detached notice, reconnect restores the
//! capsule, exit-and-keep plays the outro exactly once with the elapsed
//! caption, and leaving one instance while another runs reports
//! Still-inside instead of an outro.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA058_ID: &str = "JA-058";
/// JA-058 sizes.
pub const JA058_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Exact outro caption for the outro-last world.
pub const JA058_OUTRO_CAPTION: &str = "You were in the Construct for 2 hours 14 minutes";

/// One size of JA-058.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja058Capture {
    /// Detach landed on the manager with a Detached notice.
    pub detached: bool,
    /// Reconnect restored the capsule route.
    pub reconnected: bool,
    /// Exit-and-keep reached the outro with the exact caption.
    pub outro_caption: bool,
    /// Final outro Enter requested quit.
    pub quit_after_outro: bool,
    /// Leaving one of several instances reported still-inside.
    pub still_inside: bool,
    /// Running count after the still-inside exit.
    pub still_inside_running: usize,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn exit_and_keep(session: &mut DirectSession) {
    session.ctrl('q');
    session.key(KeyCode::Down);
    session.key(KeyCode::Down);
    session.key(KeyCode::Enter);
}

fn capture_size(viewport: Viewport) -> Ja058Capture {
    let mut frames = Vec::new();

    // Detach leg on the outro-last world.
    let mut session = DirectSession::fresh(
        JA058_ID,
        Scenario::OutroLast,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.ctrl('b');
    session.key(KeyCode::Char('d'));
    let detached_frame = session.observe("detached");
    let detached = detached_frame.route == "manager" && detached_frame.text.contains("Detached");
    frames.push(detached_frame);
    session.key(KeyCode::Enter);
    let reconnected_frame = session.observe("reconnected");
    let reconnected = reconnected_frame.route == "capsule";
    frames.push(reconnected_frame);
    exit_and_keep(&mut session);
    session.key(KeyCode::Enter);
    session.ticks(25);
    let outro_frame = session.observe("outro-caption");
    let outro_caption =
        outro_frame.route == "outro" && outro_frame.text.contains(JA058_OUTRO_CAPTION);
    frames.push(outro_frame);
    session.key(KeyCode::Enter);
    let quit_frame = session.observe("quit");
    let quit_after_outro = quit_frame.quit;
    frames.push(quit_frame);

    // Still-inside leg on the capsule-multi world.
    let mut busy = DirectSession::fresh(
        JA058_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    exit_and_keep(&mut busy);
    let still_frame = busy.observe("still-inside");
    let still_inside =
        still_frame.route == "manager" && still_frame.text.contains("Still inside the Construct");
    let still_inside_running = busy.app().world.running_count();
    frames.push(still_frame);

    Ja058Capture {
        detached,
        reconnected,
        outro_caption,
        quit_after_outro,
        still_inside,
        still_inside_running,
        frames,
    }
}

/// Capture JA-058 at both listed sizes.
#[must_use]
pub fn ja058_detach_and_still_inside() -> Vec<Ja058Capture> {
    JA058_SIZES.iter().map(|size| capture_size(*size)).collect()
}
