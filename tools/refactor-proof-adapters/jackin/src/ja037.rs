//! JA-037: 1Password error taxonomy and picker walk.
//!
//! The error variants are captured through the production
//! [`SimOnePassword`](jackin_app::sim::onepassword::SimOnePassword) simulator
//! with the pinned fixture identities; the in-app walk covers picker loading,
//! `Esc` unwinding, and cancellation with the parent form retained.

use jackin_app::screens::accounts::OP;
use jackin_app::sim::onepassword::{OpSession, SimOnePassword};
use jackin_app::{ACCOUNT_PICKER, Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, EPOCH_SECS, Viewport};

/// Scenario id.
pub const JA037_ID: &str = "JA-037";
/// JA-037 sizes.
pub const JA037_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One simulated 1Password error observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpErrorCapture {
    /// Variant name (`locked`, `authorization`, `denied`).
    pub name: String,
    /// Operator-facing message from the production simulator.
    pub message: String,
    /// Whether the production simulator marks it retryable.
    pub retryable: bool,
}

/// One size of JA-037.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja037Capture {
    /// Locked, authorization-required, and denied observations.
    pub errors: Vec<OpErrorCapture>,
    /// Account count after the deterministic unlock.
    pub unlocked_accounts: usize,
    /// Whether the 1Password picker control resolves at this size.
    pub op_reachable: bool,
    /// Picker opened from the account form.
    pub picker: ObservedFrame,
    /// Whether the picker layer is open.
    pub picker_open: bool,
    /// One frame per backward `Esc`.
    pub unwind: Vec<ObservedFrame>,
    /// After cancelling the form.
    pub cancelled: ObservedFrame,
    /// Whether the picker layer closed after cancellation.
    pub picker_closed: bool,
}

/// Capture the production error taxonomy (size-independent).
#[must_use]
pub fn ja037_error_taxonomy() -> (Vec<OpErrorCapture>, usize) {
    let mut sim = SimOnePassword::fixture(EPOCH_SECS);
    sim.session = OpSession::Locked;
    let mut errors = Vec::new();
    if let Err(locked) = sim.list_accounts() {
        errors.push(OpErrorCapture {
            name: String::from("locked"),
            message: locked.message(),
            retryable: locked.retryable(),
        });
    }
    sim.session = OpSession::SignedIn;
    let unlocked_accounts = sim.list_accounts().map_or(0, |accounts| accounts.len());
    if let Err(auth) = sim.list_vaults("acme.1password.com") {
        errors.push(OpErrorCapture {
            name: String::from("authorization"),
            message: auth.message(),
            retryable: auth.retryable(),
        });
    }
    if let Err(denied) = sim.list_items("chainargos.1password.com", "v_inf01") {
        errors.push(OpErrorCapture {
            name: String::from("denied"),
            message: denied.message(),
            retryable: denied.retryable(),
        });
    }
    (errors, unlocked_accounts)
}

/// Capture JA-037 at both listed sizes.
#[must_use]
pub fn ja037_op_errors() -> Vec<Ja037Capture> {
    JA037_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja037Capture {
    let (errors, unlocked_accounts) = ja037_error_taxonomy();
    let mut session = DirectSession::fresh(
        JA037_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('a'));
    session.key(KeyCode::Enter);
    session.type_str("Team");
    // The narrow form never paints the picker control; only continue where it
    // resolves, otherwise the contract keys would pinball across routes.
    let op_reachable = session.find("Choose 1Password reference").is_some() && session.tab_to(OP);
    if op_reachable {
        session.key(KeyCode::Enter);
        session.ticks(4);
    }
    let picker = session.observe("picker");
    let picker_open = session.is_open(ACCOUNT_PICKER);
    let mut unwind = Vec::new();
    for index in 0..3 {
        if op_reachable {
            session.key(KeyCode::Esc);
        }
        unwind.push(session.observe(&format!("unwind-{index}")));
    }
    if op_reachable {
        session.key(KeyCode::Esc);
    } else {
        // Close the stalled form so the capture ends on the accounts list.
        session.key(KeyCode::Esc);
    }
    let cancelled = session.observe("cancelled");
    let picker_closed = !session.is_open(ACCOUNT_PICKER);
    Ja037Capture {
        errors,
        unlocked_accounts,
        op_reachable,
        picker,
        picker_open,
        unwind,
        cancelled,
        picker_closed,
    }
}
