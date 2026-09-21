//! JA-064: overlay dismissal rules.
//!
//! The command palette takes focus and eats typed keys as filter input
//! (the pane never echoes them), but Esc does not dismiss it: only
//! running a command with Enter closes it. The tab menu is the opposite:
//! Esc dismisses it while the route and focus stay in the capsule, and a
//! bare Esc with no overlay open is a no-op.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA064_ID: &str = "JA-064";
/// JA-064 sizes.
pub const JA064_SIZES: [Viewport; 1] = [Viewport::new(120, 40)];

/// Focus owner while the palette is open.
pub const JA064_PALETTE_FOCUS: &str = "capsule-command-palette";

/// One size of JA-064.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja064Capture {
    /// Focus owner with the palette open.
    pub palette_focus: Option<String>,
    /// Typed keys filtered the palette instead of reaching the pane.
    pub pane_blocked: bool,
    /// Esc left the palette open and focused.
    pub palette_esc_inert: bool,
    /// Enter ran the filtered command and left the capsule.
    pub palette_enter_route: String,
    /// Focus left the palette once the command ran.
    pub palette_enter_closed: bool,
    /// Esc dismissed the tab menu while the capsule kept route and focus.
    pub menu_esc: bool,
    /// A palette-level Esc returned focus to the intact menu below.
    pub menu_survives_palette_esc: bool,
    /// A bare Esc kept the capsule route.
    pub bare_esc_noop: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn palette_open(focus: &Option<String>) -> bool {
    focus
        .as_deref()
        .is_some_and(|id| id.contains(JA064_PALETTE_FOCUS))
}

fn pane_echoed(frame: &ObservedFrame, needle: &str) -> bool {
    frame.text.lines().any(|line| {
        let trimmed = line.trim_start();
        (trimmed.starts_with('❯') || trimmed.starts_with('▎')) && line.contains(needle)
    })
}

fn capture_size(viewport: Viewport) -> Ja064Capture {
    let mut frames = Vec::new();

    let mut palette = DirectSession::fresh(
        JA064_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    palette.ctrl('\\');
    let open = palette.observe("palette-open");
    let palette_focus = open.focus.clone();
    frames.push(open);
    palette.type_str("usage");
    let filtered = palette.observe("palette-filtered");
    let pane_blocked = palette_open(&filtered.focus) && !pane_echoed(&filtered, "usage");
    frames.push(filtered);
    palette.key(KeyCode::Esc);
    let esc = palette.observe("palette-esc");
    let palette_esc_inert = palette_open(&esc.focus);
    frames.push(esc);
    palette.type_str("usage");
    palette.key(KeyCode::Enter);
    let ran = palette.observe("palette-ran");
    let palette_enter_route = ran.route.clone();
    let palette_closed = !palette_open(&ran.focus);
    frames.push(ran);

    let mut menu = DirectSession::fresh(
        JA064_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    menu.ctrl('b');
    menu.key(KeyCode::Char('m'));
    let menu_open = menu.observe("menu-open");
    let menu_shown = menu_open.text.contains("Close tab");
    frames.push(menu_open);
    menu.key(KeyCode::Esc);
    let menu_closed = menu.observe("menu-closed");
    let menu_esc =
        menu_shown && !menu_closed.text.contains("Close tab") && menu_closed.route == "capsule";
    frames.push(menu_closed);

    menu.ctrl('b');
    menu.key(KeyCode::Char('m'));
    menu.ctrl('\\');
    menu.key(KeyCode::Esc);
    let stacked = menu.observe("stacked-esc");
    // Esc pops focus back to the menu, which keeps its content; the
    // palette paint lingers underneath until a command runs.
    let menu_survives_palette_esc = stacked
        .focus
        .as_deref()
        .is_some_and(|focus| focus.contains("capsule-tab-menu"))
        && stacked.text.contains("Close tab")
        && stacked.route == "capsule";
    frames.push(stacked);

    let mut bare = DirectSession::fresh(
        JA064_ID,
        Scenario::CapsuleMulti,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    bare.key(KeyCode::Esc);
    let bare_frame = bare.observe("bare-esc");
    let bare_esc_noop = bare_frame.route == "capsule";
    frames.push(bare_frame);

    Ja064Capture {
        palette_focus,
        pane_blocked,
        palette_esc_inert,
        palette_enter_route,
        palette_enter_closed: palette_closed,
        menu_esc,
        menu_survives_palette_esc,
        bare_esc_noop,
        frames,
    }
}

/// Capture JA-064 at all listed sizes.
#[must_use]
pub fn ja064_overlay_dismissal() -> Vec<Ja064Capture> {
    JA064_SIZES.iter().map(|size| capture_size(*size)).collect()
}
