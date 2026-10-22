#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::{Tab, TableProApp};
use termrock::{Diagnostic, Id, KeyCode, MouseKind, Position, Theme};
use termrock_test_support::Harness;

const HELP: Id = Id::root("tablepro.help");
const EXPLORER: Id = Id::root("tablepro.workbench.explorer.tree");

fn two_query_tabs() -> (Harness<TableProApp>, Position, Id) {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    let Some(first) = app.workbench.active_key() else {
        unreachable!("connected fixture has an initial query tab")
    };
    let Some(active) = app.workbench.new_query("SELECT 'active draft'") else {
        unreachable!("workbench accepts a second query tab")
    };
    assert_ne!(first, active);
    assert!(app.workbench.activate(active));

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    assert!(tab_navigation::tab_to(&mut harness, EXPLORER));
    let Some(first_tab) = harness.find("Query 1") else {
        unreachable!("first tab is visible in the strip")
    };
    (harness, Position::new(first_tab.0, first_tab.1), EXPLORER)
}

fn active_query(harness: &Harness<TableProApp>) -> Option<String> {
    match harness.app().workbench.active() {
        Some(Tab::Query(query)) => Some(query.query.clone()),
        _ => None,
    }
}

#[test]
fn help_question_mark_dismissal_preserves_the_underlay() {
    let mut diagnostics_by_dismissal = Vec::new();
    for outside_dismissal in [false, true] {
        let (mut harness, underlay_tab, underlay_focus) = two_query_tabs();
        let active_key = harness.app().workbench.active_key();
        let original_query = active_query(&harness);
        assert!(original_query.is_some(), "active fixture tab is a query");
        let original_count = harness.app().workbench.tabs().len();
        assert_eq!(harness.focus(), Some(underlay_focus));

        let _ = harness.key(KeyCode::Char('?'));
        assert!(harness.layer_area(HELP).is_some());
        assert!(harness.area_of(HELP).is_some());
        assert!(harness.text().contains("Shell and workbench"));
        assert!(harness.text().contains("Ctrl+G"));
        assert_eq!(active_query(&harness), original_query);

        let _ = harness.ctrl('t');
        assert!(harness.layer_area(HELP).is_some(), "help remains modal");
        assert_eq!(harness.app().workbench.tabs().len(), original_count);
        assert_eq!(harness.app().workbench.active_key(), active_key);
        assert_eq!(active_query(&harness), original_query);

        if outside_dismissal {
            let Some(layer) = harness.layer_area(HELP) else {
                unreachable!("help is open before outside dismissal")
            };
            assert!(
                underlay_tab.x < layer.x
                    || underlay_tab.x >= layer.right()
                    || underlay_tab.y < layer.y
                    || underlay_tab.y >= layer.bottom(),
                "first tab is outside the help layer: {layer:?}"
            );
            let _ = harness.mouse(MouseKind::Down, underlay_tab.x, underlay_tab.y);
            let _ = harness.mouse(MouseKind::Up, underlay_tab.x, underlay_tab.y);
        } else {
            let _ = harness.key(KeyCode::Esc);
        }

        assert!(harness.layer_area(HELP).is_none());
        assert!(harness.area_of(HELP).is_none());
        assert_eq!(harness.app().workbench.tabs().len(), original_count);
        assert_eq!(harness.app().workbench.active_key(), active_key);
        assert_eq!(active_query(&harness), original_query);
        assert_eq!(harness.focus(), Some(underlay_focus));
        assert!(!harness.text().contains("Shell and workbench"));
        let diagnostic_count = harness.diagnostics().len();
        harness.draw();
        assert_eq!(harness.diagnostics().len(), diagnostic_count);
        diagnostics_by_dismissal.push(harness.diagnostics().to_vec());
    }
    assert!(
        diagnostics_by_dismissal
            .iter()
            .all(Vec::<Diagnostic>::is_empty),
        "each dismissal should leave no intent after a normal redraw: {diagnostics_by_dismissal:?}"
    );
}

#[test]
fn f1_opens_the_same_modal_help_and_preserves_the_underlay() -> Result<(), &'static str> {
    let (mut harness, _, underlay_focus) = two_query_tabs();
    let active_key = harness.app().workbench.active_key();
    let original_query = active_query(&harness);
    let original_count = harness.app().workbench.tabs().len();
    assert_eq!(harness.focus(), Some(underlay_focus));

    let _ = harness.key(KeyCode::F(1));
    let Some(help_area) = harness.layer_area(HELP) else {
        return Err("F1 opens the modal help overlay");
    };
    let help_layer = harness.top_layer();
    assert!(harness.area_of(HELP).is_some());
    assert!(harness.text().contains("Shell and workbench"));
    assert!(harness.text().contains("Ctrl+G"));
    assert_eq!(active_query(&harness), original_query);

    // F1 remains bound to the same help action while a modal is open. It must
    // not stack another help layer or allow the underlay action to run.
    let _ = harness.key(KeyCode::F(1));
    assert_eq!(harness.layer_area(HELP), Some(help_area));
    assert_eq!(harness.top_layer(), help_layer);
    let _ = harness.ctrl('t');
    assert_eq!(harness.layer_area(HELP), Some(help_area));
    assert_eq!(harness.app().workbench.tabs().len(), original_count);
    assert_eq!(harness.app().workbench.active_key(), active_key);
    assert_eq!(active_query(&harness), original_query);

    let _ = harness.key(KeyCode::Esc);
    assert!(harness.layer_area(HELP).is_none());
    assert_eq!(harness.app().workbench.tabs().len(), original_count);
    assert_eq!(harness.app().workbench.active_key(), active_key);
    assert_eq!(active_query(&harness), original_query);
    assert_eq!(harness.focus(), Some(underlay_focus));
    assert!(!harness.text().contains("Shell and workbench"));
    let diagnostic_count = harness.diagnostics().len();
    harness.draw();
    assert_eq!(harness.diagnostics().len(), diagnostic_count);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn question_mark_still_types_in_the_query_editor() -> Result<(), &'static str> {
    let (mut harness, _, _) = two_query_tabs();
    let Some(active_key) = harness.app().workbench.active_key() else {
        return Err("the active query has a stable key");
    };
    let editor = active_key.control("query");
    assert!(tab_navigation::tab_to(&mut harness, editor));
    assert_eq!(harness.focus(), Some(editor));

    let Some(before) = active_query(&harness) else {
        return Err("the active tab is a query");
    };
    let _ = harness.key(KeyCode::Char('?'));

    assert!(harness.layer_area(HELP).is_none());
    assert_eq!(harness.focus(), Some(editor));
    assert_eq!(
        active_query(&harness).as_deref(),
        Some(format!("?{before}").as_str()),
        "the editor owns printable '?' when focused"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}
