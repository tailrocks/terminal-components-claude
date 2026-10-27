use termrock::{HelpOverlay, HelpOverlayState, HelpSection, Id, Part, Role, StylePatch, Ui};

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

const SECTIONS: [HelpSection<'static>; 6] = [
    HelpSection::plain("Workspaces", WORKSPACES),
    HelpSection::plain("Instances", INSTANCES),
    HelpSection::plain("Everywhere", EVERYWHERE),
    HelpSection::plain("Editor and Settings", EDITOR),
    HelpSection::plain("Mouse", MOUSE),
    HelpSection::plain("Shell", SHELL),
];

const BORDER_PATCH: &[(Part, StylePatch)] =
    &[(Part::BORDER, StylePatch::new().set_fg(Role::BorderStrong))];

pub(super) fn overlay() -> HelpOverlay<'static> {
    HelpOverlay::new(ID, "Manager", &SECTIONS).patch_part(BORDER_PATCH)
}

pub(super) fn draw(ui: &mut Ui<'_>, area: termrock::Rect, state: &HelpOverlayState) {
    overlay().draw(ui, area, state);
}
