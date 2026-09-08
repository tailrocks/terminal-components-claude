//! Inline cell drafts remain owned across blur and guarded exit.
use junie_tui::{App, Dialog, Id, KeyCode, Part, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{GridView, Surface, Tab, TableProApp, Value};

fn draft(surface: Surface, value: &str) -> Harness<TableProApp> {
    let mut app = TableProApp::default();
    app.set_surface(surface);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let Some(id) = h.app().result_id() else {
        unreachable!("grid")
    };
    assert!(h.tab_to(id));
    for _ in 0..6 {
        let _ = h.key(KeyCode::Right);
    }
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    for _ in 0..3 {
        let _ = h.key(KeyCode::Backspace);
    }
    let _ = h.type_str(value);
    h
}

fn view(h: &Harness<TableProApp>) -> &GridView {
    let Some((_, view)) = h.app().workbench.active().and_then(Tab::grid) else {
        unreachable!("grid")
    };
    view
}

#[test]
fn changed_inline_draft_requires_quit_confirmation_without_committing_on_cancel() {
    let mut h = draft(Surface::TableGrid, "EUR");
    assert_eq!(h.app().result().pending_total(), 0);
    let cell = view(&h).state.edit_cell();
    let _ = h.ctrl('c');
    assert!(!h.app().should_quit());
    assert!(h.find("1 pending row change will be lost.").is_some());
    for _ in 0..3 {
        h.draw();
    }
    assert_eq!(view(&h).state.edit_cell(), cell);
    assert_eq!(view(&h).state.edit_draft(), Some("EUR"));
    assert_eq!(view(&h).pending_total(), 1);
    assert_eq!(h.app().result().pending_total(), 0);
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("USD".to_owned()))
    );
    let _ = h.key(KeyCode::Esc);
    assert!(!h.app().should_quit());
    assert_eq!(view(&h).state.edit_draft(), Some("EUR"));
    assert_eq!(h.app().result().pending_total(), 0);
    let _ = h.ctrl('q');
    let _ = h.click_id(Dialog::new(Id::root("tablepro.quit-dialog")).action_id(1));
    assert!(h.app().should_quit());
    assert_eq!(h.app().result().pending_total(), 0);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn draft_survives_tab_and_structure_blur_then_explicit_enter_commits() {
    let mut h = draft(Surface::TableGrid, "EUR");
    let Some(key) = h.app().workbench.active_key() else {
        unreachable!("table")
    };
    let _ = h.ctrl('t');
    assert!(h.tab_to(Id::root("tablepro.workbench.tab-strip")));
    let _ = h.key(KeyCode::Left);
    assert_eq!(h.app().workbench.active_key(), Some(key));
    assert_eq!(view(&h).state.edit_draft(), Some("EUR"));
    let _ = h.ctrl('d');
    let _ = h.ctrl('d');
    assert_eq!(view(&h).state.edit_draft(), Some("EUR"));
    assert_eq!(h.app().result().pending_total(), 0);
    let editor = key.control("data").part(Part::TEXT);
    assert!(h.tab_to(editor));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(view(&h).state.edit_draft(), None);
    assert_eq!(h.app().result().pending_total(), 1);
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("EUR".to_owned()))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn existing_row_update_is_not_double_counted_and_escape_discards_only_inline_draft() {
    let mut h = draft(Surface::PendingChangeBar, "GBP");
    assert_eq!(h.app().result().pending_total(), 1);
    assert_eq!(view(&h).pending_total(), 1);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(view(&h).state.edit_draft(), None);
    assert_eq!(view(&h).pending_total(), 1);
    assert_eq!(
        h.app().result().pending().value(0, 6),
        Some(&Value::Text("EUR".to_owned()))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn unchanged_inline_draft_does_not_invent_pending_work() {
    let mut h = draft(Surface::TableGrid, "USD");
    assert_eq!(view(&h).pending_total(), 0);
    let _ = h.ctrl('c');
    assert!(h.app().should_quit());
    assert_eq!(h.app().result().pending_total(), 0);
}

#[test]
fn inserted_row_and_deleted_row_accounting_keep_distinct_operations() {
    let mut deleted = draft(Surface::TableGrid, "EUR");
    let Some(table) = deleted.app_mut().workbench.active_table_mut() else {
        unreachable!("table")
    };
    assert!(table.result.model.delete_row(0));
    assert_eq!(deleted.app().result().pending_total(), 1);
    assert_eq!(
        view(&deleted).pending_total(),
        2,
        "delete plus uncommitted update are distinct operations"
    );

    let mut app = TableProApp::default();
    app.set_surface(Surface::TableGrid);
    let Some(table) = app.workbench.active_table_mut() else {
        unreachable!("table")
    };
    assert!(table.result.model.insert_row().is_some());
    let mut inserted = Harness::new(app, Theme::junie(), 120, 40);
    let Some(id) = inserted.app().result_id() else {
        unreachable!("grid")
    };
    assert!(inserted.tab_to(id));
    let _ = inserted.key_mod(KeyCode::End, junie_tui::KeyModifiers::CONTROL);
    for _ in 0..6 {
        let _ = inserted.key(KeyCode::Right);
    }
    let _ = inserted.key(KeyCode::F(2));
    let _ = inserted.type_str("EUR");
    assert!(view(&inserted).state.edit_draft().is_some());
    assert_eq!(inserted.app().result().pending_total(), 1);
    assert_eq!(
        view(&inserted).pending_total(),
        1,
        "inline value belongs to the already-counted inserted row"
    );
    assert!(
        inserted.diagnostics().is_empty(),
        "{:?}",
        inserted.diagnostics()
    );
}
