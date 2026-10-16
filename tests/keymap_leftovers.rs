//! The last fixed keys move into `NavKeys` and `KeyMap`, and type-ahead
//! reaches menus, tables and grids (api #14, #17).

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::core::keymap::{KeyBinding, NavKeys};
use junie_tui::widgets::choice_dialog::ChoiceDialog;
use junie_tui::widgets::form_dialog::FormDialog;
use junie_tui::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
use junie_tui::widgets::help::{HelpCmd, HelpOverlay};
use junie_tui::widgets::info_dialog::{InfoCmd, InfoDialog, InfoResult};
use junie_tui::widgets::menu::{ContextMenu, MenuBar, MenuBarAction, MenuItem};
use junie_tui::widgets::props::{Prop, PropsAction, PropsCmd, PropsList};
use junie_tui::widgets::table::{Cell, Column, DataTable};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Constraint;

fn key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn ch(c: char) -> Key {
    key(KeyCode::Char(c))
}

#[test]
fn choice_dialog_buttons_follow_its_nav_keys() {
    let ring = FocusRing::default();
    let make = || ChoiceDialog::new(WidgetId::of("c"), "Pick", "Kind", &["a", "b"], 0);
    let mut d = make();
    let mut focus = Focus::default();
    focus.focus(d.buttons[0].id);
    assert_eq!(d.on_key(&ch('l'), &mut focus, &ring), Outcome::Changed);
    assert_eq!(focus.current(), Some(d.buttons[1].id), "vim `l` by default");
    d.on_key(&key(KeyCode::Left), &mut focus, &ring);
    assert_eq!(focus.current(), Some(d.buttons[0].id));
    let mut d = make().nav(NavKeys::NO_VIM);
    focus.focus(d.buttons[0].id);
    d.on_key(&ch('l'), &mut focus, &ring);
    assert_eq!(focus.current(), Some(d.buttons[0].id), "no vim, no move");
    d.on_key(&key(KeyCode::Right), &mut focus, &ring);
    assert_eq!(focus.current(), Some(d.buttons[1].id));
}

#[test]
fn form_dialog_buttons_follow_its_nav_keys() {
    let ring = FocusRing::default();
    let mut f = FormDialog::new(WidgetId::of("f"), "Form", vec![]);
    let mut focus = Focus::default();
    focus.focus(f.cancel.id);
    f.on_key(&ch('l'), &mut focus, &ring);
    assert_eq!(
        focus.current(),
        Some(f.cancel.id),
        "letters stay with fields"
    );
    f.on_key(&key(KeyCode::Right), &mut focus, &ring);
    assert_eq!(focus.current(), Some(f.save.id));
    let mut f = FormDialog::new(WidgetId::of("f"), "Form", vec![]).nav(NavKeys::DEFAULT);
    focus.focus(f.cancel.id);
    f.on_key(&ch('l'), &mut focus, &ring);
    assert_eq!(focus.current(), Some(f.save.id), "opted into vim");
}

#[test]
fn info_dialog_keys_are_a_keymap_and_nav_keys() {
    let ring = FocusRing::default();
    let props = vec![Prop::new("Path", "/tmp/x").copyable()];
    let mut focus = Focus::default();
    let mut d = InfoDialog::new(WidgetId::of("i"), "Info", props.clone());
    focus.focus(d.close.id);
    d.on_key(&ch('y'), &mut focus, &ring);
    assert_eq!(d.result, Some(InfoResult::Copy("/tmp/x".into())));
    let mut d = InfoDialog::new(WidgetId::of("i"), "Info", props.clone());
    d.on_key(&ch('q'), &mut focus, &ring);
    assert_eq!(d.result, Some(InfoResult::Closed));
    let mut d = InfoDialog::new(WidgetId::of("i"), "Info", props)
        .keymap(vec![(KeyBinding::char('x'), InfoCmd::Close)])
        .nav(NavKeys::NO_VIM);
    assert_eq!(d.on_key(&ch('q'), &mut focus, &ring), Outcome::Consumed);
    assert_eq!(d.result, None, "q is unbound");
    assert_eq!(d.on_key(&ch('j'), &mut focus, &ring), Outcome::Consumed);
    d.on_key(&ch('x'), &mut focus, &ring);
    assert_eq!(d.result, Some(InfoResult::Closed));
}

