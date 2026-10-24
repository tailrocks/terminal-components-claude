use tablepro_ui::{TabRecord, TableProApp};
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

const TAB_LIST: Id = Id::root("tablepro.tab-list");
const DESTRUCTIVE: Id = Id::root("tablepro.quit-dialog");

fn connected_app() -> TableProApp {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    app
}

#[test]
fn replaced_workbench_rejects_tab_list_choose_and_delete() {
    for action in [KeyCode::Enter, KeyCode::Delete] {
        let mut h = Harness::new(connected_app(), Theme::junie(), 120, 40);
        assert!(h.app_mut().workbench.new_query("").is_some());
        h.draw();
        let _ = h.ctrl('g');
        let _ = h.type_str("Query 1");
        assert!(h.layer_area(TAB_LIST).is_some());

        let mut replacement = connected_app();
        let replacement_active = replacement.workbench.new_query("");
        assert!(replacement_active.is_some());
        let replacement_keys: Vec<_> = replacement
            .workbench
            .tabs()
            .iter()
            .map(TabRecord::key)
            .collect();
        h.app_mut().workbench = replacement.workbench;
        let _ = h.key(action);

        assert_eq!(h.app().workbench.active_key(), replacement_active);
        assert_eq!(
            h.app()
                .workbench
                .tabs()
                .iter()
                .map(TabRecord::key)
                .collect::<Vec<_>>(),
            replacement_keys,
        );
        assert!(h.app().status().contains("Workbench changed"));
        assert!(h.layer_area(TAB_LIST).is_none());
        assert!(h.layer_area(DESTRUCTIVE).is_none());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn tab_list_keeps_stable_keys_after_reorder() {
    let mut h = Harness::new(connected_app(), Theme::junie(), 120, 40);
    let first = h.app().workbench.active_key();
    assert!(h.app_mut().workbench.new_query("").is_some());
    h.draw();
    let _ = h.ctrl('g');
    let _ = h.type_str("Query 1");
    let order: Vec<_> = h
        .app()
        .workbench
        .tabs()
        .iter()
        .rev()
        .map(TabRecord::key)
        .collect();
    assert!(h.app_mut().workbench.reorder_tabs(&order));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().workbench.active_key(), first);
    assert!(h.layer_area(TAB_LIST).is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
