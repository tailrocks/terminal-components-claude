#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use std::time::Duration;

use tablepro_ui::{Screen, Surface, Tab, TableProApp};
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

fn workbench() -> Harness<TableProApp> {
    workbench_at(120, 40)
}

fn workbench_at(width: u16, height: u16) -> Harness<TableProApp> {
    let mut app = TableProApp::new_with_query_duration(Duration::from_millis(42));
    app.screen = Screen::Workbench;
    app.surface = Surface::WorkbenchDefault;
    Harness::new(app, Theme::junie(), width, height)
}

#[test]
fn z_from_the_result_grid_maximizes_only_that_query_pane() -> Result<(), std::io::Error> {
    let mut harness = workbench();
    let key = harness
        .app()
        .workbench
        .active_key()
        .ok_or_else(|| std::io::Error::other("the workbench opens an initial query tab"))?;
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
    let key = harness
        .app()
        .workbench
        .active_key()
        .ok_or_else(|| std::io::Error::other("the workbench opens an initial query tab"))?;
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
    let key = harness
        .app()
        .workbench
        .active_key()
        .ok_or_else(|| std::io::Error::other("the workbench opens an initial query tab"))?;
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
    let key = harness
        .app()
        .workbench
        .active_key()
        .ok_or_else(|| std::io::Error::other("the workbench opens an initial query tab"))?;
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
