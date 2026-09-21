//! JA-034: journey account registration (steps 1–10) plus form grids.
//!
//! Mirrors `complete_jackin_flow_keyboard_first` through step 10, then
//! captures the provider/source selector grid on fresh forms. Source audit:
//! credential inputs resolve only at wider sizes; at 80×24 the narrow form
//! exposes no folder/secret/1Password controls, which the capture records.

use jackin_app::screens::accounts::{FOLDER, FORM, OP, PROVIDER, SECRET, SOURCE};
use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA034_ID: &str = "JA-034";
/// JA-034 sizes.
pub const JA034_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-034.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja034Capture {
    /// Manager with zero instances after the intro.
    pub manager: ObservedFrame,
    /// One frame per saved account (five).
    pub registered: Vec<ObservedFrame>,
    /// Whether the folder input resolves at this size.
    pub folder_reachable: bool,
    /// Whether the 1Password control resolves at this size.
    pub op_reachable: bool,
    /// Whether the secret input resolves at this size.
    pub secret_reachable: bool,
    /// Number of accounts in the registry after step 9.
    pub account_count: usize,
    /// After validating one account and setting a provider default.
    pub validated: ObservedFrame,
    /// Provider selector frames for 0–3 downs.
    pub providers: Vec<ObservedFrame>,
    /// Source selector frames for 0–2 downs.
    pub sources: Vec<ObservedFrame>,
}

/// Capture JA-034 at both listed sizes.
#[must_use]
pub fn ja034_registration_grid() -> Vec<Ja034Capture> {
    JA034_SIZES.into_iter().map(capture_size).collect()
}

fn open_form(session: &mut DirectSession, name: &str) {
    session.key(KeyCode::Char('a'));
    session.key(KeyCode::Enter);
    session.type_str(name);
}

fn save_form(session: &mut DirectSession) {
    let _ = session.tab_to(FORM.sub("save"));
    session.key(KeyCode::Enter);
}

fn try_register_local(
    session: &mut DirectSession,
    name: &str,
    folder: &str,
) -> Option<ObservedFrame> {
    open_form(session, name);
    let _ = session.tab_to(SOURCE);
    session.key(KeyCode::Down);
    if !session.tab_to(FOLDER) {
        return None;
    }
    session.key(KeyCode::Enter);
    session.type_str(folder);
    session.key(KeyCode::Tab);
    save_form(session);
    Some(session.observe(&format!("saved-{name}")))
}

fn try_register_op(
    session: &mut DirectSession,
    name: &str,
    provider_steps: usize,
    item: &str,
) -> Option<ObservedFrame> {
    open_form(session, name);
    let _ = session.tab_to(PROVIDER);
    for _ in 0..provider_steps {
        session.key(KeyCode::Down);
    }
    if session.find("Choose 1Password reference").is_none() || !session.tab_to(OP) {
        return None;
    }
    session.key(KeyCode::Enter);
    session.ticks(4);
    session.key(KeyCode::Enter);
    session.ticks(4);
    session.key(KeyCode::Enter);
    session.ticks(4);
    session.type_str(item);
    session.ticks(4);
    session.key(KeyCode::Enter);
    session.ticks(4);
    session.key(KeyCode::Enter);
    session.ticks(2);
    save_form(session);
    Some(session.observe(&format!("saved-{name}")))
}

fn capture_size(viewport: Viewport) -> Ja034Capture {
    let mut session = DirectSession::fresh(
        JA034_ID,
        Scenario::FirstUse,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.ticks(3);
    session.key(KeyCode::Enter);
    let manager = session.observe("manager");
    session.key(KeyCode::Char('c'));

    let mut registered = Vec::new();
    let mut folder_reachable = true;
    let mut op_reachable = true;
    let mut secret_reachable = true;
    for (name, folder) in [("Personal", "~/.claude"), ("Work", "~/.claude-work")] {
        match try_register_local(&mut session, name, folder) {
            Some(frame) => registered.push(frame),
            None => {
                folder_reachable = false;
                registered.push(session.observe(&format!("saved-{name}-unreachable")));
                session.key(KeyCode::Esc);
            }
        }
    }
    for (name, steps, item) in [("Primary", 1, "Codex Primary"), ("Team", 2, "Grok Team")] {
        match try_register_op(&mut session, name, steps, item) {
            Some(frame) => registered.push(frame),
            None => {
                op_reachable = false;
                registered.push(session.observe(&format!("saved-{name}-unreachable")));
                session.key(KeyCode::Esc);
            }
        }
    }
    open_form(&mut session, "Go");
    let _ = session.tab_to(PROVIDER);
    for _ in 0..3 {
        session.key(KeyCode::Down);
    }
    let _ = session.tab_to(SOURCE);
    session.key(KeyCode::Down);
    session.key(KeyCode::Down);
    if session.tab_to(SECRET) {
        session.key(KeyCode::Enter);
        session.type_str("oc-fixture-only-plain-1234");
        session.key(KeyCode::Tab);
        save_form(&mut session);
        registered.push(session.observe("saved-Go"));
    } else {
        secret_reachable = false;
        registered.push(session.observe("saved-Go-unreachable"));
        session.key(KeyCode::Esc);
    }
    let account_count = session.app().world.accounts.accounts.len();

    let fully_registered = folder_reachable && op_reachable && secret_reachable;
    let validated = if fully_registered {
        session.key(KeyCode::Char('v'));
        session.ticks(20);
        session.key(KeyCode::Home);
        for _ in 0..3 {
            session.key(KeyCode::Down);
        }
        session.key(KeyCode::Char(' '));
        session.observe("validated")
    } else {
        session.observe("validated-unreachable")
    };

    let mut providers = Vec::new();
    for steps in 0..4 {
        let mut form = DirectSession::fresh(
            JA034_ID,
            Scenario::AccountsMixed,
            Motion::Reduced,
            0,
            viewport,
            CaptureColor::TrueColor,
        );
        open_form(&mut form, "Probe");
        let _ = form.tab_to(PROVIDER);
        for _ in 0..steps {
            form.key(KeyCode::Down);
        }
        providers.push(form.observe(&format!("provider-{steps}")));
    }

    let mut sources = Vec::new();
    for steps in 0..3 {
        let mut form = DirectSession::fresh(
            JA034_ID,
            Scenario::AccountsMixed,
            Motion::Reduced,
            0,
            viewport,
            CaptureColor::TrueColor,
        );
        open_form(&mut form, "Probe");
        let _ = form.tab_to(SOURCE);
        for _ in 0..steps {
            form.key(KeyCode::Down);
        }
        sources.push(form.observe(&format!("source-{steps}")));
    }

    Ja034Capture {
        manager,
        registered,
        folder_reachable,
        op_reachable,
        secret_reachable,
        account_count,
        validated,
        providers,
        sources,
    }
}
