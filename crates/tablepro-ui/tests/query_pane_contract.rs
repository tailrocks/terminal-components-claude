#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use std::time::Duration;

use tablepro_ui::{QueryOutcome, Screen, Surface, Tab, TabKey, TableProApp, Workbench};
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

const TAB_STRIP: Id = Id::root("tablepro.workbench.tab-strip");

#[derive(Clone, Copy)]
enum PaneContent {
    Results,
    EmptyResults,
    Error,
    Plan,
    Idle,
}

fn workbench() -> Harness<TableProApp> {
    workbench_at(120, 40)
}

fn workbench_at(width: u16, height: u16) -> Harness<TableProApp> {
    let mut app = TableProApp::new_with_query_duration(Duration::from_millis(42));
    app.screen = Screen::Workbench;
    app.surface = Surface::WorkbenchDefault;
    Harness::new(app, Theme::junie(), width, height)
}

fn activate_tab(harness: &mut Harness<TableProApp>, target: TabKey) {
    assert!(tab_navigation::tab_to(harness, TAB_STRIP));
    for _ in 0..harness.app().workbench.tabs().len() {
        if harness.app().workbench.active_key() == Some(target) {
            break;
        }
        let _ = harness.key(KeyCode::Left);
    }
    assert_eq!(harness.app().workbench.active_key(), Some(target));
}

fn set_pane_content(
    harness: &mut Harness<TableProApp>,
    key: TabKey,
    content: PaneContent,
) -> Result<(), std::io::Error> {
    match content {
        PaneContent::Results => {}
        PaneContent::EmptyResults => assert!(matches!(
            harness
                .app_mut()
                .run_query("SELECT id, currency FROM orders WHERE id = 'missing'"),
            QueryOutcome::Executed { rows: 0, .. }
        )),
        PaneContent::Error => {
            let Some(Tab::Query(query)) = harness.app_mut().workbench.tab_mut(key) else {
                return Err(std::io::Error::other("active tab remains a query"));
            };
            query.result = None;
            query.plan = None;
            query.error = Some("pane error marker".to_owned());
        }
        PaneContent::Plan => {
            let catalog = harness.app().workbench.catalog.clone();
            let Some(Tab::Query(query)) = harness.app_mut().workbench.tab_mut(key) else {
                return Err(std::io::Error::other("active tab remains a query"));
            };
            query.result = None;
            query.error = None;
            query.plan = None;
            query
                .explain(&catalog)
                .map_err(|_| std::io::Error::other("query plan is available"))?;
        }
        PaneContent::Idle => {
            let Some(Tab::Query(query)) = harness.app_mut().workbench.tab_mut(key) else {
                return Err(std::io::Error::other("active tab remains a query"));
            };
            query.result = None;
            query.error = None;
            query.plan = None;
        }
    }
    Ok(())
}

fn assert_pane_content(
    harness: &Harness<TableProApp>,
    key: TabKey,
    result: Id,
    content: PaneContent,
) -> Result<(), std::io::Error> {
    match content {
        PaneContent::Results => assert!(harness.area_of(result).is_some()),
        PaneContent::EmptyResults => {
            assert!(harness.area_of(result).is_some());
            let Some(Tab::Query(query)) = harness.app().workbench.tab(key) else {
                return Err(std::io::Error::other("active tab remains a query"));
            };
            assert_eq!(
                query.result.as_ref().map(|grid| grid.model.row_count()),
                Some(0)
            );
            assert!(harness.text().contains("SELECT orders (0)"));
        }
        PaneContent::Error => {
            assert_eq!(harness.area_of(result), None);
            assert!(harness.text().contains("pane error marker"));
        }
        PaneContent::Plan => {
            assert_eq!(harness.area_of(result), None);
            let Some(Tab::Query(query)) = harness.app().workbench.tab(key) else {
                return Err(std::io::Error::other("active tab remains a query"));
            };
            let operation = query
                .plan
                .as_ref()
                .ok_or_else(|| std::io::Error::other("plan is present"))?
                .op
                .as_str();
            assert!(harness.text().contains(operation), "{operation}");
        }
        PaneContent::Idle => {
            assert_eq!(harness.area_of(result), None);
            assert!(harness.text().contains("No results yet"));
        }
    }
    Ok(())
}

