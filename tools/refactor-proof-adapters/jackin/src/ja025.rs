//! JA-025: unsaved-leave branches and fail-once save retry.
//!
//! Source audit (`app.rs` leave path): leaving a dirty editor sets the
//! `Save changes before leaving?` status and stays in the editor. There is no
//! Stay/Discard/Save choice dialog in the pinned source, and no General key
//! dirties the draft, so the capture records those facts and exercises the
//! production Stay (second `Esc`), Save (preview path), and fail-once retry
//! branches through the supported mounts `r` dirty key.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA025_ID: &str = "JA-025";
/// JA-025 sizes.
pub const JA025_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-025.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja025Capture {
    /// After attempting a General name change.
    pub general_attempt: ObservedFrame,
    /// Whether the General attempt dirtied the draft.
    pub general_dirty: bool,
    /// After dirtying via mounts `r`.
    pub dirty: ObservedFrame,
    /// After `Esc`: leave status, still in the editor.
    pub leave_status: ObservedFrame,
    /// After a second `Esc`: Stay branch.
    pub stay: ObservedFrame,
    /// Whether the Stay branch preserved the dirty draft.
    pub stay_dirty: bool,
    /// Save preview listing the change.
    pub preview: ObservedFrame,
    /// After confirming the save and its deterministic ticks.
    pub saved: ObservedFrame,
    /// After a fail-once save attempt and its ticks.
    pub failed: ObservedFrame,
    /// Whether the failed save kept the draft dirty.
    pub failed_dirty: bool,
    /// After retrying the save to completion.
    pub retried: ObservedFrame,
}

/// Capture JA-025 at both listed sizes.
#[must_use]
pub fn ja025_leave_branches() -> Vec<Ja025Capture> {
    JA025_SIZES.into_iter().map(capture_size).collect()
}

fn open_editor(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA025_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Down);
    session.key(KeyCode::Char('e'));
    session
}

fn dirty_via_mounts(session: &mut DirectSession) {
    session.key(KeyCode::Char(']'));
    session.key(KeyCode::Enter);
    session.key(KeyCode::Char('r'));
}

fn confirm_save(session: &mut DirectSession) {
    session.ctrl('s');
    session.key(KeyCode::Right);
    session.key(KeyCode::Enter);
}

fn capture_size(viewport: Viewport) -> Ja025Capture {
    let mut general = open_editor(viewport);
    general.key(KeyCode::Enter);
    general.type_str("x");
    general.key(KeyCode::Enter);
    let general_attempt = general.observe("general-attempt");
    let general_dirty = general.app().editor.dirty;

    let mut stay_session = open_editor(viewport);
    dirty_via_mounts(&mut stay_session);
    let dirty = stay_session.observe("dirty");
    stay_session.key(KeyCode::Esc);
    let leave_status = stay_session.observe("leave-status");
    stay_session.key(KeyCode::Esc);
    let stay = stay_session.observe("stay");
    let stay_dirty = stay_session.app().editor.dirty;

    let mut save_session = open_editor(viewport);
    dirty_via_mounts(&mut save_session);
    save_session.ctrl('s');
    let preview = save_session.observe("preview");
    save_session.key(KeyCode::Right);
    save_session.key(KeyCode::Enter);
    save_session.ticks(20);
    let saved = save_session.observe("saved");

    let mut fail_session = open_editor(viewport);
    dirty_via_mounts(&mut fail_session);
    fail_session.app_mut().world.save_fails_once = true;
    confirm_save(&mut fail_session);
    fail_session.ticks(20);
    let failed = fail_session.observe("failed");
    let failed_dirty = fail_session.app().editor.dirty;
    confirm_save(&mut fail_session);
    fail_session.ticks(20);
    let retried = fail_session.observe("retried");

    Ja025Capture {
        general_attempt,
        general_dirty,
        dirty,
        leave_status,
        stay,
        stay_dirty,
        preview,
        saved,
        failed,
        failed_dirty,
        retried,
    }
}
