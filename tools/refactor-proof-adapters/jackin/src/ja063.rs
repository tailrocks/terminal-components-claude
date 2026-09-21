//! JA-063: help overlays.
//!
//! `?` on the manager opens the keyboard-shortcuts overlay and Esc closes
//! it (focus settles into the tree). On the capsule the focused pane
//! eats `?` as typed input, so capsule help opens with `?` only once the
//! menubar owns focus (`F10,?`); Esc closes it again. The palette's and
//! the Help menu's Keyboard-shortcuts items are inert in the pinned
//! source: running them opens no overlay. On the intro `?` is ignored
//! and the frame stays bit-identical.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA063_ID: &str = "JA-063";
/// JA-063 sizes.
pub const JA063_SIZES: [Viewport; 1] = [Viewport::new(120, 40)];

/// One size of JA-063.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja063Capture {
    /// The manager overlay showed its title and close hint.
    pub manager_overlay: bool,
    /// Esc closed the manager overlay while the manager kept the route.
    pub manager_restored: bool,
    /// Capsule `?` was typed into the pane instead of opening help.
    pub capsule_question_typed: bool,
    /// `F10,?` opened capsule help.
    pub capsule_overlay: bool,
    /// Esc closed capsule help.
    pub capsule_restored: bool,
    /// The palette's Keyboard-shortcuts item opened no overlay.
    pub palette_shortcuts_inert: bool,
    /// Intro `?` left the frame bit-identical.
    pub intro_ignored: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn capture_size(viewport: Viewport) -> Ja063Capture {
    let mut frames = Vec::new();

    let mut manager = DirectSession::fresh(
        JA063_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    frames.push(manager.observe("before-help"));
    manager.key(KeyCode::Char('?'));
    let overlay = manager.observe("manager-help");
    let manager_overlay = overlay.route == "manager"
        && overlay.text.contains("Keyboard shortcuts")
        && overlay.text.contains("Esc Close");
    frames.push(overlay);
    manager.key(KeyCode::Esc);
    let restored = manager.observe("manager-restored");
    // The overlay closes but focus settles into the tree, so the frame
    // differs from the pre-open one by the focus ring alone.
    let manager_restored = restored.route == "manager"
        && !restored.text.contains("Keyboard shortcuts")
        && !restored.text.contains("Esc Close");
    frames.push(restored);

    let mut capsule = DirectSession::fresh(
        JA063_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let capsule_before = capsule.observe("capsule-before").digest;
    capsule.key(KeyCode::Char('?'));
    let typed = capsule.observe("capsule-question");
    let capsule_question_typed = typed.digest != capsule_before
        && typed
            .text
            .lines()
            .any(|line| line.contains('?') && (line.contains('❯') || line.contains('▎')));
    frames.push(typed);

    capsule.key(KeyCode::F(10));
    capsule.key(KeyCode::Char('?'));
    let capsule_help = capsule.observe("capsule-help");
    let capsule_overlay = capsule_help.route == "capsule"
        && capsule_help.text.contains("Previous tab")
        && capsule_help.text.contains("Navigation");
    frames.push(capsule_help);
    capsule.key(KeyCode::Esc);
    let capsule_closed = capsule.observe("capsule-closed");
    let capsule_restored =
        !capsule_closed.text.contains("Previous tab") && capsule_closed.route == "capsule";
    frames.push(capsule_closed);

    let mut inert = DirectSession::fresh(
        JA063_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    inert.ctrl('\\');
    inert.type_str("shortcuts");
    inert.key(KeyCode::Enter);
    let inert_frame = inert.observe("shortcuts-inert");
    let palette_shortcuts_inert = inert_frame.route == "capsule"
        && !inert_frame.text.contains("Previous tab")
        && !inert_frame.text.contains("Navigation");
    frames.push(inert_frame);

    let mut intro = DirectSession::fresh(
        JA063_ID,
        Scenario::FirstUse,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let intro_before = intro.observe("intro-before").digest;
    intro.key(KeyCode::Char('?'));
    let intro_after = intro.observe("intro-after");
    let intro_ignored = intro_after.digest == intro_before;
    frames.push(intro_after);

    Ja063Capture {
        manager_overlay,
        manager_restored,
        capsule_question_typed,
        capsule_overlay,
        capsule_restored,
        palette_shortcuts_inert,
        intro_ignored,
        frames,
    }
}

/// Capture JA-063 at all listed sizes.
#[must_use]
pub fn ja063_help_overlays() -> Vec<Ja063Capture> {
    JA063_SIZES.iter().map(|size| capture_size(*size)).collect()
}