#[test]
fn z_from_the_result_grid_maximizes_only_that_query_pane() -> Result<(), std::io::Error> {
    let mut harness = workbench();
    let Some(key) = harness.app().workbench.active_key() else {
        return Err(std::io::Error::other(
            "the workbench opens an initial query tab",
        ));
    };
    let editor = key.control("query");
    let result = key.control("results");
    assert!(
        harness.area_of(result).is_some(),
        "the query has a result grid"
    );
    assert!(
        tab_navigation::tab_to(&mut harness, result),
        "the result grid receives focus"
    );
    assert_eq!(harness.focus(), Some(result));

    let normal_grid = harness
        .area_of(result)
        .ok_or_else(|| std::io::Error::other("visible result grid"))?;
    let query_before = harness.app().query().to_owned();
    let rows_before = match harness.app().workbench.tab(key) {
        Some(Tab::Query(query)) => query.result.as_ref().map(|grid| grid.model.row_count()),
        _ => None,
    };
    let history_before = harness.app().workbench.history.entries.len();

    let _ = harness.key(KeyCode::Char('z'));

    let maximized_grid = harness
        .area_of(result)
        .ok_or_else(|| std::io::Error::other("the focused grid stays visible"))?;
    assert!(
        maximized_grid.width > normal_grid.width,
        "focused query results use the workbench width: {normal_grid:?} -> {maximized_grid:?}"
    );
    assert_eq!(
        harness.area_of(editor),
        None,
        "the hidden editor has no hit area"
    );
    assert!(harness.app().workbench.maximized);
    assert_eq!(harness.app().query(), query_before);
    assert_eq!(
        match harness.app().workbench.tab(key) {
            Some(Tab::Query(query)) => query.result.as_ref().map(|grid| grid.model.row_count()),
            _ => None,
        },
        rows_before
    );
    assert_eq!(
        harness.app().workbench.history.entries.len(),
        history_before
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.app().workbench.maximized);
    assert_eq!(harness.focus(), Some(result));
    assert!(harness.area_of(editor).is_some());
    assert!(harness.area_of(result).is_some());

    let _ = harness.key(KeyCode::Char('z'));
    assert!(harness.app().workbench.maximized);
    assert_eq!(harness.area_of(editor), None);
    assert!(harness.area_of(result).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn z_from_the_editor_maximizes_only_that_query_pane_and_editing_z_is_text()
-> Result<(), std::io::Error> {
    let mut harness = workbench();
    let Some(key) = harness.app().workbench.active_key() else {
        return Err(std::io::Error::other(
            "the workbench opens an initial query tab",
        ));
    };
    let editor = key.control("query");
    let result = key.control("results");
    assert!(
        tab_navigation::tab_to(&mut harness, editor),
        "the query editor receives focus"
    );
    assert_eq!(harness.focus(), Some(editor));

    let query_before = harness.app().query().to_owned();
    let _ = harness.key(KeyCode::Char('z'));
    assert_eq!(harness.app().query().len(), query_before.len() + 1);
    assert!(
        !harness.app().workbench.maximized,
        "editing keeps z as text"
    );

    let _ = harness.key(KeyCode::Esc);
    assert_eq!(harness.focus(), Some(editor));
    let Some(Tab::Query(query)) = harness.app().workbench.tab(key) else {
        return Err(std::io::Error::other("the active tab remains a query"));
    };
    assert!(!query.editor_state.editor().is_editing());

    let _ = harness.key(KeyCode::Char('z'));

    assert!(harness.app().workbench.maximized);
    assert!(
        harness.area_of(editor).is_some(),
        "the focused editor stays visible"
    );
    assert_eq!(
        harness.area_of(result),
        None,
        "the hidden grid has no hit area"
    );
    assert_eq!(harness.app().query().len(), query_before.len() + 1);
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.app().workbench.maximized);
    assert_eq!(harness.focus(), Some(editor));
    assert!(harness.area_of(editor).is_some());
    assert!(harness.area_of(result).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn z_from_the_explorer_keeps_the_whole_query_workbench_path() -> Result<(), std::io::Error> {
    let mut harness = workbench();
    let Some(key) = harness.app().workbench.active_key() else {
        return Err(std::io::Error::other(
            "the workbench opens an initial query tab",
        ));
    };
    let editor = key.control("query");
    let result = key.control("results");
    let explorer = Id::root("tablepro.workbench.explorer.tree");
    assert!(tab_navigation::tab_to(&mut harness, explorer));

    let _ = harness.key(KeyCode::Char('z'));

    assert!(harness.app().workbench.maximized);
    assert!(harness.area_of(editor).is_some());
    assert!(harness.area_of(result).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn zero_restores_the_explorer_and_both_normal_query_controls() -> Result<(), std::io::Error> {
    let mut harness = workbench();
    let Some(key) = harness.app().workbench.active_key() else {
        return Err(std::io::Error::other(
            "the workbench opens an initial query tab",
        ));
    };
    let editor = key.control("query");
    let result = key.control("results");
    assert!(tab_navigation::tab_to(&mut harness, result));
    let _ = harness.key(KeyCode::Char('z'));
    assert_eq!(harness.area_of(editor), None);

    let _ = harness.key(KeyCode::Char('0'));

    assert!(!harness.app().workbench.maximized);
    assert_eq!(
        harness.focus(),
        Some(Id::root("tablepro.workbench.explorer.tree"))
    );
    assert!(harness.area_of(editor).is_some());
    assert!(harness.area_of(result).is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn help_names_focused_query_zoom_and_fits_at_minimum_sizes() -> Result<(), std::io::Error> {
    let mut readable = workbench_at(120, 40);
    let _ = readable.key(KeyCode::F(1));
    let help_area = readable
        .layer_area(Id::root("tablepro.help"))
        .ok_or_else(|| std::io::Error::other("help card is visible"))?;
    let (label_x, label_y) = readable
        .find("Zoom workbench; focused")
        .ok_or_else(|| std::io::Error::other("complete SHELL zoom description is visible"))?;
    let (continuation_x, continuation_y) = readable
        .find("query pane")
        .ok_or_else(|| std::io::Error::other("wrapped SHELL zoom description is visible"))?;
    assert!(label_x >= help_area.x && label_x < help_area.right());
    assert!(continuation_x >= help_area.x && continuation_x < help_area.right());
    assert!(label_y > help_area.y && label_y.saturating_add(1) < help_area.bottom());
    assert_eq!(continuation_y, label_y.saturating_add(1));
    assert!(readable.row(label_y).contains("Zoom workbench; focused"));
    assert!(readable.row(continuation_y).contains("query pane"));
    let (grid_x, grid_y) = readable
        .find("Zoom the focused pane")
        .ok_or_else(|| std::io::Error::other("GRID description stays unchanged"))?;
    assert!(grid_x >= help_area.x && grid_x < help_area.right());
    assert!(grid_y > help_area.y && grid_y < help_area.bottom());

    for (width, height) in [(72, 20), (80, 24)] {
        let mut harness = workbench_at(width, height);
        let _ = harness.key(KeyCode::F(1));

        assert!(harness.is_open(Id::root("tablepro.help")));
        let area = harness
            .layer_area(Id::root("tablepro.help"))
            .ok_or_else(|| std::io::Error::other("help card is visible"))?;
        assert!(area.x.saturating_add(area.width) <= width);
        assert!(area.y.saturating_add(area.height) <= height);
        assert!(harness.text().contains("Shell and workbench"));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    Ok(())
}

#[test]
fn focused_query_pane_choice_is_stable_per_tab_and_workbench() -> Result<(), std::io::Error> {
    let mut harness = workbench();
    let first = harness
        .app()
        .workbench
        .active_key()
        .ok_or_else(|| std::io::Error::other("initial query tab"))?;
    let first_editor = first.control("query");
    let first_grid = first.control("results");
    assert!(tab_navigation::tab_to(&mut harness, first_grid));
    let _ = harness.key(KeyCode::Char('z'));
    assert!(harness.app().workbench.maximized);
    assert_eq!(harness.area_of(first_editor), None);

    let second = harness
        .app_mut()
        .workbench
        .new_query("SELECT 1")
        .ok_or_else(|| std::io::Error::other("neighbor query identity"))?;
    let second_editor = second.control("query");
    let second_grid = second.control("results");
    harness.draw();
    assert!(harness.area_of(second_editor).is_some());
    assert_eq!(harness.area_of(second_grid), None);

    harness.app_mut().workbench.maximized = false;
    harness.draw();
    assert!(tab_navigation::tab_to(&mut harness, second_editor));
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.app().workbench.maximized);
    let _ = harness.key(KeyCode::Char('z'));
    assert!(harness.app().workbench.maximized);
    assert!(harness.area_of(second_editor).is_some());
    assert_eq!(harness.area_of(second_grid), None);

    activate_tab(&mut harness, first);
    assert_eq!(harness.area_of(first_editor), None);
    assert!(harness.area_of(first_grid).is_some());

    assert!(harness.app_mut().workbench.reorder_tabs(&[second, first]));
    harness.draw();
    assert!(harness.area_of(first_grid).is_some());
    activate_tab(&mut harness, second);
    assert!(harness.area_of(second_editor).is_some());
    assert_eq!(harness.area_of(second_grid), None);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn replacing_the_workbench_clears_old_query_pane_modes() -> Result<(), std::io::Error> {
    let mut harness = workbench();
    let old_key = harness
        .app()
        .workbench
        .active_key()
        .ok_or_else(|| std::io::Error::other("initial query tab"))?;
    let old_grid = old_key.control("results");
    assert!(tab_navigation::tab_to(&mut harness, old_grid));
    let _ = harness.key(KeyCode::Char('z'));
    assert_eq!(harness.area_of(old_key.control("query")), None);
    assert!(tab_navigation::tab_to(&mut harness, TAB_STRIP));

    let connection = harness.app().workbench.connection.clone();
    let catalog = harness.app().workbench.catalog.clone();
    let mut replacement = Workbench::new(connection, catalog);
    let new_key = replacement
        .new_query("SELECT 2")
        .ok_or_else(|| std::io::Error::other("replacement query identity"))?;
    replacement.maximized = true;
    harness.app_mut().workbench = replacement;
    harness.draw();

    assert!(harness.app().workbench.maximized);
    assert!(harness.area_of(new_key.control("query")).is_some());
    assert_eq!(harness.area_of(new_key.control("results")), None);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    Ok(())
}

#[test]
fn query_result_pane_reflows_and_stays_clipped_across_resizes() -> Result<(), std::io::Error> {
    let mut harness = workbench();
    let key = harness
        .app()
        .workbench
        .active_key()
        .ok_or_else(|| std::io::Error::other("initial query tab"))?;
    let editor = key.control("query");
    let result = key.control("results");
    assert!(tab_navigation::tab_to(&mut harness, result));
    let normal_result = harness
        .area_of(result)
        .ok_or_else(|| std::io::Error::other("normal result viewport"))?;
    let _ = harness.key(KeyCode::Char('z'));
    let first_maximized = harness
        .area_of(result)
        .ok_or_else(|| std::io::Error::other("maximized result viewport"))?;
    assert!(first_maximized.width > normal_result.width);
    assert_eq!(harness.area_of(editor), None);

    for (width, height) in [
        (72, 20),
        (80, 24),
        (99, 28),
        (100, 30),
        (160, 50),
        (120, 40),
    ] {
        let _ = harness.resize(width, height);
        let area = harness.area_of(result).ok_or_else(|| {
            std::io::Error::other("focused results remain registered after resize")
        })?;
        assert_eq!(harness.area_of(editor), None);
        assert!(
            area.width > 0 && area.height > 0,
            "{width}x{height}: {area:?}"
        );
        assert!(area.right() <= width, "{width}x{height}: {area:?}");
        assert!(area.bottom() <= height, "{width}x{height}: {area:?}");
        assert!(harness.app().workbench.maximized);
        assert!(
            harness.diagnostics().is_empty(),
            "{width}x{height}: {:?}",
            harness.diagnostics()
        );
    }
    Ok(())
}

#[test]
fn maximized_results_pane_renders_each_query_content_state_without_hidden_controls()
-> Result<(), std::io::Error> {
    let mut harness = workbench();
    let key = harness
        .app()
        .workbench
        .active_key()
        .ok_or_else(|| std::io::Error::other("initial query tab"))?;
    let editor = key.control("query");
    let result = key.control("results");
    assert!(tab_navigation::tab_to(&mut harness, result));
    let _ = harness.key(KeyCode::Char('z'));
    assert!(tab_navigation::tab_to(&mut harness, TAB_STRIP));

    for content in [
        PaneContent::Results,
        PaneContent::EmptyResults,
        PaneContent::Error,
        PaneContent::Plan,
        PaneContent::Idle,
    ] {
        set_pane_content(&mut harness, key, content)?;
        harness.draw();
        assert!(harness.app().workbench.maximized);
        assert_eq!(harness.area_of(editor), None, "editor stays hidden");
        assert_pane_content(&harness, key, result, content)?;
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    Ok(())
}
