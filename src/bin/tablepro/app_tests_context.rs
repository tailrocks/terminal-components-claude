//! Secondary clicks and context menus through the real App.

use junie_tui::core::event::{Input, Key, MouseKind};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::context::{ContextConfig, ContextKeys};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use crate::app::Modal;
use crate::app_tests::H;
use crate::context::{CtxMenu, Target};
use crate::workbench::WorkTab;

/// Open `orders` from the explorer (its grid focused).
fn open_orders(h: &mut H) {
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
}

fn filter_column_open(h: &H) -> Option<bool> {
    match h.app.modal() {
        Some(Modal::Filter(f)) => Some(f.column.open),
        _ => None,
    }
}

#[test]
fn a_right_click_outside_the_filter_editors_open_select_closes_it() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    h.key(KeyCode::Char('f'));
    // the column select, opened from the keyboard
    let column = WidgetId::of("filter-editor").sub("col");
    h.tab_to(column);
    h.key(KeyCode::Enter);
    assert_eq!(filter_column_open(&h), Some(true), "the select is open");
    // a right-click on the dimmed page, outside the popup
    h.mouse(MouseKind::Secondary, 3, 30);
    assert_eq!(
        filter_column_open(&h),
        Some(false),
        "the right-click outside closes the select, like a click"
    );
    assert!(
        matches!(h.app.modal(), Some(Modal::Filter(_))),
        "the form itself stays: only its Esc or Cancel discards it"
    );
}

fn menu(h: &H) -> &CtxMenu {
    match h.app.modal() {
        Some(Modal::Menu(m)) => m,
        _ => panic!("no context menu open"),
    }
}

fn table(h: &H) -> &crate::tabs::TableTab {
    match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => t,
        _ => panic!("expected a table tab"),
    }
}

/// Where hit `id` was drawn this frame.
fn area(h: &H, id: WidgetId) -> Rect {
    h.app.hits.area_of(id).expect("drawn this frame")
}

/// Click the open menu's item labelled `label`.
fn choose(h: &mut H, label: &str) {
    let m = menu(h);
    let i = m
        .menu
        .items
        .iter()
        .position(|it| it.label == label)
        .unwrap_or_else(|| panic!("no item {label}"));
    let r = area(h, m.menu.row_id(i));
    h.click(r.x + 2, r.y);
}

fn shift_f10(h: &mut H) {
    h.app.handle(Input::Key(Key {
        code: KeyCode::F(10),
        mods: KeyModifiers::SHIFT,
    }));
    h.draw();
}

#[test]
fn a_right_click_on_a_grid_cell_opens_its_menu_at_the_pointer() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    let grid = table(&h).grid.id;
    let cell = area(&h, table(&h).grid.cell_id(2, 1));
    let (x, y) = (cell.x + 1, cell.y);
    h.mouse(MouseKind::Secondary, x, y);
    let m = menu(&h);
    assert_eq!(m.target, Target::Grid(2, 1));
    assert_eq!(
        m.menu.anchor,
        Rect::new(x, y, 1, 1),
        "anchored at the pointer"
    );
    assert_eq!(m.menu.area.y, y + 1, "the menu opens below the pointer");
    assert!(m.menu.area.x <= x && x < m.menu.area.right());
    assert_eq!(
        table(&h).grid.cursor,
        (2, 1),
        "the cursor moves to the cell"
    );
    assert_eq!(h.focus(), Some(grid));
    let text = h.text();
    let col = table(&h).grid.columns[1].name.clone();
    assert!(
        text.contains(&format!("{col} · row 3")),
        "titled by the cell"
    );
    for item in [
        "Copy cell",
        "Edit cell",
        "Filter by this value",
        "Delete row",
    ] {
        assert!(text.contains(item), "{item} listed");
    }
    assert!(
        text.contains("No pending changes"),
        "Save is unavailable, with its reason"
    );
    // the menu's command runs the grid's own `-`
    choose(&mut h, "Delete row");
    assert!(h.app.modal().is_none(), "choosing closes the menu");
    assert_eq!(h.focus(), Some(grid), "focus returns to the grid");
    let g = &table(&h).grid;
    assert!(
        g.pending.deleted.contains(&g.source_row(2)),
        "row 3 deleted"
    );
    // now the row offers its restore, and the pending edit can be saved
    h.mouse(MouseKind::Secondary, x, y);
    let text = h.text();
    assert!(text.contains("Restore row"));
    assert!(!text.contains("No pending changes"));
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none(), "Esc dismisses");
    assert_eq!(h.focus(), Some(grid));
}

