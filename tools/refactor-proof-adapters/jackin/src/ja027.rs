//! JA-027: attempted settings global mount/env actions (ground truth).
//!
//! Source audit: the pinned settings screen has no mount/env tabs, so the
//! contract's `2`/`3` tab entries are inert, `Enter` leaves settings for the
//! manager, and the remaining keys fall through to global navigation. The
//! capture drives the contract sequence verbatim and records the actual
//! routes, then covers the production trust-toggle → preview → cancel path
//! that `Ctrl-S` owns.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA027_ID: &str = "JA-027";
/// JA-027 sizes.
pub const JA027_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Contract keys attempted after `s,2,Enter`, in order.
pub const JA027_MOUNT_KEYS: [KeyCode; 6] = [
    KeyCode::Char('s'),
    KeyCode::Char('a'),
    KeyCode::Char('e'),
    KeyCode::Char('d'),
    KeyCode::Char('u'),
    KeyCode::Esc,
];

/// Contract keys attempted after `Esc,3,Enter`, in order.
pub const JA027_ENV_KEYS: [KeyCode; 6] = [
    KeyCode::Char('a'),
    KeyCode::Char('e'),
    KeyCode::Char('s'),
    KeyCode::Char('d'),
    KeyCode::Char('u'),
    KeyCode::Esc,
];

/// One size of JA-027.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja027Capture {
    /// After `s,2,Enter`.
    pub mount_attempt: ObservedFrame,
    /// One frame per attempted mount key.
    pub mount_keys: Vec<ObservedFrame>,
    /// After `Esc,3,Enter` on a fresh settings page.
    pub env_attempt: ObservedFrame,
    /// One frame per attempted env key.
    pub env_keys: Vec<ObservedFrame>,
    /// Trust toggled, then `Ctrl-S` preview.
    pub preview: ObservedFrame,
    /// After cancelling the preview.
    pub cancelled: ObservedFrame,
    /// Whether the cancelled draft is still dirty.
    pub cancelled_dirty: bool,
}

/// Capture JA-027 at both listed sizes.
#[must_use]
pub fn ja027_settings_global_attempts() -> Vec<Ja027Capture> {
    JA027_SIZES.into_iter().map(capture_size).collect()
}

fn open_settings(viewport: Viewport) -> DirectSession {
    let mut session = DirectSession::fresh(
        JA027_ID,
        Scenario::Returning,
        Motion::Reduced,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Char('s'));
    session
}

fn capture_size(viewport: Viewport) -> Ja027Capture {
    let mut mount_session = open_settings(viewport);
    mount_session.key(KeyCode::Char('2'));
    mount_session.key(KeyCode::Enter);
    let mount_attempt = mount_session.observe("mount-attempt");
    let mut mount_keys = Vec::new();
    for (index, key) in JA027_MOUNT_KEYS.into_iter().enumerate() {
        mount_session.key(key);
        mount_keys.push(mount_session.observe(&format!("mount-key-{index}")));
    }

    let mut env_session = open_settings(viewport);
    env_session.key(KeyCode::Esc);
    env_session.key(KeyCode::Char('3'));
    env_session.key(KeyCode::Enter);
    let env_attempt = env_session.observe("env-attempt");
    let mut env_keys = Vec::new();
    for (index, key) in JA027_ENV_KEYS.into_iter().enumerate() {
        env_session.key(key);
        env_keys.push(env_session.observe(&format!("env-key-{index}")));
    }

    let mut preview_session = open_settings(viewport);
    preview_session.key(KeyCode::Char(' '));
    preview_session.ctrl('s');
    let preview = preview_session.observe("preview");
    preview_session.key(KeyCode::Esc);
    let cancelled = preview_session.observe("cancelled");
    let cancelled_dirty = preview_session.app().settings.dirty;

    Ja027Capture {
        mount_attempt,
        mount_keys,
        env_attempt,
        env_keys,
        preview,
        cancelled,
        cancelled_dirty,
    }
}
