//! Manager keyboard-shortcuts card. Sections are the jackin keymap.
//! `HelpOverlay` owns the card, columns, scrollbar, and Done control.
use termrock::{HelpOverlay, HelpOverlayState, HelpSection, Id, Rect, Ui};

pub(super) const ID: Id = Id::root("jackin.manager.help");

const WORKSPACES: &[(&str, &str)] = &[
    ("↑↓ j k", "move"),
    ("←→ h l", "collapse / expand"),
    ("Space", "fold workspace"),
    ("Enter", "launch"),
    ("e", "edit workspace"),
    ("n", "new workspace"),
    ("d", "delete…"),
    ("w", "prewarm"),
    ("o", "open in GitHub"),
    ("* -", "expand / collapse all"),
    ("Tab", "details"),
];

const INSTANCES: &[(&str, &str)] = &[
    ("Enter r", "reconnect / restore"),
    ("a", "new session"),
    ("x", "open shell"),
    ("i", "inspect container"),
    ("t", "stop"),
    ("p", "purge…"),
];

const EVERYWHERE: &[(&str, &str)] = &[("F5", "refresh now")];

const EDITOR: &[(&str, &str)] = &[
    ("←→ 1–5 [ ]", "switch tab"),
    ("Ctrl+S", "save (preview first)"),
    ("Space", "toggle / cycle"),
    ("a e d", "add / edit / remove"),
    ("m p s", "mask · 1Password · scope"),
];

const MOUSE: &[(&str, &str)] = &[
    ("click", "select · 2× activates"),
    ("wheel", "scroll under pointer"),
    ("drag", "seam · scrollbar thumb"),
];

const SHELL: &[(&str, &str)] = &[
    ("s", "Settings"),
    ("c", "Accounts"),
    ("u", "Usage"),
    ("F1 ?", "This reference"),
    ("F10 Ctrl+F2", "Menu bar"),
    ("Ctrl+PgDn", "Next page"),
    ("Ctrl+PgUp", "Previous page"),
    ("q", "Leave this screen"),
    ("Tab", "Next control"),
    ("Shift+Tab", "Previous control"),
    ("Esc", "Back"),
    ("Ctrl+Q", "Quit"),
    ("Ctrl+C", "Copy, else quit"),
];

pub(super) fn draw(ui: &mut Ui<'_>, screen: Rect, state: &HelpOverlayState) {
    let sections = [
        HelpSection::plain("Workspaces", WORKSPACES),
        HelpSection::plain("Instances", INSTANCES),
        HelpSection::plain("Everywhere", EVERYWHERE),
        HelpSection::plain("Editor and Settings", EDITOR),
        HelpSection::plain("Mouse", MOUSE),
        HelpSection::plain("Shell", SHELL),
    ];
    HelpOverlay::new(ID, "Manager", &sections).draw(ui, card_area(screen), state);
}

/// Dim the manager around the card. The card itself stays undimmed so its
/// padding still samples the page ink once.
pub(super) fn dim_underlay(ui: &mut Ui<'_>, screen: Rect) {
    let card = card_area(screen);
    let footer_y = screen.bottom().saturating_sub(1);
    if card.y > screen.y {
        ui.modal_backdrop(Rect {
            height: card.y.saturating_sub(screen.y),
            ..screen
        });
    }
    let below = card.bottom();
    if below < footer_y {
        ui.modal_backdrop(Rect {
            y: below,
            height: footer_y.saturating_sub(below),
            ..screen
        });
    }
    if card.x > screen.x {
        ui.modal_backdrop(Rect {
            y: card.y,
            width: card.x.saturating_sub(screen.x),
            height: card.height,
            ..screen
        });
    }
    let right = card.right();
    if right < screen.right() {
        ui.modal_backdrop(Rect {
            x: right,
            y: card.y,
            width: screen.right().saturating_sub(right),
            height: card.height,
        });
    }
}

/// Same 80% inset `HelpOverlay::measured_size` uses for a modal layer.
fn card_area(screen: Rect) -> Rect {
    let width = screen
        .width
        .saturating_sub(4)
        .min(percent80(screen.width))
        .max(16);
    let height = screen
        .height
        .saturating_sub(3)
        .min(percent80(screen.height))
        .max(8);
    Rect {
        x: screen
            .x
            .saturating_add(screen.width.saturating_sub(width) / 2),
        y: screen
            .y
            .saturating_add(screen.height.saturating_sub(height) / 2),
        width,
        height,
    }
}

fn percent80(n: u16) -> u16 {
    u16::try_from(u32::from(n) * 80 / 100).unwrap_or(u16::MAX)
}
