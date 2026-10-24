#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::{Surface, TableProApp};
use termrock::{Id, KeyCode, MouseKind, Theme};
use termrock_test_support::Harness;

const TAB_LIST: Id = Id::root("tablepro.tab-list");
const QUIT_DIALOG: Id = Id::root("tablepro.quit-dialog");

fn connected() -> Harness<TableProApp> {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    Harness::new(app, Theme::junie(), 120, 40)
}

type AppHarness = Harness<TableProApp>;

struct DirtyQueryPicker {
    harness: AppHarness,
    neighbor_key: tablepro_ui::TabKey,
    dirty_key: tablepro_ui::TabKey,
    draft: String,
}

fn live_query_draft(h: &AppHarness, key: tablepro_ui::TabKey) -> Option<String> {
    match h.app().workbench.tab(key)? {
        tablepro_ui::Tab::Query(query) => Some(query.editor_state.editor().text().to_owned()),
        tablepro_ui::Tab::Table(_) | tablepro_ui::Tab::History(_) => None,
    }
}

fn dirty_query_picker() -> Result<DirtyQueryPicker, &'static str> {
    let mut app = TableProApp::new();
    if !app.connect(0) {
        return Err("production connection");
    }
    let Some(neighbor_key) = app.workbench.active_key() else {
        return Err("initial neighbor query");
    };
    let Some(dirty_key) = app.workbench.new_query("SELECT 'saved'") else {
        return Err("dirty query fixture");
    };
    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    if !tab_navigation::tab_to(&mut h, dirty_key.control("query")) {
        return Err("focus the real query editor");
    }
    let _ = h.type_str(" -- live unsaved edit");
    let Some(tablepro_ui::Tab::Query(query)) = h.app().workbench.tab(dirty_key) else {
        return Err("dirty target remains a query");
    };
    if !query.dirty() {
        return Err("keyboard-edited query is actually dirty");
    }
    let draft = query.editor_state.editor().text().to_owned();
    let _ = h.ctrl('g');
    let _ = h.type_str("Query 2");
    if h.focus() != Some(TAB_LIST) {
        return Err("tab picker owns focus after filtering");
    }
    if h.area_of_part(
        TAB_LIST,
        termrock::PartRef::item(termrock::Part::ROW, termrock::ItemKey::num(dirty_key.get())),
    )
    .is_none()
    {
        return Err("filtered dirty row is registered");
    }
    Ok(DirtyQueryPicker {
        harness: h,
        neighbor_key,
        dirty_key,
        draft,
    })
}

fn assert_close_confirmation(h: &AppHarness) -> Result<(), &'static str> {
    if h.layer_area(QUIT_DIALOG).is_none() {
        return Err("dirty delete opens the existing confirmation dialog");
    }
    if !h.text().contains("Close tab with unsaved work?")
        || !h.text().contains("Close anyway")
        || !h.text().contains("Cancel")
    {
        return Err("confirmation exposes Cancel and explicit Close anyway");
    }
    Ok(())
}

