//! JA-022: env reference/scope/delete/undo, key validation, role folding.

use jackin_app::{Motion, Scenario};
use junie_tui::{Id, KeyCode};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA022_ID: &str = "JA-022";
/// JA-022 sizes.
pub const JA022_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Variant names in contract order.
pub const JA022_VARIANT_NAMES: [&str; 5] = [
    "edit",
    "reference",
    "scope",
    "delete-cancel",
    "delete-confirm-undo",
];

fn variant_keys() -> Vec<Vec<KeyCode>> {
    vec![
        vec![KeyCode::Char('e')],
        vec![KeyCode::Char('p')],
        vec![KeyCode::Char('s')],
        vec![KeyCode::Char('d'), KeyCode::Esc],
        vec![
            KeyCode::Char('d'),
            KeyCode::Right,
            KeyCode::Enter,
            KeyCode::Char('u'),
        ],
    ]
}

/// One size of JA-022.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja022Capture {
    /// After `Down,e,4,Enter` opens environments.
    pub open: ObservedFrame,
    /// One frame per contract variant on fresh environments.
    pub variants: Vec<ObservedFrame>,
    /// Invalid key submitted through the add form.
    pub invalid: ObservedFrame,
    /// Duplicate key submitted through the add form.
    pub duplicate: ObservedFrame,
    /// Role section after `Left`, `Right`, `Space` folds.
    pub folds: Vec<ObservedFrame>,
}

/// Capture JA-022 at both listed sizes.
#[must_use]
pub fn ja022_env_actions() -> Vec<Ja022Capture> {
    JA022_SIZES.into_iter().map(capture_size).collect()
}

fn cfg_save() -> Id {
    Id::root("editor.cfg").sub("form").sub("save")
}

fn open_env(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA022_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    session.key(KeyCode::Char('4'));
    session.key(KeyCode::Enter);
    session
}

fn submit_key(session: &mut DirectSession, key: &str) {
    session.key(KeyCode::Char('a'));
    session.key(KeyCode::Enter);
    session.type_str(key);
    let _ = session.tab_to(cfg_save());
    session.key(KeyCode::Enter);
}

fn capture_size(viewport: Viewport) -> Ja022Capture {
    let open = open_env(viewport).observe("env-open");

    let mut variants = Vec::new();
    for (name, keys) in JA022_VARIANT_NAMES.into_iter().zip(variant_keys()) {
        let mut session = open_env(viewport);
        for key in keys {
            session.key(key);
        }
        variants.push(session.observe(&format!("variant-{name}")));
    }

    let mut invalid_session = open_env(viewport);
    submit_key(&mut invalid_session, "BAD-NAME");
    let invalid = invalid_session.observe("invalid-key");

    let mut duplicate_session = open_env(viewport);
    submit_key(&mut duplicate_session, "DATABASE_URL");
    let duplicate = duplicate_session.observe("duplicate-key");

    let mut folds = Vec::new();
    for (name, key) in [
        ("fold-left", KeyCode::Left),
        ("fold-right", KeyCode::Right),
        ("fold-space", KeyCode::Char(' ')),
    ] {
        let mut session = open_env(viewport);
        session.key(key);
        folds.push(session.observe(name));
    }

    Ja022Capture {
        open,
        variants,
        invalid,
        duplicate,
        folds,
    }
}