#[test]
fn shift_f10_opens_the_grid_menu_under_the_cursor_cell() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    let grid = table(&h).grid.id;
    assert_eq!(h.focus(), Some(grid));
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Right);
    let cell = area(&h, table(&h).grid.cell_id(2, 1));
    shift_f10(&mut h);
    let m = menu(&h);
    assert_eq!(m.target, Target::Grid(2, 1));
    assert_eq!(m.menu.area.y, cell.y + 1, "right under the cursor cell");
    // keyboard choice: Copy cell is first
    h.key(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert!(
        h.app
            .status
            .as_ref()
            .is_some_and(|s| s.0.starts_with("Copied")),
        "the grid's y ran"
    );
    // the knob turns the key off
    h.app.context = ContextConfig::DEFAULT.keys(ContextKeys::NONE);
    shift_f10(&mut h);
    assert!(h.app.modal().is_none(), "no context key configured");
}

#[test]
fn a_right_click_outside_an_open_menu_reopens_it_at_the_new_spot() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    let cell = area(&h, table(&h).grid.cell_id(1, 0));
    h.mouse(MouseKind::Secondary, cell.x + 1, cell.y);
    assert!(matches!(menu(&h).target, Target::Grid(1, 0)));
    // a right-click on the first tab of the strip, outside the menu
    let tab = area(&h, h.wb().strip.tab_id(0));
    h.mouse(MouseKind::Secondary, tab.x + 1, tab.y);
    let m = menu(&h);
    assert_eq!(
        m.target,
        Target::Tab(0),
        "the old menu closed, the tab's opened"
    );
    assert_eq!(m.menu.anchor, Rect::new(tab.x + 1, tab.y, 1, 1));
    assert_eq!(
        m.menu.area.y,
        tab.y + 1,
        "the new menu opens at the new spot"
    );
    let below = m.menu.area.bottom();
    assert_eq!(h.app.modals.len(), 1, "one menu at a time");
    // a right-click on an explorer row below that menu: the row's menu
    let i = (0..h.wb().explorer.rows().len())
        .find(|&i| {
            h.app
                .hits
                .area_of(h.wb().explorer.row_id(i))
                .is_some_and(|r| r.y > below)
        })
        .expect("an explorer row below the menu");
    let row = area(&h, h.wb().explorer.row_id(i));
    h.mouse(MouseKind::Secondary, row.x + 2, row.y);
    assert_eq!(menu(&h).target, Target::Explorer(i));
    assert_eq!(h.wb().explorer.cursor, i, "the cursor moves to the row");
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none());
}

#[test]
fn the_tab_menu_closes_the_tab_it_was_opened_on() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    let n = h.wb().tabs.len();
    assert!(n >= 2, "a query tab and orders");
    // the query tab (not the active one)
    let tab = area(&h, h.wb().strip.tab_id(0));
    h.mouse(MouseKind::Secondary, tab.x + 1, tab.y);
    assert!(h.text().contains("Close tab"));
    choose(&mut h, "Close tab");
    assert_eq!(h.wb().tabs.len(), n - 1, "tab 0 closed");
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "orders"));
}

#[test]
fn the_explorer_menu_opens_a_table_and_shift_f10_follows_the_cursor() {
    let mut h = H::connected(120, 40);
    // the cursor on `orders`, as open_orders finds it
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.ticks(3);
    let cursor = h.wb().explorer.cursor;
    let row = area(&h, h.wb().explorer.row_id(cursor));
    shift_f10(&mut h);
    let m = menu(&h);
    assert_eq!(m.target, Target::Explorer(cursor));
    assert_eq!(m.menu.area.y, row.y + 1, "under the cursor row");
    choose(&mut h, "Open");
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "orders"));
    assert!(!table(&h).preview, "Open is Enter: a permanent tab");
}

#[test]
fn a_read_only_result_grid_names_why_edits_are_unavailable() {
    let mut h = H::connected(120, 40);
    h.ctrl('t');
    match h.app.workbench.as_mut().unwrap().active_tab_mut() {
        Some(WorkTab::Query(q)) => q.editor.set_text("SELECT count(*) FROM orders"),
        _ => panic!("expected query tab"),
    }
    h.draw();
    h.ctrl('r');
    h.ticks(10);
    let g = match h.wb().active_tab() {
        Some(WorkTab::Query(q)) => q.active_grid().expect("a result grid"),
        _ => panic!("expected query tab"),
    };
    let cell = area(&h, g.cell_id(0, 0));
    h.mouse(MouseKind::Secondary, cell.x + 1, cell.y);
    let m = menu(&h);
    assert!(matches!(m.target, Target::Results(0, 0)));
    let item = |l: &str| m.menu.items.iter().find(|i| i.label == l).unwrap();
    assert!(!item("Copy cell").disabled);
    assert_eq!(item("Edit cell").reason.as_deref(), Some("Read-only rows"));
    assert_eq!(
        item("Filter by this value").reason.as_deref(),
        Some("Table tabs only")
    );
}

