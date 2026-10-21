#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::{Surface, TableProApp};
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

const TAB_LIST: Id = Id::root("tablepro.tab-list");

fn connected() -> Harness<TableProApp> {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    Harness::new(app, Theme::junie(), 120, 40)
}

#[test]
fn choosing_a_query_tab_moves_focus_to_its_editor() {
    let mut harness = connected();
    let Some(first) = harness
        .app()
        .workbench
        .tabs()
        .first()
        .map(tablepro_ui::TabRecord::key)
    else {
        unreachable!("connected fixture has an initial tab")
    };
    harness.app_mut().workbench.new_query("SELECT 1");
    harness.draw();
    assert_ne!(harness.app().workbench.active_key(), Some(first));

    let _ = harness.ctrl('g');
    assert!(harness.layer_area(TAB_LIST).is_some());
    let _ = harness.key(KeyCode::Up);
    let _ = harness.key(KeyCode::Enter);

    assert_eq!(harness.app().workbench.active_key(), Some(first));
    assert_eq!(harness.focus(), Some(first.control("query")));
    assert_ne!(harness.app().surface(), Surface::TabListPicker);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn new_query_focus_belongs_to_the_visible_editor_at_adjacent_viewports() -> Result<(), &'static str>
{
    for (width, height) in [(119, 40), (120, 40), (121, 40), (120, 39), (120, 41)] {
        let mut app = TableProApp::new();
        assert!(app.connect(0));
        let mut harness = Harness::new(app, Theme::junie(), width, height);

        let _ = harness.ctrl('t');

        let Some(key) = harness.app().workbench.active_key() else {
            unreachable!("Ctrl+T creates an active query tab")
        };
        assert!(matches!(
            harness.app().workbench.active(),
            Some(tablepro_ui::Tab::Query(query)) if query.query.is_empty()
        ));
        let editor = key.control("query");
        assert_eq!(harness.focus(), Some(editor), "viewport {width}×{height}");
        let Some(entry) = harness.ring().entry(editor) else {
            return Err("query editor is registered");
        };
        assert!(
            entry.area.width > 0 && entry.area.height > 0,
            "focused query editor has positive resolved bounds at {width}×{height}: {:?}",
            entry.area
        );
        assert_eq!(harness.area_of(editor), Some(entry.area));
        assert!(
            harness.cursor().is_some(),
            "visible query editor owns its cursor at {width}×{height}"
        );
        assert!(
            harness.diagnostics().is_empty(),
            "viewport {width}×{height}: {:?}",
            harness.diagnostics()
        );
    }
    Ok(())
}

#[test]
fn inactive_query_editor_receives_focus_out_without_ending_its_edit_session() {
    for (connection, width, resized_width) in
        [(0, 119, 121), (0, 121, 119), (4, 119, 121), (4, 121, 119)]
    {
        let mut app = TableProApp::new();
        assert!(app.connect(connection));
        let mut harness = Harness::new(app, Theme::junie(), width, 40);

        let _ = harness.ctrl('t');
        let Some(first) = harness.app().workbench.active_key() else {
            unreachable!("Ctrl+T creates an active query tab")
        };
        let first_editor = first.control("query");
        assert_eq!(harness.focus(), Some(first_editor));
        assert_eq!(harness.diagnostics(), []);
        assert!(matches!(
            harness.app().workbench.active(),
            Some(tablepro_ui::Tab::Query(query)) if query.editor_state.editor().is_editing()
        ));
        let sql = "SELECT '東京 e\u{301}' AS saved_draft";
        let _ = harness.type_str(sql);
        let Some(tablepro_ui::Tab::Query(query)) = harness.app().workbench.active() else {
            unreachable!("first Ctrl+T creates a query tab")
        };
        assert_eq!(query.query, sql);
        assert_eq!(harness.diagnostics(), []);

        let _ = harness.resize(resized_width, 40);
        assert_eq!(harness.focus(), Some(first_editor));
        assert!(harness.cursor().is_some());
        assert_eq!(harness.diagnostics(), []);

        let _ = harness.ctrl('t');
        let Some(second) = harness.app().workbench.active_key() else {
            unreachable!("second Ctrl+T creates an active query tab")
        };
        let second_editor = second.control("query");
        assert_ne!(first, second);
        assert_eq!(harness.focus(), Some(second_editor));
        assert!(harness.cursor().is_some());
        assert_eq!(harness.diagnostics(), []);
        assert!(matches!(
            harness.app().workbench.tabs().iter().find(|tab| tab.key() == first),
            Some(record)
                if matches!(record.payload(), tablepro_ui::Tab::Query(query)
                    if query.editor_state.editor().is_editing()
                        && query.editor_state.editor().text() == sql
                        && query.query == sql)
        ));
    }
}

#[test]
fn active_query_editor_commits_when_focus_moves_to_its_results() {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    let outcome = app.run_query("SELECT id, email FROM customers LIMIT 3");
    assert!(matches!(
        outcome,
        tablepro_ui::QueryOutcome::Executed { .. }
    ));
    let Some(query_id) = app.query_id() else {
        unreachable!("running a query creates its editor")
    };
    let Some(results_id) = app.result_id() else {
        unreachable!("running a query creates its result grid")
    };
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);

    assert!(tab_navigation::tab_to(&mut harness, query_id));
    let _ = harness.key(KeyCode::End);
    let _ = harness.type_str(" -- draft");
    let Some(tablepro_ui::Tab::Query(query)) = harness.app().workbench.active() else {
        unreachable!("active tab remains a query")
    };
    assert!(query.editor_state.editor().is_editing());
    assert!(query.query.ends_with(" -- draft"));

    assert!(tab_navigation::tab_to(&mut harness, results_id));

    let Some(tablepro_ui::Tab::Query(query)) = harness.app().workbench.active() else {
        unreachable!("active tab remains a query")
    };
    assert!(!query.editor_state.editor().is_editing());
    assert!(query.query.ends_with(" -- draft"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn new_query_focus_follows_the_real_editor_after_resize_and_typing() -> Result<(), &'static str> {
    let mut app = TableProApp::new();
    assert!(app.connect(4));
    assert!(app.workbench.new_query("SELECT 1").is_some());
    let mut harness = Harness::new(app, Theme::junie(), 119, 40);
    let tabs_before = harness.app().workbench.tabs().len();

    let _ = harness.ctrl('t');
    let Some(key) = harness.app().workbench.active_key() else {
        unreachable!("Ctrl+T creates an active query tab")
    };
    let editor = key.control("query");
    assert_eq!(harness.focus(), Some(editor));
    assert_eq!(harness.app().workbench.tabs().len(), tabs_before + 1);
    let Some(editor_area) = harness
        .area_of(editor)
        .filter(|area| area.width > 0 && area.height > 0)
    else {
        return Err("active query editor has resolved bounds");
    };
    assert!(
        harness
            .state_of(editor)
            .contains(termrock::StateFlags::FOCUSED)
    );
    let Some(cursor) = harness.cursor() else {
        return Err("focused editor owns the terminal cursor");
    };
    assert!(
        cursor.x >= editor_area.x
            && cursor.x < editor_area.right()
            && cursor.y >= editor_area.y
            && cursor.y < editor_area.bottom(),
        "cursor {cursor:?} belongs to resolved editor area {editor_area:?}"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "after new tab: {:?}",
        harness.diagnostics()
    );

    let _ = harness.type_str("SELECT 1");
    let Some(tablepro_ui::Tab::Query(query)) = harness.app().workbench.active() else {
        unreachable!("active tab remains a query")
    };
    assert_eq!(query.query, "SELECT 1");
    assert_eq!(harness.focus(), Some(editor));
    assert!(
        harness.diagnostics().is_empty(),
        "after typing: {:?}",
        harness.diagnostics()
    );

    let _ = harness.resize(121, 40);
    assert_eq!(harness.focus(), Some(editor));
    assert!(
        harness
            .area_of(editor)
            .is_some_and(|area| area.width > 0 && area.height > 0)
    );
    assert!(harness.cursor().is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "after resize: {:?}",
        harness.diagnostics()
    );

    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn quick_switcher_escape_restores_explorer_footer_hints() {
    let mut harness = connected();
    harness.draw();

    let _ = harness.ctrl('o');
    assert!(harness.text().contains("Open Quickly"));
    let _ = harness.key(KeyCode::Esc);
    harness.draw();

    assert!(
        harness.text().contains("Ctrl+O Quick open"),
        "{}",
        harness.text()
    );
    assert!(!harness.text().contains("F6"), "{}", harness.text());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn zero_returns_from_table_grid_to_explorer_for_next_table() {
    let mut harness = connected();
    harness.draw();
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.text().contains("public › orders"),
        "{}",
        harness.text()
    );

    let _ = harness.key(KeyCode::Char('0'));
    assert_eq!(
        harness.focus(),
        Some(Id::root("tablepro.workbench.explorer.tree"))
    );
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);

    let Some(table) = harness.app().workbench.active_table() else {
        unreachable!("explorer activation creates a table tab")
    };
    assert_eq!(table.table.name, "order_items");
    assert!(
        harness.text().contains("public › order_items"),
        "{}",
        harness.text()
    );
    let _ = harness.ctrl('g');
    assert!(harness.text().contains("order_items"), "{}", harness.text());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn delete_closes_clean_tab_and_reopens_picker_with_empty_filter() {
    let mut harness = connected();
    let Some(removed) = harness.app_mut().workbench.new_query("SELECT 1") else {
        unreachable!("fixture has available tab identities")
    };
    harness.draw();

    let _ = harness.ctrl('g');
    let _ = harness.type_str("Query 2");
    assert!(harness.text().contains("Query 2"), "{}", harness.text());
    let _ = harness.key(KeyCode::Delete);

    assert!(
        !harness
            .app()
            .workbench
            .tabs()
            .iter()
            .any(|tab| tab.key() == removed)
    );
    assert!(harness.layer_area(TAB_LIST).is_some());
    assert!(harness.text().contains("Query 1"), "{}", harness.text());
    assert!(!harness.text().contains("Close tab with unsaved work?"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn delete_discards_dirty_query_tab_from_picker() {
    let mut harness = connected();
    let Some(query_key) = harness.app_mut().workbench.new_query("SELECT 1") else {
        unreachable!("fixture has available tab identities")
    };
    let Some(tablepro_ui::Tab::Query(query)) = harness.app_mut().workbench.tab_mut(query_key)
    else {
        unreachable!("created tab is a query")
    };
    query.saved_text = String::new();
    assert!(query.dirty());
    harness.draw();

    let _ = harness.ctrl('g');
    let _ = harness.type_str("Query 2");
    let _ = harness.key(KeyCode::Delete);

    assert!(harness.app().workbench.tab(query_key).is_none());
    assert!(harness.layer_area(TAB_LIST).is_some());
    assert!(!harness.text().contains("Close tab with unsaved work?"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn delete_discards_pending_table_edits_from_picker() {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    assert!(app.workbench.open_table("customers"));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let Some(table_key) = harness.app().workbench.active_key() else {
        unreachable!("connected fixture has a table tab")
    };
    let Some(tablepro_ui::Tab::Table(table)) = harness.app_mut().workbench.tab_mut(table_key)
    else {
        unreachable!("connected fixture opens a table")
    };
    assert!(table.result.model.insert_row().is_some());
    assert!(table.result.pending_total() > 0);
    harness.draw();

    let _ = harness.ctrl('g');
    let _ = harness.key(KeyCode::Delete);

    assert!(harness.app().workbench.tab(table_key).is_none());
    assert!(harness.layer_area(TAB_LIST).is_some());
    assert!(!harness.text().contains("Close tab with unsaved work?"));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn selecting_structure_tab_does_not_focus_a_painted_only_control() {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    assert!(app.workbench.open_table("customers"));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let Some(table_key) = harness.app().workbench.active_key() else {
        unreachable!("connected fixture has a table tab")
    };
    let Some(tablepro_ui::Tab::Table(table)) = harness.app_mut().workbench.tab_mut(table_key)
    else {
        unreachable!("connected fixture opens a table")
    };
    table.mode = tablepro_ui::tabs::TableMode::Structure;
    harness.draw();

    let _ = harness.ctrl('g');
    let _ = harness.key(KeyCode::Enter);

    assert_eq!(harness.app().workbench.active_key(), Some(table_key));
    assert_ne!(harness.focus(), Some(table_key.control("structure")));
    assert_ne!(harness.focus(), Some(table_key.control("data")));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn query_tab_detail_uses_its_last_execution_status() {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    assert!(matches!(
        app.run_query("SELECT * FROM customers LIMIT 1"),
        tablepro_ui::QueryOutcome::Executed { .. }
    ));
    let Some(query_key) = app.workbench.active_key() else {
        unreachable!("query fixture has active tab")
    };
    let Some(query) = app.workbench.tab(query_key) else {
        unreachable!("active query exists")
    };
    let tablepro_ui::Tab::Query(query) = query else {
        unreachable!("query fixture")
    };
    let Some(status) = query.last_status.as_ref() else {
        unreachable!("executed query has a status")
    };
    assert!(!status.is_error);
    assert!(status.summary.starts_with("ok · "));
    assert!(status.duration_ms < 60_000);
    assert!(matches!(
        app.run_query("SELECT * FROM missing_table"),
        tablepro_ui::QueryOutcome::Rejected { .. }
    ));
    let Some(query) = app.workbench.tab(query_key) else {
        unreachable!("active query exists")
    };
    let tablepro_ui::Tab::Query(query) = query else {
        unreachable!("query fixture")
    };
    let Some(status) = query.last_status.as_ref() else {
        unreachable!("failed query has a status")
    };
    assert!(status.is_error);
    assert_eq!(status.summary, "error");

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let _ = harness.ctrl('g');
    assert!(harness.text().contains("error"), "{}", harness.text());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
