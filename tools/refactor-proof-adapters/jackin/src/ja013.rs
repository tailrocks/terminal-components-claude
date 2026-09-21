//! JA-013: prelude git URL, custom destination, name, and Esc rewind.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA013_ID: &str = "JA-013";
/// JA-013 sizes.
pub const JA013_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-013.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja013Capture {
    /// After Char('n').
    pub after_n: ObservedFrame,
    /// After opening prelude.
    pub prelude: ObservedFrame,
    /// After Char('g').
    pub after_g: ObservedFrame,
    /// After typing the git URL.
    pub after_git_url: ObservedFrame,
    /// After choosing source.
    pub after_source: ObservedFrame,
    /// After typing a custom destination.
    pub after_destination: ObservedFrame,
    /// After continuing through Edit/Workdir.
    pub after_continue: ObservedFrame,
    /// After typing Demo.
    pub after_name: ObservedFrame,
    /// Frames from Esc rewind until Manager or bound.
    pub rewind: Vec<ObservedFrame>,
}

/// Capture JA-013 at both listed sizes.
#[must_use]
pub fn ja013_prelude_git_and_rewind() -> Vec<Ja013Capture> {
    JA013_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja013Capture {
    let mut session = DirectSession::fresh(
        JA013_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('n'));
    let after_n = session.observe("key-n");
    session.key(KeyCode::End);
    session.key(KeyCode::Enter);
    let prelude = session.observe("prelude");
    session.key(KeyCode::Char('g'));
    let after_g = session.observe("git");
    session.type_str("https://github.com/example/demo.git");
    let after_git_url = session.observe("git-url");
    session.key(KeyCode::Char(' '));
    let after_source = session.observe("choose-source");
    session.type_str("/work/demo");
    let after_destination = session.observe("destination");
    session.key(KeyCode::Enter);
    session.key(KeyCode::Enter);
    let after_continue = session.observe("continue");
    session.type_str("Demo");
    let after_name = session.observe("name-demo");
    let mut rewind = Vec::new();
    for index in 0..8 {
        if session.app().route() != jackin_app::Route::Prelude {
            break;
        }
        session.key(KeyCode::Esc);
        rewind.push(session.observe(&format!("rewind-{index}")));
    }
    Ja013Capture {
        after_n,
        prelude,
        after_g,
        after_git_url,
        after_source,
        after_destination,
        after_continue,
        after_name,
        rewind,
    }
}
