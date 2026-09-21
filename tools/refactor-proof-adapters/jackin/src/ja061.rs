//! JA-061: the outro exit ritual.
//!
//! Exit-and-keep on the outro-last world plays the warp, skips to the
//! caption on Enter, and quits on the closing Enter. The elapsed caption
//! is pinned by the world's `entered_at_ms` fixture, and stray keys during
//! the warp change nothing.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA061_ID: &str = "JA-061";
/// JA-061 sizes.
pub const JA061_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Exact outro caption for the outro-last world.
pub const JA061_OUTRO_CAPTION: &str = "You were in the Construct for 2 hours 14 minutes";
/// Pinned fixture instant behind the 2h14m caption.
pub const JA061_ENTERED_AT_MS: i64 = -8_040_000;

/// One size of JA-061.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja061Capture {
    /// The warp plays with no caption before the skip.
    pub warp_before_caption: bool,
    /// The world's entry fixture matches the pinned instant.
    pub entered_at_ms: Option<i64>,
    /// Stray keys during the warp left route and quit state alone.
    pub redundant_inert: bool,
    /// The skip reached the exact caption.
    pub caption_exact: bool,
    /// The closing Enter requested quit.
    pub quit_at_end: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn capture_size(viewport: Viewport) -> Ja061Capture {
    let mut frames = Vec::new();
    let mut session = DirectSession::fresh(
        JA061_ID,
        Scenario::OutroLast,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let entered_at_ms = session.app().world.arbiter.entered_at_ms;
    session.ctrl('q');
    session.key(KeyCode::Down);
    session.key(KeyCode::Down);
    session.key(KeyCode::Enter);
    let warp = session.observe("warp");
    let warp_before_caption =
        warp.route == "outro" && !warp.text.contains("You were in the Construct for");
    frames.push(warp);

    session.key(KeyCode::Char('x'));
    session.key(KeyCode::Down);
    let redundant = session.observe("redundant");
    let redundant_inert = redundant.route == "outro" && !redundant.quit;
    frames.push(redundant);

    session.key(KeyCode::Enter);
    session.ticks(25);
    let caption = session.observe("caption");
    let caption_exact = caption.route == "outro" && caption.text.contains(JA061_OUTRO_CAPTION);
    frames.push(caption);

    session.key(KeyCode::Enter);
    let end = session.observe("end");
    let quit_at_end = end.quit;
    frames.push(end);

    Ja061Capture {
        warp_before_caption,
        entered_at_ms,
        redundant_inert,
        caption_exact,
        quit_at_end,
        frames,
    }
}

/// Capture JA-061 at both listed sizes.
#[must_use]
pub fn ja061_outro_ritual() -> Vec<Ja061Capture> {
    JA061_SIZES.iter().map(|size| capture_size(*size)).collect()
}
