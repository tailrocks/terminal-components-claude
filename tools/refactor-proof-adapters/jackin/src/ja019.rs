//! JA-019: editor workspace mounts add/edit/delete/undo and isolation.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA019_ID: &str = "JA-019";
/// JA-019 sizes.
pub const JA019_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Variant names in contract order.
pub const JA019_VARIANT_NAMES: [&str; 9] = [
    "add",
    "edit",
    "readonly",
    "isolation",
    "iso-1",
    "iso-2",
    "iso-3",
    "open",
    "delete-undo",
];

fn variant_keys() -> Vec<Vec<KeyCode>> {
    vec![
        vec![KeyCode::Char('a')],
        vec![KeyCode::Char('e')],
        vec![KeyCode::Char('r')],
        vec![KeyCode::Char('i')],
        vec![KeyCode::Char('1')],
        vec![KeyCode::Char('2')],
        vec![KeyCode::Char('3')],
        vec![KeyCode::Char('o')],
        vec![KeyCode::Char('d'), KeyCode::Char('u')],
    ]
}

/// One size of JA-019.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja019Capture {
    /// After `Down,e,2,Enter` opens the mounts tab.
    pub open: ObservedFrame,
    /// One frame per contract variant on a fresh mounts tab.
    pub variants: Vec<ObservedFrame>,
    /// Add form opened with `a`, then dismissed with `Esc`.
    pub add_dismissed: ObservedFrame,
    /// `running_isolated` fixture installed, then `i`.
    pub isolated: ObservedFrame,
    /// Fixture flag read back from the pending mount.
    pub isolated_flag: bool,
    /// Debug isolation of the pending mount after `i` under the fixture.
    pub isolated_isolation: String,
}

/// Capture JA-019 at both listed sizes.
#[must_use]
pub fn ja019_mount_actions() -> Vec<Ja019Capture> {
    JA019_SIZES.into_iter().map(capture_size).collect()
}

fn open_mounts(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA019_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    session.key(KeyCode::Char('2'));
    session.key(KeyCode::Enter);
    session
}

fn capture_size(viewport: Viewport) -> Ja019Capture {
    let open = open_mounts(viewport).observe("mounts-open");

    let mut variants = Vec::new();
    for (name, keys) in JA019_VARIANT_NAMES.into_iter().zip(variant_keys()) {
        let mut session = open_mounts(viewport);
        for key in keys {
            session.key(key);
        }
        variants.push(session.observe(&format!("variant-{name}")));
    }

    let mut add = open_mounts(viewport);
    add.key(KeyCode::Char('a'));
    let _ = add.observe("add-open");
    add.key(KeyCode::Esc);
    let add_dismissed = add.observe("add-dismissed");

    let mut isolated_session = open_mounts(viewport);
    if let Some(mount) = isolated_session.app_mut().editor.pending.mounts.first_mut() {
        mount.running_isolated = true;
    }
    isolated_session.draw();
    isolated_session.key(KeyCode::Char('i'));
    let isolated = isolated_session.observe("isolated-i");
    let (isolated_flag, isolated_isolation) = isolated_session
        .app()
        .editor
        .pending
        .mounts
        .first()
        .map_or((false, String::from("no-mount")), |mount| {
            (mount.running_isolated, format!("{:?}", mount.isolation))
        });

    Ja019Capture {
        open,
        variants,
        add_dismissed,
        isolated,
        isolated_flag,
        isolated_isolation,
    }
}
