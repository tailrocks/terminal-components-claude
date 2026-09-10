//! Closing a tab requires one explicit, stable-target destructive intent.
use junie_tui::{App, Id, ItemKey, KeyCode, Part, PartRef, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{Surface, Tab, TableProApp};
const STRIP: Id = Id::root("tablepro.workbench.tab-strip");
fn dirty_table() -> TableProApp {
    let mut app = TableProApp::default();
    app.set_surface(Surface::PendingChangeBar);
    app
}
fn confirm(h: &mut Harness<TableProApp>) {
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
}
#[test]
fn public_close_refuses_dirty_tab_and_keyboard_cancel_preserves_it() {
    let mut app = dirty_table();
    let key = app.workbench.active_key();
    let index = app.workbench.active_index().unwrap_or(0);
    assert!(!app.workbench.close_tab(index));
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    assert!(h.tab_to(STRIP));
    let _ = h.key(KeyCode::Char('x'));
    assert!(h.find("Close tab with unsaved work?").is_some());
    assert_eq!(h.app().workbench.active_key(), key);
    assert_eq!(h.app().result().pending_total(), 1);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(STRIP));
    assert_eq!(h.app().result().pending_total(), 1);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn mouse_close_confirmation_targets_key_after_neighbor_reorder() {
    let mut app = dirty_table();
    let Some(key) = app.workbench.active_key() else {
        unreachable!("key")
    };
    app.workbench.new_query("");
    assert!(app.workbench.activate(key));
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.click_part(STRIP, PartRef::item(Part::CLOSE, ItemKey::num(key.get())));
    assert!(h.find("Close tab with unsaved work?").is_some());
    let mut keys: Vec<_> = h
        .app()
        .workbench
        .tabs()
        .iter()
        .map(tablepro_app::TabRecord::key)
        .collect();
    keys.reverse();
    assert!(h.app_mut().workbench.reorder_tabs(&keys));
    h.draw();
    confirm(&mut h);
    assert!(!h.app().workbench.tabs().iter().any(|tab| tab.key() == key));
    assert_eq!(h.app().workbench.tabs().len(), 2);
    assert!(!h.app().should_quit());
    let count = h.app().workbench.tabs().len();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.tabs().len(), count);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn query_and_inline_drafts_survive_cancel_without_commit() {
    for query in [false, true] {
        let mut app = TableProApp::default();
        app.set_surface(Surface::TableGrid);
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        if query {
            let _ = h.ctrl('t');
            let Some(id) = h.app().query_id() else {
                unreachable!("query")
            };
            assert!(h.tab_to(id));
            let _ = h.type_str("SELECT private_draft");
        } else {
            let Some(id) = h.app().result_id() else {
                unreachable!("grid")
            };
            assert!(h.tab_to(id));
            for _ in 0..6 {
                let _ = h.key(KeyCode::Right);
            }
            let _ = h.key(KeyCode::F(2));
            let _ = h.key(KeyCode::End);
            let _ = h.type_str(" draft");
        }
        let key = h.app().workbench.active_key();
        assert!(h.tab_to(STRIP));
        let _ = h.key(KeyCode::Delete);
        assert!(h.find("Close tab with unsaved work?").is_some());
        let _ = h.key(KeyCode::Esc);
        assert_eq!(h.app().workbench.active_key(), key);
        if query {
            let Some(Tab::Query(tab)) = h.app().workbench.active() else {
                unreachable!("query")
            };
            assert_eq!(tab.editor_state.draft_text(), Some("SELECT private_draft"));
        } else {
            let Some((_, view)) = h.app().workbench.active_grid() else {
                unreachable!("grid")
            };
            assert_eq!(view.state.edit_draft(), Some("USD draft"));
            assert_eq!(view.model.pending_total(), 0);
        }
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn vanished_confirmation_target_cannot_close_replacement() {
    let app = dirty_table();
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    assert!(h.tab_to(STRIP));
    let _ = h.key(KeyCode::Char('x'));
    let Some(key) = h.app().workbench.active_key() else {
        unreachable!("key")
    };
    // Explicit payload replacement keeps the record identity but makes it clean,
    // so the public close API can remove the captured target before confirmation.
    if let Some(payload) = h.app_mut().workbench.tab_mut(key) {
        *payload = Tab::Query(tablepro_app::QueryTab::new(99, ""));
    }
    let Some(index) = h.app().workbench.active_index() else {
        unreachable!("target")
    };
    assert!(h.app_mut().workbench.close_tab(index));
    h.app_mut().workbench.new_query("");
    let replacement = h.app().workbench.active_key();
    h.draw();
    confirm(&mut h);
    assert_eq!(h.app().workbench.active_key(), replacement);
    assert!(
        h.app()
            .workbench
            .tabs()
            .iter()
            .any(|tab| Some(tab.key()) == replacement)
    );
}
