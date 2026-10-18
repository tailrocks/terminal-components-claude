//! Destructive authorization belongs to reviewed content, not only its tab key.
use junie_tui::{App, GridEditor, Id, KeyCode, Theme};
use junie_tui_testing::Harness;
use tablepro_app::{QueryTab, Screen, Tab, TableProApp, Value};

fn pending() -> TableProApp {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let _ = app.run_query("SELECT id, currency FROM orders LIMIT 1");
    let Some((_, view)) = app.workbench.active_grid_mut() else {
        unreachable!("grid")
    };
    assert!(view.model.commit_cell(0, 1, "EUR").is_ok());
    app
}
fn confirm(h: &mut Harness<TableProApp>) {
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
}
fn replace_payload(app: &mut TableProApp) {
    let mut replacement = QueryTab::new(900, "SELECT id, currency FROM orders LIMIT 4");
    assert!(replacement.execute(&app.workbench.catalog).is_ok());
    let Some(view) = replacement.result.as_mut() else {
        unreachable!("result")
    };
    assert!(view.model.commit_cell(0, 1, "GBP").is_ok());
    let Some(key) = app.workbench.active_key() else {
        unreachable!("key")
    };
    let Some(payload) = app.workbench.tab_mut(key) else {
        unreachable!("payload")
    };
    *payload = Tab::Query(replacement);
}
#[test]
fn same_key_payload_replacement_invalidates_result_and_reconnect_confirmations() {
    for reconnect in [false, true] {
        let mut app = pending();
        if reconnect {
            app.screen = Screen::Connections;
        }
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        if reconnect {
            let _ = h.click_id(Id::root("tablepro.connections.details"));
        } else {
            let _ = h.ctrl('r');
        }
        replace_payload(h.app_mut());
        let key = h.app().workbench.active_key();
        h.draw();
        confirm(&mut h);
        assert_eq!(h.app().workbench.active_key(), key);
        assert_eq!(h.app().result().row_count(), 4);
        assert_eq!(h.app().result().pending_total(), 1);
        assert_eq!(
            h.app().result().pending().value(0, 1),
            Some(&Value::Text("GBP".to_owned()))
        );
        assert!(h.find("Work changed").is_some());
        assert!(!h.app().should_quit());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn all_public_mutable_payload_accessors_invalidate_existing_authorization() {
    for access in 0..3 {
        let mut h = Harness::new(pending(), Theme::junie(), 120, 40);
        let _ = h.ctrl('r');
        match access {
            0 => {
                let Some(key) = h.app().workbench.active_key() else {
                    unreachable!("key")
                };
                let _ = h.app_mut().workbench.tab_mut(key);
            }
            1 => {
                let _ = h.app_mut().workbench.active_mut();
            }
            _ => {
                let _ = h.app_mut().workbench.active_grid_mut();
            }
        }
        confirm(&mut h);
        assert_eq!(h.app().result().pending_total(), 1);
        assert!(h.find("Work changed").is_some());
        // A new review authorizes the unchanged current payload.
        let _ = h.ctrl('r');
        for _ in 0..4 {
            h.draw();
        }
        confirm(&mut h);
        assert_eq!(h.app().result().pending_total(), 0);
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn quit_and_close_refuse_replaced_work_until_a_fresh_review() {
    for quit in [false, true] {
        let mut h = Harness::new(pending(), Theme::junie(), 120, 40);
        if quit {
            let _ = h.ctrl('q');
        } else {
            assert!(h.tab_to(Id::root("tablepro.workbench.tab-strip")));
            let _ = h.key(KeyCode::Char('x'));
        }
        replace_payload(h.app_mut());
        let key = h.app().workbench.active_key();
        confirm(&mut h);
        assert!(!h.app().should_quit());
        assert_eq!(h.app().workbench.active_key(), key);
        assert_eq!(h.app().result().row_count(), 4);
        if quit {
            let _ = h.ctrl('q');
        } else {
            assert!(h.tab_to(Id::root("tablepro.workbench.tab-strip")));
            let _ = h.key(KeyCode::Char('x'));
        }
        confirm(&mut h);
        if quit {
            assert!(h.app().should_quit());
        } else {
            assert_ne!(h.app().workbench.active_key(), key);
        }
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn close_only_binds_target_but_quit_binds_every_tab() {
    for quit in [false, true] {
        let mut h = Harness::new(pending(), Theme::junie(), 120, 40);
        let target = h.app().workbench.active_key();
        if quit {
            let _ = h.ctrl('q');
        } else {
            assert!(h.tab_to(Id::root("tablepro.workbench.tab-strip")));
            let _ = h.key(KeyCode::Char('x'));
        }
        let _ = h.app_mut().workbench.new_query("new neighboring work");
        confirm(&mut h);
        assert!(!h.app().should_quit());
        let Some(target) = target else {
            unreachable!("target")
        };
        assert_eq!(h.app().workbench.tab(target).is_some(), quit);
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn quit_from_connection_screen_still_reviews_retained_workbench() {
    let mut app = pending();
    app.screen = Screen::Connections;
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.ctrl('q');
    assert!(!h.app().should_quit());
    assert!(h.find("Quit TablePro?").is_some());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().result().pending_total(), 1);
}

#[test]
fn mutable_table_access_invalidates_quit_scope() {
    let mut app = TableProApp::default();
    app.set_surface(tablepro_app::Surface::PendingChangeBar);
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.ctrl('q');
    assert!(h.app_mut().workbench.active_table_mut().is_some());
    confirm(&mut h);
    assert!(!h.app().should_quit());
    assert_eq!(h.app().result().pending_total(), 1);
}

#[test]
fn whole_workbench_replacement_cannot_reuse_destructive_authorization() {
    for action in 0..4 {
        let mut app = pending();
        if action == 1 {
            app.screen = Screen::Connections;
        }
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        match action {
            0 => {
                let _ = h.ctrl('r');
            }
            1 => {
                let _ = h.click_id(Id::root("tablepro.connections.details"));
            }
            2 => {
                assert!(h.tab_to(Id::root("tablepro.workbench.tab-strip")));
                let _ = h.key(KeyCode::Char('x'));
            }
            _ => {
                let _ = h.ctrl('q');
            }
        }
        let key = h.app().workbench.active_key();
        h.app_mut().workbench = pending().workbench;
        assert_eq!(h.app().workbench.active_key(), key);
        confirm(&mut h);
        assert!(!h.app().should_quit());
        assert_eq!(h.app().workbench.active_key(), key);
        assert_eq!(h.app().result().pending_total(), 1);
        assert!(h.find("Work changed").is_some());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
