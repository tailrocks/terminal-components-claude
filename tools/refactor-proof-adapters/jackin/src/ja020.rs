//! JA-020: editor roles enable/default, search, load picker, dirty count.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA020_ID: &str = "JA-020";
/// JA-020 sizes.
pub const JA020_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-020.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja020Capture {
    /// After `Down,e,3,Enter` opens the roles tab.
    pub open: ObservedFrame,
    /// After `Space,Enter` toggles enable/default.
    pub toggled: ObservedFrame,
    /// After `/`, typing `architect`, and `Enter`.
    pub searched: ObservedFrame,
    /// After `a` opens the role load picker.
    pub picker: ObservedFrame,
    /// After selecting a role fixture row and `Enter`.
    pub loaded: ObservedFrame,
    /// After `Esc` leaves the roles body.
    pub escaped: ObservedFrame,
}

/// Capture JA-020 at both listed sizes.
#[must_use]
pub fn ja020_role_actions() -> Vec<Ja020Capture> {
    JA020_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja020Capture {
    let mut session = DirectSession::fresh(
        JA020_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    session.key(KeyCode::Char('3'));
    session.key(KeyCode::Enter);
    let open = session.observe("roles-open");
    session.key(KeyCode::Char(' '));
    session.key(KeyCode::Enter);
    let toggled = session.observe("toggled");
    session.key(KeyCode::Char('/'));
    session.type_str("architect");
    session.key(KeyCode::Enter);
    let searched = session.observe("searched");
    session.key(KeyCode::Char('a'));
    let picker = session.observe("picker");
    session.key(KeyCode::Home);
    session.key(KeyCode::Down);
    session.key(KeyCode::Enter);
    let loaded = session.observe("loaded");
    session.key(KeyCode::Esc);
    let escaped = session.observe("escaped");
    Ja020Capture {
        open,
        toggled,
        searched,
        picker,
        loaded,
        escaped,
    }
}
