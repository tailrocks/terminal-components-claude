//! Mouse-only navigation in tablepro (docs/audits/design-review-2026-10/mouse.md):
//! every state reachable, and every view left, by pointer alone.

use crate::app_tests::H;

// mouse #6: the empty connection list offers the way in as a button
#[test]
fn the_first_connection_is_created_by_mouse_from_the_empty_list() {
    let mut h = H::new(120, 40);
    h.app.connections.connections.clear();
    h.draw();
    assert!(h.text().contains("No connections"), "{}", h.text());
    assert!(!h.text().contains("Save & connect"));
    let (x, y) = h.find("New connection").expect("a New connection button");
    assert_eq!(
        h.app.hits.hit(ratatui::layout::Position::new(x, y)),
        Some(crate::connections::NEW_ID)
    );
    h.click(x, y);
    assert!(h.text().contains("Save & connect"), "{}", h.text());
}

// mouse #7: the query tab runs, runs all, explains and cancels by buttons
#[test]
fn the_query_toolbar_runs_explains_and_cancels_by_mouse() {
    use crate::tabs::{TOOLBAR_CANCEL, TOOLBAR_EXPLAIN, TOOLBAR_RUN, TOOLBAR_RUN_ALL};
    let mut h = H::connected(120, 40);
    h.key(ratatui::crossterm::event::KeyCode::Tab); // editor
    h.type_str("SELECT * FROM orders LIMIT 5");
    h.key(ratatui::crossterm::event::KeyCode::Esc);
    let at = |h: &H, id| {
        let r = h
            .app
            .hits
            .area_of(id)
            .unwrap_or_else(|| panic!("toolbar button {id:?} missing:\n{}", h.text()));
        (r.x + 1, r.y)
    };
    for id in [TOOLBAR_RUN, TOOLBAR_RUN_ALL, TOOLBAR_EXPLAIN] {
        assert!(h.app.hits.area_of(id).is_some(), "{}", h.text());
    }
    assert!(h.app.hits.area_of(TOOLBAR_CANCEL).is_none());
    // Run
    let (x, y) = at(&h, TOOLBAR_RUN);
    h.click(x, y);
    assert!(h.wb_query().is_running(), "Run did not start the query");
    // Cancel shows while it runs
    let (x, y) = at(&h, TOOLBAR_CANCEL);
    h.click(x, y);
    assert!(!h.wb_query().is_running(), "Cancel did not stop it");
    // Run all, then Explain
    let (x, y) = at(&h, TOOLBAR_RUN_ALL);
    h.click(x, y);
    assert!(h.wb_query().is_running());
    h.ticks(10);
    assert!(!h.text().contains("Planning"), "{}", h.text());
    let (x, y) = at(&h, TOOLBAR_EXPLAIN);
    h.click(x, y);
    h.ticks(10);
    assert!(h.text().contains("Planning"), "{}", h.text());
}

// mouse #8: a hidden explorer leaves a rail that brings it back, and the
// open explorer collapses by its button
#[test]
fn the_explorer_collapses_and_comes_back_by_mouse() {
    use crate::workbench::{EXPLORER, EXPLORER_RAIL};
    let mut h = H::connected(120, 40);
    assert!(h.text().contains("Explorer"));
    assert!(
        h.app.hits.area_of(EXPLORER_RAIL).is_none(),
        "no rail while open"
    );
    let collapse = h
        .app
        .hits
        .area_of(EXPLORER_RAIL.part(junie_tui::core::id::PartRef::of(
            junie_tui::core::id::Part::ICON,
        )))
        .expect("the explorer has a collapse button");
    h.click(collapse.x + 1, collapse.y);
    assert!(!h.wb().explorer_visible, "{}", h.text());
    let rail = h
        .app
        .hits
        .area_of(EXPLORER_RAIL)
        .expect("a collapsed explorer leaves a rail");
    h.click(rail.x, rail.y + 3);
    assert!(h.wb().explorer_visible, "{}", h.text());
    assert_eq!(h.focus(), Some(EXPLORER));
    // Ctrl+B hides it too, and the rail still brings it back
    h.ctrl('b');
    assert!(!h.wb().explorer_visible);
    let rail = h
        .app
        .hits
        .area_of(EXPLORER_RAIL)
        .expect("rail after Ctrl+B");
    h.click(rail.x, rail.y);
    assert!(h.wb().explorer_visible);
}

// mouse #8: below 100 columns the unfocused explorer drawer keeps a rail
#[test]
fn a_narrow_workbench_opens_the_explorer_drawer_from_its_rail() {
    use crate::workbench::{EXPLORER, EXPLORER_RAIL};
    let mut h = H::connected(90, 30);
    h.key(ratatui::crossterm::event::KeyCode::Tab);
    assert_ne!(h.focus(), Some(EXPLORER));
    let rail = h
        .app
        .hits
        .area_of(EXPLORER_RAIL)
        .expect("the closed drawer leaves a rail");
    h.click(rail.x, rail.y + 1);
    assert_eq!(h.focus(), Some(EXPLORER), "{}", h.text());
    assert!(h.text().contains("Explorer"));
}

// mouse #40: the TablePro segment opens the app menu: open tabs, disconnect, quit
#[test]
fn the_app_menu_disconnects_and_quits_by_mouse() {
    use crate::app::STRIP_APP;
    let mut h = H::connected(120, 40);
    let open_menu = |h: &mut H| {
        let r = h.app.hits.area_of(STRIP_APP).expect("the TablePro segment");
        h.click(r.x, r.y);
    };
    open_menu(&mut h);
    assert!(
        h.text().contains("Disconnect") && h.text().contains("Quit TablePro"),
        "{}",
        h.text()
    );
    let (x, y) = h.find("Disconnect").unwrap();
    h.click(x, y);
    assert!(h.app.workbench.is_none(), "{}", h.text());
    assert!(h.text().contains("Connections"), "{}", h.text());
    // on the connection list the menu has Quit only
    open_menu(&mut h);
    assert!(!h.text().contains("Open tabs…"), "{}", h.text());
    let (x, y) = h.find("Quit TablePro").unwrap();
    h.click(x, y);
    assert!(h.app.quit, "Quit did not quit");
}

// mouse #40: pending work keeps the connection open
#[test]
fn disconnect_keeps_an_unsaved_query() {
    use crate::app::STRIP_APP;
    let mut h = H::connected(120, 40);
    h.key(ratatui::crossterm::event::KeyCode::Tab);
    h.type_str("SELECT 1");
    h.key(ratatui::crossterm::event::KeyCode::Esc);
    let r = h.app.hits.area_of(STRIP_APP).unwrap();
    h.click(r.x, r.y);
    let (x, y) = h.find("Disconnect").unwrap();
    h.click(x, y);
    assert!(h.app.workbench.is_some());
    assert!(h.text().contains("before disconnecting"), "{}", h.text());
}