#[test]
fn a_right_click_on_a_connection_row_offers_its_commands() {
    let mut h = H::new(120, 40);
    let tree = h.app.connections.tree().id;
    // the second connection row (row 0 is its group)
    let row = (0..h.app.connections.tree().rows().len())
        .filter(|&r| h.app.connections.connection_at_row(r).is_some())
        .nth(1)
        .expect("two connections");
    let conn = h.app.connections.connection_at_row(row).unwrap();
    let name = h.app.connections.connections[conn].name.clone();
    let r = area(&h, h.app.connections.tree().row_id(row));
    h.mouse(MouseKind::Secondary, r.x + 3, r.y);
    let m = menu(&h);
    assert_eq!(m.target, Target::Connection(row));
    assert_eq!(m.owner, tree);
    assert_eq!(h.app.connections.selected, Some(conn), "selects the row");
    assert!(h.text().contains("Duplicate"));
    choose(&mut h, "Delete…");
    let text = h.text();
    assert!(text.contains("Delete connection?"), "the delete gate opens");
    assert!(text.contains(&name), "for the right-clicked connection");
}

#[test]
fn shift_f10_on_the_connections_list_opens_its_menu_under_the_cursor() {
    let mut h = H::new(120, 40);
    let tree = h.app.connections.tree().id;
    h.app.focus.focus(tree);
    h.draw();
    assert!(
        h.text().contains("Shift+F10 Menu"),
        "the hint bar names the key"
    );
    let cursor = h.app.connections.tree().cursor;
    let row = area(&h, h.app.connections.tree().row_id(cursor));
    shift_f10(&mut h);
    let m = menu(&h);
    assert_eq!(m.target, Target::Connection(cursor));
    assert_eq!(m.menu.area.y, row.y + 1, "under the cursor row");
    choose(&mut h, "New connection…");
    assert!(h.app.modal().is_none());
    assert_eq!(
        h.focus(),
        Some(WidgetId::of("connections").sub("form").sub("name")),
        "the new-connection form opens on its name"
    );
}

#[test]
fn the_history_list_menu_runs_the_right_clicked_query() {
    let mut h = H::connected(120, 40);
    h.ctrl('y');
    let (list, sql) = match h.wb().active_tab() {
        Some(WorkTab::History(t)) => {
            let id = *t.filtered.get(2).expect("three queries");
            let e = h.app.history.entries.iter().find(|e| e.id == id).unwrap();
            (t.list.id, e.sql.clone())
        }
        _ => panic!("expected the history tab"),
    };
    let r = area(&h, list.child(2));
    h.mouse(MouseKind::Secondary, r.x + 2, r.y);
    let m = menu(&h);
    assert_eq!(m.target, Target::History(2));
    assert_eq!(m.owner, list);
    let item = |l: &str| m.menu.items.iter().any(|i| i.label == l);
    assert!(item("Open in new tab") && item("Copy query") && item("Show failed only"));
    choose(&mut h, "Run in new tab");
    assert!(
        matches!(h.wb().active_tab(), Some(WorkTab::Query(q)) if q.editor.text() == sql),
        "the right-clicked query, not the first one"
    );
}

#[test]
fn shift_f10_on_the_history_list_toggles_its_scope() {
    let mut h = H::connected(180, 40);
    h.ctrl('y');
    let list = match h.wb().active_tab() {
        Some(WorkTab::History(t)) => t.list.id,
        _ => panic!("expected the history tab"),
    };
    h.app.focus.focus(list);
    h.draw();
    assert!(
        h.text().contains("Shift+F10 Menu"),
        "listed last: a wide bar shows it"
    );
    shift_f10(&mut h);
    choose(&mut h, "Show all connections");
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::History(t)) if t.scope_all));
}

#[test]
fn the_help_dialog_lists_the_context_menu_key() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Char('?'));
    assert!(h.text().contains("Shift+F10"));
}