#[test]
fn props_copy_and_help_close_are_keymaps() {
    let props = || vec![Prop::new("Path", "/tmp/x").copyable()];
    let mut p = PropsList::new(WidgetId::of("p"), props());
    assert_eq!(p.on_key(&ch('y')).1, Some(PropsAction::Copy(0)));
    let mut p = PropsList::new(WidgetId::of("p"), props())
        .keymap(vec![(KeyBinding::char('c'), PropsCmd::Copy)]);
    assert_eq!(p.on_key(&ch('y')).1, None);
    assert_eq!(p.on_key(&ch('c')).1, Some(PropsAction::Copy(0)));

    let help = || HelpOverlay::new(WidgetId::of("h"), "App", vec![("Move", vec![("k", "Up")])]);
    let mut h = help();
    h.on_key(&ch('?'));
    assert!(h.closed);
    let mut h = help();
    h.on_key(&ch('q'));
    assert!(h.closed);
    let mut h = help().keymap(vec![(KeyBinding::char('x'), HelpCmd::Close)]);
    h.on_key(&ch('q'));
    assert!(!h.closed, "q unbound");
    h.on_key(&key(KeyCode::Enter));
    assert!(h.closed, "Enter always closes");
}

fn menu_items() -> Vec<MenuItem> {
    ["Copy", "Cut", "Paste", "Rename", "Reveal"]
        .into_iter()
        .map(MenuItem::new)
        .collect()
}

#[test]
fn context_menus_take_type_ahead_when_asked() {
    let mut m = ContextMenu::new(WidgetId::of("m"), menu_items());
    m.on_key(&ch('p'));
    assert_eq!(m.cursor, 0, "off by default");
    let mut m = ContextMenu::new(WidgetId::of("m"), menu_items()).type_ahead(true);
    m.on_key(&ch('p'));
    assert_eq!(m.cursor, 2);
    m.on_key(&ch('r'));
    // "pr" matches nothing: the cursor stays
    assert_eq!(m.cursor, 2);
    let mut m = ContextMenu::new(WidgetId::of("m"), menu_items()).type_ahead(true);
    m.on_key(&ch('r'));
    assert_eq!(m.cursor, 3);
    m.on_key(&ch('r'));
    assert_eq!(m.cursor, 4, "a repeated letter cycles");
    // a menu bar hands its open menu the letters, never `l` to the next title
    let mut bar = MenuBar::new(
        WidgetId::of("bar"),
        vec![("Edit", menu_items()), ("View", menu_items())],
    )
    .type_ahead(true);
    bar.open_menu(0);
    let (_, ev) = bar.on_key(&ch('r'));
    assert_eq!(ev, None);
    assert_eq!(bar.open_index(), Some(0));
    assert_eq!(bar.open.as_ref().unwrap().cursor, 3);
    let (_, ev) = bar.on_key(&key(KeyCode::Right));
    assert_eq!(ev, Some(MenuBarAction::Opened(1)), "arrows still switch");
}

#[test]
fn tables_and_grids_take_type_ahead_when_asked() {
    let names = ["alpha", "bravo", "charlie", "carol"];
    let table = || {
        DataTable::new(
            WidgetId::of("t"),
            vec![
                Column::new("id", Constraint::Length(4)),
                Column::new("name", Constraint::Length(10)),
            ],
            names
                .iter()
                .enumerate()
                .map(|(i, n)| vec![Cell::new(i.to_string()), Cell::new(*n)])
                .collect(),
        )
    };
    let mut t = table();
    t.on_key(&ch('c'));
    assert_eq!(t.cursor_row, 0, "off by default");
    let mut t = table().type_ahead(true).type_ahead_column(Some(1));
    t.on_key(&ch('c'));
    assert_eq!(t.cursor_row, 2);
    t.on_key(&ch('a'));
    assert_eq!(t.cursor_row, 3, "`ca` is carol");

    let grid = || {
        let mut g = DataGrid::new(
            WidgetId::of("g"),
            vec![
                ColumnSpec::new("name", CellKind::Text),
                ColumnSpec::new("qty", CellKind::Number),
            ],
        );
        g.set_rows(GridRows {
            rows: names
                .iter()
                .enumerate()
                .map(|(i, n)| vec![CellValue::Text((*n).into()), CellValue::Int(i as i64)])
                .collect(),
            total: RowTotal::Exact(4),
            more: false,
        });
        g
    };
    let mut g = grid();
    g.on_key(&ch('b'));
    assert_eq!(g.cursor.0, 0, "off by default");
    let mut g = grid().type_ahead(true);
    // the cursor's column (name) by default
    g.on_key(&ch('b'));
    assert_eq!(g.cursor.0, 1);
}