fn click_close_anyway(h: &mut AppHarness) -> Result<(), &'static str> {
    let Some((x, y)) = h.find("Close anyway") else {
        return Err("visible Close anyway action");
    };
    let _ = h.click(x, y);
    Ok(())
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
fn tab_list_dismissal_preserves_the_active_tab_and_blocks_underlay_input()
-> Result<(), &'static str> {
    for outside_dismissal in [false, true] {
        let mut app = TableProApp::new();
        assert!(app.connect(0));
        let Some(first) = app.workbench.active_key() else {
            return Err("initial query tab");
        };
        let Some(active) = app.workbench.new_query("SELECT 'active tab draft'") else {
            return Err("second query tab");
        };
        let Some(other) = app.workbench.new_query("SELECT 'other tab draft'") else {
            return Err("third query tab");
        };
        assert_ne!(first, active);
        assert_ne!(active, other);
        assert!(app.workbench.activate(active));

        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        let active_editor = active.control("query");
        assert!(tab_navigation::tab_to(&mut harness, active_editor));
        let Some(first_tab) = harness.find("Query 1") else {
            return Err("first tab in strip");
        };
        let original_count = harness.app().workbench.tabs().len();
        let original_query = match harness.app().workbench.active() {
            Some(tablepro_ui::Tab::Query(query)) => query.query.clone(),
            _ => return Err("active tab is a query"),
        };
        let keys = harness
            .app()
            .workbench
            .tabs()
            .iter()
            .map(tablepro_ui::TabRecord::key)
            .collect::<Vec<_>>();

        let _ = harness.ctrl('g');
        assert!(harness.layer_area(TAB_LIST).is_some());
        assert!(harness.text().contains("Open tabs"));
        let _ = harness.key(KeyCode::Down);
        assert!(harness.layer_area(TAB_LIST).is_some());

        let _ = harness.ctrl('t');
        assert!(
            harness.layer_area(TAB_LIST).is_some(),
            "picker remains open"
        );
        assert_eq!(harness.app().workbench.tabs().len(), original_count);
        assert_eq!(harness.app().workbench.active_key(), Some(active));

        if outside_dismissal {
            let Some(layer) = harness.layer_area(TAB_LIST) else {
                return Err("tab-list layer");
            };
            assert!(
                first_tab.0 < layer.x
                    || first_tab.0 >= layer.right()
                    || first_tab.1 < layer.y
                    || first_tab.1 >= layer.bottom(),
                "first tab is outside the picker layer: {layer:?}"
            );
            let _ = harness.mouse(MouseKind::Down, first_tab.0, first_tab.1);
            let _ = harness.mouse(MouseKind::Up, first_tab.0, first_tab.1);
        } else {
            let _ = harness.key(KeyCode::Esc);
        }

        assert!(harness.layer_area(TAB_LIST).is_none());
        assert_eq!(harness.app().workbench.tabs().len(), original_count);
        assert_eq!(harness.app().workbench.active_key(), Some(active));
        assert_eq!(harness.focus(), Some(active_editor));
        assert!(matches!(
            harness.app().workbench.active(),
            Some(tablepro_ui::Tab::Query(query)) if query.query == original_query
        ));
        assert!(
            keys.iter()
                .all(|key| harness.app().workbench.tab(*key).is_some()),
            "dismissal preserves every stable tab key"
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    Ok(())
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
fn delete_dirty_query_cancel_preserves_picker_and_live_draft() -> Result<(), &'static str> {
    use termrock::PartRef;

    let DirtyQueryPicker {
        harness: mut h,
        neighbor_key,
        dirty_key,
        draft,
    } = dirty_query_picker()?;
    let neighbor_draft = live_query_draft(&h, neighbor_key)
        .ok_or("neighbor query draft exists before confirmation")?;
    let _ = h.key(KeyCode::Delete);
    assert_close_confirmation(&h)?;
    assert!(h.layer_area(TAB_LIST).is_some());
    assert_eq!(h.app().workbench.active_key(), Some(dirty_key));
    assert_eq!(
        live_query_draft(&h, dirty_key).as_deref(),
        Some(draft.as_str())
    );
    assert!(matches!(
        h.app().workbench.tab(dirty_key),
        Some(tablepro_ui::Tab::Query(query)) if query.dirty()
    ));
    assert!(h.app().workbench.tab(neighbor_key).is_some());
    assert_eq!(
        live_query_draft(&h, neighbor_key).as_deref(),
        Some(neighbor_draft.as_str())
    );

    let _ = h.key(KeyCode::Esc);
    assert!(h.layer_area(QUIT_DIALOG).is_none());
    assert!(h.layer_area(TAB_LIST).is_some());
    assert_eq!(h.focus(), Some(TAB_LIST));
    assert_eq!(h.app().workbench.active_key(), Some(dirty_key));
    assert_eq!(
        live_query_draft(&h, dirty_key).as_deref(),
        Some(draft.as_str())
    );
    assert!(h.text().contains("Query 2"), "{}", h.text());
    assert!(
        h.area_of_part(
            TAB_LIST,
            PartRef::item(termrock::Part::ROW, termrock::ItemKey::num(dirty_key.get())),
        )
        .is_some()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    // The preserved filter/cursor still chooses the dirty query after Cancel.
    let _ = h.key(KeyCode::Enter);
    assert!(h.layer_area(TAB_LIST).is_none());
    assert_eq!(h.app().workbench.active_key(), Some(dirty_key));
    assert_eq!(h.focus(), Some(dirty_key.control("query")));
    assert_eq!(
        live_query_draft(&h, dirty_key).as_deref(),
        Some(draft.as_str())
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    Ok(())
}

#[test]
fn delete_dirty_query_confirms_exact_target_and_closes_picker_owner() -> Result<(), &'static str> {
    let DirtyQueryPicker {
        harness: mut h,
        neighbor_key,
        dirty_key,
        draft,
    } = dirty_query_picker()?;
    let neighbor_draft = live_query_draft(&h, neighbor_key)
        .ok_or("neighbor query draft exists before confirmation")?;
    let _ = h.key(KeyCode::Delete);
    assert_close_confirmation(&h)?;
    assert_eq!(
        live_query_draft(&h, dirty_key).as_deref(),
        Some(draft.as_str())
    );
    click_close_anyway(&mut h)?;

    assert!(h.layer_area(QUIT_DIALOG).is_none());
    assert!(h.layer_area(TAB_LIST).is_none());
    assert!(h.app().workbench.tab(dirty_key).is_none());
    assert!(h.app().workbench.tab(neighbor_key).is_some());
    assert_eq!(h.app().workbench.tabs().len(), 1);
    assert_eq!(h.app().workbench.active_key(), Some(neighbor_key));
    assert_eq!(
        live_query_draft(&h, neighbor_key).as_deref(),
        Some(neighbor_draft.as_str())
    );
    assert_eq!(h.focus(), Some(neighbor_key.control("query")));

    // Enter after confirmation is delivered to live content, never a stale row.
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().workbench.tab(dirty_key).is_none());
    assert_eq!(h.app().workbench.tabs().len(), 1);
    assert_eq!(h.app().workbench.active_key(), Some(neighbor_key));
    let _ = h.ctrl('g');
    assert!(h.layer_area(TAB_LIST).is_some());
    assert!(h.text().contains("Query 1"), "{}", h.text());
    assert!(!h.text().contains("Query 2"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    let _ = h.key(KeyCode::Esc);
    assert!(h.layer_area(TAB_LIST).is_none());
    assert_eq!(h.focus(), Some(neighbor_key.control("query")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    Ok(())
}

#[test]
fn delete_pending_table_edits_cancel_preserves_rows_and_picker_filter() -> Result<(), &'static str>
{
    use termrock::{ItemKey, Part, PartRef};

    let mut app = TableProApp::new();
    if !app.connect(0) || !app.workbench.open_table("customers") {
        return Err("production table fixture");
    }
    let Some(table_key) = app.workbench.active_key() else {
        return Err("dirty table key");
    };
    let Some(tablepro_ui::Tab::Table(table)) = app.workbench.tab_mut(table_key) else {
        return Err("customers is a table tab");
    };
    if table.result.model.insert_row().is_none() || table.result.pending_total() == 0 {
        return Err("table has an actual pending inserted row");
    }
    let pending_before = table.result.pending_total();
    let rows_before = table.result.model.row_count();
    let Some(neighbor_key) = app.workbench.new_query("SELECT 'neighbor'") else {
        return Err("neighbor query fixture");
    };
    let neighbor_draft = match app.workbench.tab(neighbor_key) {
        Some(tablepro_ui::Tab::Query(query)) => query.editor_state.editor().text().to_owned(),
        _ => return Err("neighbor query exists"),
    };

    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.ctrl('g');
    let _ = h.type_str("customers");
    let row = PartRef::item(Part::ROW, ItemKey::num(table_key.get()));
    if h.area_of_part(TAB_LIST, row).is_none() {
        return Err("filtered pending table row is registered");
    }
    let _ = h.key(KeyCode::Delete);
    assert_close_confirmation(&h)?;
    let Some(tablepro_ui::Tab::Table(table)) = h.app().workbench.tab(table_key) else {
        return Err("pending table remains before confirmation");
    };
    assert!(
        h.app()
            .workbench
            .tab(table_key)
            .is_some_and(tablepro_ui::Tab::dirty)
    );
    assert_eq!(table.result.pending_total(), pending_before);
    assert_eq!(table.result.model.row_count(), rows_before);
    assert_eq!(h.app().workbench.active_key(), Some(neighbor_key));
    assert_eq!(
        live_query_draft(&h, neighbor_key).as_deref(),
        Some(neighbor_draft.as_str())
    );

    let _ = h.key(KeyCode::Esc);
    assert!(h.layer_area(QUIT_DIALOG).is_none());
    assert!(h.layer_area(TAB_LIST).is_some());
    assert_eq!(h.focus(), Some(TAB_LIST));
    assert!(h.text().contains("customers"), "{}", h.text());
    assert!(h.area_of_part(TAB_LIST, row).is_some());
    let Some(tablepro_ui::Tab::Table(table)) = h.app().workbench.tab(table_key) else {
        return Err("pending table remains after Cancel");
    };
    assert!(
        h.app()
            .workbench
            .tab(table_key)
            .is_some_and(tablepro_ui::Tab::dirty)
    );
    assert_eq!(table.result.pending_total(), pending_before);
    assert_eq!(table.result.model.row_count(), rows_before);
    assert_eq!(h.app().workbench.active_key(), Some(neighbor_key));
    assert_eq!(
        live_query_draft(&h, neighbor_key).as_deref(),
        Some(neighbor_draft.as_str())
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    Ok(())
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

fn assert_tab_list_margin_geometry(
    h: &Harness<TableProApp>,
    expected: (u16, u16, u16, u16),
) -> Result<(), &'static str> {
    use termrock::{Part, PartRef};

    let Some(area) = h.layer_area(TAB_LIST) else {
        return Err("tab-list layer remains open");
    };
    assert_eq!((area.x, area.y, area.width, area.height), expected);
    if h.focus() != Some(TAB_LIST) || h.ring().entry(TAB_LIST).is_none() {
        return Err("tab-list query owner remains focused and registered");
    }
    if h.area_of_part(TAB_LIST, PartRef::of(Part::BORDER))
        .is_none()
        || h.find("Filter tabs").is_none()
        || h.find("Delete Close tab").is_none()
    {
        return Err("tab-list border, query, and hint remain visible");
    }
    Ok(())
}

fn assert_last_tab_row_visible(
    h: &Harness<TableProApp>,
    last_key: termrock::ItemKey,
) -> Result<(), &'static str> {
    use termrock::{Part, PartRef};

    if h.area_of_part(TAB_LIST, PartRef::item(Part::ROW, last_key))
        .is_none()
    {
        return Err("selected tail tab row remains registered after resize");
    }
    if !h.text().contains("Query 12") {
        return Err("selected tail tab remains visible");
    }
    Ok(())
}

#[test]
fn tab_list_margin_tracks_selection_through_short_resize_and_escape() -> Result<(), &'static str> {
    use termrock::{ItemKey, Part, PartRef};

    let mut app = TableProApp::new();
    if !app.connect(0) {
        return Err("production connection");
    }
    let Some(first_key) = app.workbench.active_key() else {
        return Err("initial query tab");
    };
    let mut last_key = first_key;
    for index in 1..12 {
        let Some(key) = app.workbench.new_query(format!("SELECT {index}")) else {
            return Err("twelve-tab geometry fixture");
        };
        last_key = key;
    }
    if !app.workbench.activate(first_key) {
        return Err("activate first query before opening tab list");
    }

    let mut h = Harness::new(app, Theme::junie(), 120, 40);
    let _ = h.ctrl('g');
    assert_tab_list_margin_geometry(&h, (28, 10, 64, 19))?;
    if h.area_of_part(
        TAB_LIST,
        PartRef::item(Part::ROW, ItemKey::num(last_key.get())),
    )
    .is_none()
    {
        return Err("tab-list tail row is registered before resize");
    }
    let picker_focus = h.focus();
    for _ in 0..11 {
        let _ = h.key(KeyCode::Down);
    }

    let _ = h.resize(120, 20);
    assert_tab_list_margin_geometry(&h, (28, 1, 64, 18))?;
    assert_last_tab_row_visible(&h, ItemKey::num(last_key.get()))?;
    assert_eq!(h.focus(), picker_focus);

    let _ = h.resize(120, 21);
    assert_tab_list_margin_geometry(&h, (28, 1, 64, 19))?;
    assert_last_tab_row_visible(&h, ItemKey::num(last_key.get()))?;
    assert_eq!(h.focus(), picker_focus);
    assert_eq!(h.app().workbench.active_key(), Some(first_key));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let _ = h.resize(120, 24);
    assert_tab_list_margin_geometry(&h, (28, 2, 64, 19))?;
    let _ = h.resize(120, 40);
    assert_tab_list_margin_geometry(&h, (28, 10, 64, 19))?;
    assert_eq!(h.focus(), picker_focus);
    let _ = h.key(KeyCode::Enter);
    assert!(h.layer_area(TAB_LIST).is_none());
    assert_eq!(h.app().workbench.active_key(), Some(last_key));
    assert_eq!(h.focus(), Some(last_key.control("query")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let _ = h.ctrl('g');
    let _ = h.resize(120, 20);
    assert_tab_list_margin_geometry(&h, (28, 1, 64, 18))?;
    let _ = h.key(KeyCode::Esc);
    assert!(h.layer_area(TAB_LIST).is_none());
    assert_eq!(h.focus(), Some(last_key.control("query")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    Ok(())
}

#[test]
fn tab_list_preserves_query_and_selection_across_minimum_width_resize() -> Result<(), &'static str>
{
    use termrock::{ItemKey, Part, PartRef};

    let mut app = TableProApp::new();
    if !app.connect(0) {
        return Err("production connection");
    }
    let Some(active_before) = app.workbench.active_key() else {
        return Err("initial active tab");
    };
    let mut last_key = active_before;
    for index in 1..12 {
        let Some(key) = app.workbench.new_query(format!("SELECT {index}")) else {
            return Err("twelve-tab fixture");
        };
        last_key = key;
    }
    if !app.workbench.activate(active_before) {
        return Err("activate the initial tab before opening the list");
    }

    let mut h = Harness::new(app, Theme::junie(), 72, 40);
    let _ = h.ctrl('g');
    let Some(narrow) = h.layer_area(TAB_LIST) else {
        return Err("Ctrl+G opens at the narrow viewport");
    };
    assert_eq!(
        (narrow.x, narrow.y, narrow.width, narrow.height),
        (4, 10, 64, 19)
    );
    assert_eq!(h.focus(), Some(TAB_LIST));
    assert!(h.ring().entry(TAB_LIST).is_some());

    let _ = h.type_str("Query 12");
    let last_item = ItemKey::num(last_key.get());
    assert!(h.text().contains("Query 12"), "{}", h.text());
    assert!(h.cursor().is_some(), "tab-list query cursor is visible");
    assert!(
        h.area_of_part(TAB_LIST, PartRef::item(Part::ROW, last_item))
            .is_some()
    );
    assert_eq!(h.app().workbench.active_key(), Some(active_before));

    let _ = h.resize(120, 40);
    let Some(wide) = h.layer_area(TAB_LIST) else {
        return Err("tab list remains open after a real Resize input");
    };
    assert_eq!((wide.x, wide.y, wide.width, wide.height), (28, 10, 64, 19));
    assert_eq!(h.focus(), Some(TAB_LIST));
    assert!(h.ring().entry(TAB_LIST).is_some());
    assert!(
        h.cursor().is_some(),
        "tab-list query cursor remains visible after reflow"
    );
    let _ = h.type_str("x");
    assert!(
        h.text().contains("Query 12x"),
        "the caret remains after the query: {}",
        h.text()
    );
    let _ = h.key(KeyCode::Backspace);
    assert!(h.text().contains("Query 12"), "{}", h.text());
    assert!(
        h.area_of_part(TAB_LIST, PartRef::item(Part::ROW, last_item))
            .is_some()
    );
    assert_eq!(h.app().workbench.active_key(), Some(active_before));

    let _ = h.key(KeyCode::Enter);
    assert!(h.layer_area(TAB_LIST).is_none());
    assert_eq!(h.app().workbench.active_key(), Some(last_key));
    assert_eq!(h.focus(), Some(last_key.control("query")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    Ok(())
}
