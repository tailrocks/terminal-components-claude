//! Structural tab records keep identity outside cloneable/replaced payloads.
use junie_tui::{App, Id, KeyCode, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{QueryTab, Surface, Tab, TableProApp};
#[test]
fn cloned_payload_receives_fresh_identity_and_clean_close_keeps_dirty_owner() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::PendingChangeBar);
    let Some(original) = app.workbench.active_key() else {
        unreachable!("key")
    };
    let Some(payload) = app.workbench.active().cloned() else {
        unreachable!("payload")
    };
    let Some(copy) = app.workbench.insert_tab(0, payload) else {
        unreachable!("insert")
    };
    assert_ne!(copy, original);
    let Some(payload) = app.workbench.tab_mut(copy) else {
        unreachable!("copy")
    };
    *payload = Tab::Query(QueryTab::new(99, ""));
    assert_eq!(
        app.workbench
            .tabs()
            .first()
            .map(tablepro_app::TabRecord::key),
        Some(copy)
    );
    assert!(app.workbench.close_tab(0));
    assert!(
        matches!(app.workbench.tab(original),Some(Tab::Table(table)) if table.result.pending_total()==1)
    );
}
#[test]
fn captured_close_rejects_payload_replacement_after_clone_insertion_and_reorder() {
    let mut app = TableProApp::default();
    app.set_surface(Surface::PendingChangeBar);
    let Some(original) = app.workbench.active_key() else {
        unreachable!("key")
    };
    let Some(payload) = app.workbench.active().cloned() else {
        unreachable!("payload")
    };
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    assert!(h.tab_to(Id::root("tablepro.workbench.tab-strip")));
    let _ = h.key(KeyCode::Char('x'));
    assert!(h.find("Close tab with unsaved work?").is_some());
    let Some(copy) = h.app_mut().workbench.insert_tab(0, payload) else {
        unreachable!("copy")
    };
    let mut order: Vec<_> = h
        .app()
        .workbench
        .tabs()
        .iter()
        .map(tablepro_app::TabRecord::key)
        .collect();
    order.reverse();
    assert!(h.app_mut().workbench.reorder_tabs(&order));
    let Some(target) = h.app_mut().workbench.tab_mut(original) else {
        unreachable!("target")
    };
    *target = Tab::Query(QueryTab::new(100, "replacement payload"));
    let count = h.app().workbench.tabs().len();
    h.draw();
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.tabs().len(), count);
    assert!(h.app().workbench.tab(original).is_some());
    assert!(h.find("Work changed").is_some());
    assert!(
        matches!(h.app().workbench.tab(copy),Some(Tab::Table(table)) if table.result.pending_total()==1)
    );
    assert!(!h.app().should_quit());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn invalid_reorder_and_insert_refuse_without_mutating_identity_or_active_tab() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let _ = app.workbench.new_query("");
    let keys: Vec<_> = app
        .workbench
        .tabs()
        .iter()
        .map(tablepro_app::TabRecord::key)
        .collect();
    let active = app.workbench.active_key();
    let Some(first) = keys.first().copied() else {
        unreachable!("first")
    };
    assert!(!app.workbench.reorder_tabs(&[first, first]));
    assert!(!app.workbench.reorder_tabs(&[]));
    assert!(
        app.workbench
            .insert_tab(usize::MAX, Tab::Query(QueryTab::new(20, "")))
            .is_none()
    );
    assert_eq!(
        app.workbench
            .tabs()
            .iter()
            .map(tablepro_app::TabRecord::key)
            .collect::<Vec<_>>(),
        keys
    );
    assert_eq!(app.workbench.active_key(), active);
}
