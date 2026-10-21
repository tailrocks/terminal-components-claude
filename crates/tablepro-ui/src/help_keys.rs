use termrock::{Cx, HelpOverlay, HelpOverlayState, HelpSection, Id, LayerSpec, Ui};

pub(crate) const ID: Id = Id::root("tablepro.help");

const SHELL: &[(&str, &str)] = &[
    ("] Ctrl+PgDn", "Next page"),
    ("[ Ctrl+PgUp", "Previous page"),
    ("Esc", "Cancel the running query"),
    ("Ctrl+R", "Run the statement at the cursor"),
    ("F5", "Run, or refresh the table"),
    ("Alt+R Shift+F5", "Run all statements"),
    ("Ctrl+X", "EXPLAIN"),
    ("Alt+X Shift+F7", "EXPLAIN ANALYZE"),
    ("Ctrl+T", "New query tab"),
    ("Ctrl+W", "Close tab"),
    ("Ctrl+O Ctrl+P", "Open Quickly"),
    ("Ctrl+G", "Tab list"),
    ("Ctrl+Y", "Query history"),
    ("Ctrl+B", "Show or hide the explorer"),
    ("Ctrl+L", "Safe Mode"),
    ("Ctrl+D", "Data / Structure"),
    ("Ctrl+S", "Save pending changes"),
    ("Ctrl+F", "Filter or find"),
    ("z", "Maximize the pane"),
    ("Ctrl+↑", "Grow the results"),
    ("Ctrl+↓", "Grow the editor"),
    ("Ctrl+N", "New connection"),
    ("F1 ?", "This reference"),
    ("F10 Ctrl+F2", "Menu bar"),
    ("0", "Focus the explorer"),
    ("Tab", "Next control"),
    ("Shift+Tab", "Previous control"),
    ("Esc", "Back"),
    ("Ctrl+Q q", "Quit"),
    ("Ctrl+C", "Copy, else quit"),
];

const GRID: &[(&str, &str)] = &[
    ("Enter F2", "Edit a cell · Esc cancels"),
    ("Ctrl+F", "Filter (grid) or find (editor)"),
    ("p", "Preview the SQL of pending changes"),
    ("z", "Zoom the focused pane"),
    ("Shift+F10", "Menu of the focused row, cell or tab"),
    ("Right-click", "The same menu at the pointer"),
];

pub(crate) fn layer(cx: &Cx<'_>) -> LayerSpec {
    let shell = HelpSection::plain("Shell and workbench", SHELL);
    let grid = HelpSection::plain("Grid and editor", GRID);
    let sections = [shell, grid];
    HelpOverlay::new(ID, "TablePro", &sections).layer(cx)
}

pub(crate) fn update(
    cx: &mut Cx<'_>,
    state: &mut HelpOverlayState,
) -> termrock::Response<termrock::HelpAction> {
    let shell = HelpSection::plain("Shell and workbench", SHELL);
    let grid = HelpSection::plain("Grid and editor", GRID);
    let sections = [shell, grid];
    HelpOverlay::new(ID, "TablePro", &sections).update(cx, state)
}

pub(crate) fn draw(ui: &mut Ui<'_>, area: termrock::Rect, state: &HelpOverlayState) {
    let shell = HelpSection::plain("Shell and workbench", SHELL);
    let grid = HelpSection::plain("Grid and editor", GRID);
    let sections = [shell, grid];
    HelpOverlay::new(ID, "TablePro", &sections).draw(ui, area, state);
}
