//! JA-032: replay the 1Password registration journey with checkpoints.
//!
//! Source audit: the 1Password picker button is painted and reachable only at
//! wider sizes. At 80×24 the narrow account form never paints the `Choose
//! 1Password reference…` control and excludes it from the focus ring, so the
//! picker flow cannot start there. The capture runs the full journey where the
//! control resolves and records the narrow gap as data.

use jackin_app::screens::accounts::{FORM, OP};
use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA032_ID: &str = "JA-032";
/// JA-032 sizes.
pub const JA032_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-032.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja032Capture {
    /// New-account form.
    pub form: ObservedFrame,
    /// Whether the 1Password picker control resolves at this size.
    pub op_reachable: bool,
    /// 1Password account list (`chainargos`).
    pub op_accounts: ObservedFrame,
    /// Vault list (`Engineering`).
    pub vaults: ObservedFrame,
    /// Field list (`credential`).
    pub fields: ObservedFrame,
    /// Selected reference (`Anthropic · Work › credential`).
    pub reference: ObservedFrame,
    /// Duplicate-source refusal.
    pub duplicate: ObservedFrame,
    /// Whether the duplicate created no account.
    pub duplicate_absent: bool,
    /// Codex/Throttled save with the rate-limit status.
    pub codex_saved: ObservedFrame,
    /// After refreshing the throttled account.
    pub refreshed: ObservedFrame,
}

/// Replay JA-032 at both listed sizes.
#[must_use]
pub fn ja032_one_password_replay() -> Vec<Ja032Capture> {
    JA032_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja032Capture {
    let mut session = DirectSession::fresh(
        JA032_ID,
        Scenario::AccountsMixed,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('a'));
    session.key(KeyCode::Enter);
    session.type_str("Team");
    let form = session.observe("form");
    let op_reachable = session.find("Choose 1Password reference").is_some() && session.tab_to(OP);
    if !op_reachable {
        let stall = |session: &DirectSession, name: &str| session.observe(name);
        return Ja032Capture {
            op_accounts: stall(&session, "op-accounts-unreachable"),
            vaults: stall(&session, "vaults-unreachable"),
            fields: stall(&session, "fields-unreachable"),
            reference: stall(&session, "reference-unreachable"),
            duplicate: stall(&session, "duplicate-unreachable"),
            duplicate_absent: true,
            codex_saved: stall(&session, "codex-saved-unreachable"),
            refreshed: stall(&session, "refreshed-unreachable"),
            form,
            op_reachable,
        };
    }
    session.key(KeyCode::Enter);
    session.ticks(4);
    let op_accounts = session.observe("op-accounts");
    session.key(KeyCode::Enter);
    session.ticks(4);
    let vaults = session.observe("vaults");
    session.key(KeyCode::Enter);
    session.ticks(4);
    session.type_str("Anthropic");
    session.ticks(4);
    session.key(KeyCode::Enter);
    session.ticks(4);
    let fields = session.observe("fields");
    session.key(KeyCode::Enter);
    session.ticks(2);
    let reference = session.observe("reference");
    let _ = session.tab_to(FORM.sub("save"));
    session.key(KeyCode::Enter);
    let duplicate = session.observe("duplicate");
    let duplicate_absent = session
        .app()
        .world
        .accounts
        .get("acct-anthropic-team")
        .is_none();
    let _ = session.tab_to(FORM.sub("provider"));
    session.key(KeyCode::Down);
    let _ = session.tab_to(OP);
    session.key(KeyCode::Enter);
    session.ticks(4);
    session.key(KeyCode::Enter);
    session.ticks(4);
    session.key(KeyCode::Enter);
    session.ticks(4);
    session.type_str("Throttled");
    session.ticks(4);
    session.key(KeyCode::Enter);
    session.ticks(4);
    session.key(KeyCode::Enter);
    session.ticks(2);
    let _ = session.tab_to(FORM.sub("save"));
    session.key(KeyCode::Enter);
    let codex_saved = session.observe("codex-saved");
    session.key(KeyCode::Char('r'));
    session.ticks(60);
    let refreshed = session.observe("refreshed");
    Ja032Capture {
        form,
        op_reachable,
        op_accounts,
        vaults,
        fields,
        reference,
        duplicate,
        duplicate_absent,
        codex_saved,
        refreshed,
    }
}
