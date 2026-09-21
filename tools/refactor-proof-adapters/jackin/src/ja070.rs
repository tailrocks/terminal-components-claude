//! JA-070: the Construct boundary arbiter.
//!
//! Source audit: production calls exactly one arbiter method —
//! `complete_entry` at the entry ritual's close. The entry/exit decision
//! table (`request_entry`, `request_exit`, `release_entry`) is a pure
//! model pinned by its in-crate unit tests; the adapter replays it
//! through the world's public arbiter field and records each decision's
//! `Debug` rendering, since the arbiter module itself is private. The
//! `Unknown` branch is unreachable from outside the crate: constructing
//! the discovery failure requires naming the private `DiscoveryError`.
//! App-level entry (intro plays vs active join) and exit (outro token vs
//! still-inside) outcomes close the loop.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA070_ID: &str = "JA-070";
/// JA-070 sizes.
pub const JA070_SIZES: [Viewport; 1] = [Viewport::new(120, 40)];

/// Why the `Unknown` branch has no adapter leg.
pub const JA070_UNKNOWN_REASON: &str = "DiscoveryError lives in the private arbiter module and cannot be constructed outside the crate; the branch is pinned by the in-crate unit tests";

/// One size of JA-070.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja070Capture {
    /// Decision `Debug` renderings as `(case, decision)` pairs.
    pub decisions: Vec<(String, String)>,
    /// `complete_entry` stamped the blamed instant.
    pub completed_at_ms: Option<i64>,
    /// Entry routes: first-use plays the intro, returning joins.
    pub entry_routes: (String, String),
    /// The exit token played the outro exactly once, then quit.
    pub exit_token_once: bool,
    /// Leaving one of several instances reported still-inside.
    pub exit_still_inside: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn fresh_arbiter() -> DirectSession {
    DirectSession::fresh(
        JA070_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        Viewport::new(120, 40),
        CaptureColor::TrueColor,
    )
}

fn capture_size(viewport: Viewport) -> Ja070Capture {
    let _ = viewport;
    let mut frames = Vec::new();
    let mut decisions = Vec::new();

    let mut play = fresh_arbiter();
    play.app_mut().world.arbiter.set_running(0);
    let first = format!("{:?}", play.app_mut().world.arbiter.request_entry());
    let repeat = format!("{:?}", play.app_mut().world.arbiter.request_entry());
    decisions.push(("entry-play-intro".to_owned(), first));
    decisions.push(("entry-play-repeat".to_owned(), repeat));

    let mut active = fresh_arbiter();
    active.app_mut().world.arbiter.set_running(2);
    decisions.push((
        "entry-join-active".to_owned(),
        format!("{:?}", active.app_mut().world.arbiter.request_entry()),
    ));

    let mut duplicate = fresh_arbiter();
    duplicate.app_mut().world.arbiter.set_running(0);
    duplicate.app_mut().world.arbiter.foreign_claim = true;
    decisions.push((
        "entry-duplicate".to_owned(),
        format!("{:?}", duplicate.app_mut().world.arbiter.request_entry()),
    ));

    let mut released = fresh_arbiter();
    released.app_mut().world.arbiter.set_running(0);
    released.app_mut().world.arbiter.release_entry();
    decisions.push((
        "entry-after-release".to_owned(),
        format!("{:?}", released.app_mut().world.arbiter.request_entry()),
    ));

    let mut completed = fresh_arbiter();
    completed.app_mut().world.arbiter.complete_entry(1_234);
    let completed_at_ms = completed.app().world.arbiter.entered_at_ms;

    let mut still = fresh_arbiter();
    still.app_mut().world.arbiter.set_running(1);
    decisions.push((
        "exit-still-inside".to_owned(),
        format!("{:?}", still.app_mut().world.arbiter.request_exit(9_999)),
    ));

    let mut outro = fresh_arbiter();
    outro.app_mut().world.arbiter.set_running(0);
    outro.app_mut().world.arbiter.entered_at_ms = Some(0);
    decisions.push((
        "exit-outro".to_owned(),
        format!(
            "{:?}",
            outro.app_mut().world.arbiter.request_exit(8_041_000)
        ),
    ));
    decisions.push((
        "exit-already-ended".to_owned(),
        format!(
            "{:?}",
            outro.app_mut().world.arbiter.request_exit(8_042_000)
        ),
    ));
    let first_use = DirectSession::fresh(
        JA070_ID,
        Scenario::FirstUse,
        Motion::Full,
        0,
        Viewport::new(120, 40),
        CaptureColor::TrueColor,
    );
    let entry_first_frame = first_use.observe("entry-first");
    let entry_first = entry_first_frame.route.clone();
    frames.push(entry_first_frame);
    let returning = DirectSession::fresh(
        JA070_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        Viewport::new(120, 40),
        CaptureColor::TrueColor,
    );
    let entry_returning_frame = returning.observe("entry-returning");
    let entry_returning = entry_returning_frame.route.clone();
    frames.push(entry_returning_frame);

    let mut token = DirectSession::fresh(
        JA070_ID,
        Scenario::OutroLast,
        Motion::Full,
        0,
        Viewport::new(120, 40),
        CaptureColor::TrueColor,
    );
    token.ctrl('q');
    token.key(KeyCode::Down);
    token.key(KeyCode::Down);
    token.key(KeyCode::Enter);
    token.key(KeyCode::Enter);
    token.ticks(25);
    let caption = token.observe("exit-caption");
    let captioned =
        caption.route == "outro" && caption.text.contains("You were in the Construct for");
    frames.push(caption);
    token.key(KeyCode::Enter);
    let quit = token.observe("exit-quit");
    let exit_token_once = captioned && quit.quit;
    frames.push(quit);

    let mut busy = DirectSession::fresh(
        JA070_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        Viewport::new(120, 40),
        CaptureColor::TrueColor,
    );
    busy.ctrl('q');
    busy.key(KeyCode::Down);
    busy.key(KeyCode::Down);
    busy.key(KeyCode::Enter);
    let still_frame = busy.observe("exit-still");
    let exit_still_inside = still_frame.route == "manager"
        && still_frame.text.contains("Still inside the Construct")
        && busy.app().world.running_count() == 1;
    frames.push(still_frame);

    Ja070Capture {
        decisions,
        completed_at_ms,
        entry_routes: (entry_first, entry_returning),
        exit_token_once,
        exit_still_inside,
        frames,
    }
}

/// Capture JA-070 at all listed sizes.
#[must_use]
pub fn ja070_arbiter() -> Vec<Ja070Capture> {
    JA070_SIZES.iter().map(|size| capture_size(*size)).collect()
}
