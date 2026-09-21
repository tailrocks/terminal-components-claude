//! JA-035: account row action variants on the `Work` account.
//!
//! Source audit: removal is unconfirmable in the pinned source — `x` stages
//! the `Remove account …?` question and no key consumes the confirmation, so
//! both the cancel and the confirm variants retain the account.

use jackin_app::screens::accounts::LIST;
use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA035_ID: &str = "JA-035";
/// JA-035 sizes.
pub const JA035_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Fixture id of the Claude `Work` account.
pub const JA035_WORK_ID: &str = "acct-claude-work";

/// Variant names in contract order.
pub const JA035_VARIANT_NAMES: [&str; 9] = [
    "edit",
    "disable",
    "default",
    "validate",
    "refresh",
    "mask",
    "remove-cancel",
    "remove-confirm",
    "f5",
];

fn variant_keys() -> Vec<Vec<KeyCode>> {
    vec![
        vec![KeyCode::Char('e')],
        vec![KeyCode::Char('d')],
        vec![KeyCode::Char(' ')],
        vec![KeyCode::Char('v')],
        vec![KeyCode::Char('r')],
        vec![KeyCode::Char('m')],
        vec![KeyCode::Char('x'), KeyCode::Esc],
        vec![KeyCode::Char('x'), KeyCode::Right, KeyCode::Enter],
        vec![KeyCode::F(5)],
    ]
}

/// One size of JA-035.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja035Capture {
    /// One frame per variant after its keys and `T(60)`.
    pub variants: Vec<ObservedFrame>,
    /// Whether `Work` survived the remove-cancel variant.
    pub cancel_retained: bool,
    /// Whether `Work` survived the remove-confirm variant (no key consumes
    /// the confirmation in the pinned source, so this stays `true`).
    pub confirm_retained: bool,
}

/// Capture JA-035 at both listed sizes.
#[must_use]
pub fn ja035_account_actions() -> Vec<Ja035Capture> {
    JA035_SIZES.into_iter().map(capture_size).collect()
}

fn select_work(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA035_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    // Fresh accounts starts host-focused; the list owns rows after `Tab`.
    // `Work` is cursor index 4 (`Home`, four `Down`s).
    let _ = session.tab_to(LIST);
    session.key(KeyCode::Home);
    for _ in 0..4 {
        session.key(KeyCode::Down);
    }
    session
}

fn capture_size(viewport: Viewport) -> Ja035Capture {
    let mut variants = Vec::new();
    for (name, keys) in JA035_VARIANT_NAMES.into_iter().zip(variant_keys()) {
        let mut session = select_work(viewport);
        for key in keys {
            session.key(key);
        }
        session.ticks(60);
        variants.push(session.observe(&format!("variant-{name}")));
    }
    let cancel_retained = {
        let mut session = select_work(viewport);
        session.key(KeyCode::Char('x'));
        session.key(KeyCode::Esc);
        session.app().world.accounts.get(JA035_WORK_ID).is_some()
    };
    let confirm_retained = {
        let mut session = select_work(viewport);
        session.key(KeyCode::Char('x'));
        session.key(KeyCode::Right);
        session.key(KeyCode::Enter);
        session.app().world.accounts.get(JA035_WORK_ID).is_some()
    };
    Ja035Capture {
        variants,
        cancel_retained,
        confirm_retained,
    }
}
