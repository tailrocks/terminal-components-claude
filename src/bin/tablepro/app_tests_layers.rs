//! TablePro's layer stack (plan S5): every layer kind the app opens is
//! dismissed through the stack under its own policy (Esc, an outside
//! click, the frame's `×`, a refused click that flashes), and closing it
//! gives the focus back to the control it covered (K9).

use junie_tui::runtime::Application;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use junie_tui::core::event::{Input, Key};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::modal::{CLOSE, SURFACE};
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::picker::Picker;

use crate::app_tests::H;
use crate::context::CtxMenu;
use crate::filter_editor::FilterEditor;
use crate::workbench::WorkTab;

fn open_orders(h: &mut H) {
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
}

fn grid(h: &H) -> WidgetId {
    match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => t.grid.id,
        _ => panic!("expected a table tab"),
    }
}

fn shift_f10(h: &mut H) {
    h.app.handle(Input::Key(Key {
        code: KeyCode::F(10),
        mods: KeyModifiers::SHIFT,
    }));
    h.draw();
}

/// A click on the dimmed page, away from every layer.
fn click_outside(h: &mut H) {
    h.click(2, 30);
}

/// The frame flashes: a refused outside click under `Flash` feedback.
fn flashing(h: &H) -> bool {
    h.app.interaction().flash == Some(SURFACE)
}

#[test]
fn the_filter_editor_closes_on_esc_only_and_gives_the_grid_its_focus_back() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    let g = grid(&h);
    assert_eq!(h.focus(), Some(g));
    h.key(KeyCode::Char('f'));
    assert!(h.app.top_as::<FilterEditor>().is_some(), "{}", h.text());
    assert_ne!(h.focus(), Some(g), "focus moved into the editor");
    // ESCAPE: a stray click outside is refused, and the frame says so
    click_outside(&mut h);
    assert!(h.app.top_as::<FilterEditor>().is_some(), "the form stays");
    assert!(flashing(&h), "a refused click flashes the frame");
    // Esc dismisses it as cancelled: no filter, focus back on the grid
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none(), "{}", h.text());
    assert_eq!(h.focus(), Some(g), "K9: the grid has its focus back");
    match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => assert!(t.filters.is_empty(), "nothing applied"),
        _ => unreachable!(),
    }
}

#[test]
fn the_filter_editor_applies_through_its_result() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    let g = grid(&h);
    h.key(KeyCode::Char('f'));
    // prefilled from the cursor cell, it opens on its Apply button
    let apply = h.app.top_as::<FilterEditor>().unwrap().apply.id;
    assert_eq!(h.focus(), Some(apply));
    h.key(KeyCode::Enter);
    assert!(h.app.modal().is_none(), "{}", h.text());
    assert_eq!(h.focus(), Some(g));
    match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => assert_eq!(t.filters.len(), 1),
        _ => unreachable!(),
    }
}

#[test]
fn a_dialog_closes_on_esc_on_its_close_button_and_on_an_outside_click() {
    let mut h = H::connected(120, 40);
    let before = h.focus();
    h.key(KeyCode::Char('?'));
    assert!(h.app.top_as::<Dialog>().is_some(), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none(), "Esc closes the help");
    assert_eq!(h.focus(), before, "K9");

    h.key(KeyCode::Char('?'));
    let x = h
        .app
        .shell
        .hits
        .area_of(CLOSE)
        .expect("the frame draws its ×");
    h.click(x.x + 1, x.y);
    assert!(h.app.modal().is_none(), "× closes the help\n{}", h.text());
    assert_eq!(h.focus(), before, "K9");

    h.key(KeyCode::Char('?'));
    click_outside(&mut h);
    assert!(h.app.modal().is_none(), "an outside click closes the help");
    assert_eq!(h.focus(), before, "K9");
}

#[test]
fn the_write_gate_refuses_an_outside_click_and_cancels_on_esc() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.type_str("DELETE FROM orders");
    h.key(KeyCode::Esc);
    let editor = h.focus();
    h.ctrl('r');
    assert!(h.app.top_as::<Dialog>().is_some());
    click_outside(&mut h);
    assert!(
        h.app.top_as::<Dialog>().is_some(),
        "a destructive confirmation stays"
    );
    assert!(flashing(&h), "the refusal flashes");
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none(), "{}", h.text());
    assert!(h.text().contains("Cancelled · nothing was executed"));
    assert!(!h.wb_query().is_running());
    assert_eq!(h.focus(), editor, "K9: back on the editor");
}

#[test]
fn a_picker_closes_on_esc_and_on_an_outside_click() {
    let mut h = H::connected(120, 40);
    let before = h.focus();
    h.ctrl('o');
    assert!(h.app.top_as::<Picker>().is_some());
    assert_eq!(h.app.modal().unwrap().tag.kind, "switcher");
    click_outside(&mut h);
    assert!(h.app.modal().is_none(), "{}", h.text());
    assert_eq!(h.focus(), before, "K9");

    h.ctrl('g');
    assert_eq!(h.app.modal().unwrap().tag.kind, "tablist");
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert_eq!(h.focus(), before, "K9");
}

#[test]
fn the_switcher_scope_cycles_both_ways_through_the_layer_events() {
    let mut h = H::connected(120, 40);
    h.ctrl('o');
    let scope = |h: &H| h.app.top_as::<Picker>().unwrap().scope.clone().unwrap();
    assert!(scope(&h).starts_with("All"));
    h.key(KeyCode::Tab);
    assert!(scope(&h).starts_with("Tables"), "{}", scope(&h));
    h.key(KeyCode::BackTab);
    h.key(KeyCode::BackTab);
    assert!(scope(&h).starts_with("Queries"), "{}", scope(&h));
}

/// Fix: a paste while a picker is open goes into its search, not into the
/// page under it.
#[test]
fn a_paste_lands_in_the_open_pickers_search() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    let typed = h.wb_query().editor.text().to_owned();
    h.ctrl('o');
    h.app.handle(Input::Paste("customers".into()));
    h.draw();
    let p = h.app.top_as::<Picker>().expect("the picker stays open");
    assert_eq!(p.query, "customers");
    assert!(
        p.items.iter().any(|it| it.label == "customers"),
        "the results follow the pasted query"
    );
    assert_eq!(h.wb_query().editor.text(), typed, "the page got nothing");
}

#[test]
fn a_context_menu_closes_on_esc_and_on_an_outside_click_back_to_its_owner() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    let g = grid(&h);
    shift_f10(&mut h);
    assert!(h.app.top_as::<CtxMenu>().is_some(), "{}", h.text());
    assert_eq!(h.focus(), Some(g), "the menu's keyboard stays on its owner");
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert_eq!(h.focus(), Some(g), "K9");

    shift_f10(&mut h);
    click_outside(&mut h);
    assert!(h.app.modal().is_none(), "{}", h.text());
    assert_eq!(h.focus(), Some(g), "K9");
}
